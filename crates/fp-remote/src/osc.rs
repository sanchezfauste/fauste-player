//! OSC (remote control spec §4): addresses under `/fauste` turned into
//! operations, the values sent to subscribers, and what each subscriber was
//! last sent. Pure; `osc_server` does the I/O.

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use fp_model::{AppState, CartId, CartPage, PlayerId};
use rosc::{OscMessage, OscPacket, OscType};

use crate::api::Operation;
use crate::control::Playback;
use crate::dto;

/// Bundles nested deeper than this are refused. rosc decodes them
/// recursively, so the check runs on the raw bytes first: a hostile packet
/// would otherwise overflow the remote thread's stack.
const MAX_BUNDLE_DEPTH: usize = 8;
/// Arrays (`[` in the type tags) nested deeper than this are refused, for
/// the same reason.
const MAX_ARRAY_DEPTH: usize = 4;

#[derive(Debug, Clone, PartialEq)]
pub enum OscRequest {
    Op(Operation),
    /// To the sender's address, on this port or the packet's source port.
    Subscribe(Option<u16>),
    Unsubscribe(Option<u16>),
}

/// The messages of a packet, bundles flattened in order (time tags ignored).
pub fn messages(packet: &[u8]) -> Result<Vec<OscMessage>, &'static str> {
    if !within_limits(packet, 0) {
        return Err("OSC packet nested too deeply or malformed");
    }
    let (_, packet) = rosc::decoder::decode_udp(packet).map_err(|_| "malformed OSC packet")?;
    let mut out = Vec::new();
    flatten(packet, &mut out, 0);
    Ok(out)
}

/// Whether `packet` nests bundles and arrays within the limits. Recursion
/// here is bounded by `MAX_BUNDLE_DEPTH`.
fn within_limits(packet: &[u8], depth: usize) -> bool {
    if packet.starts_with(b"#bundle\0") {
        if depth >= MAX_BUNDLE_DEPTH {
            return false;
        }
        // Tag (8 bytes) and time tag (8 bytes), then size-prefixed elements.
        let mut rest = packet.get(16..).unwrap_or_default();
        while !rest.is_empty() {
            let Some(size) = rest.get(..4).and_then(|b| <[u8; 4]>::try_from(b).ok()) else {
                return false;
            };
            let size = u32::from_be_bytes(size) as usize;
            let Some(element) = rest.get(4..4 + size) else {
                return false;
            };
            if !within_limits(element, depth + 1) {
                return false;
            }
            rest = rest.get(4 + size..).unwrap_or_default();
        }
        true
    } else {
        arrays_within_limits(packet)
    }
}

/// The type tags of a message (the string after the padded address) nest
/// arrays no deeper than `MAX_ARRAY_DEPTH`.
fn arrays_within_limits(message: &[u8]) -> bool {
    let Some(end) = message.iter().position(|b| *b == 0) else {
        return true; // not a message rosc can read; it refuses it itself
    };
    let tags_at = (end / 4 + 1) * 4;
    let tags = message.get(tags_at..).unwrap_or_default();
    let mut depth = 0usize;
    for b in tags.iter().take_while(|b| **b != 0) {
        match b {
            b'[' => {
                depth += 1;
                if depth > MAX_ARRAY_DEPTH {
                    return false;
                }
            }
            b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    true
}

fn flatten(packet: OscPacket, out: &mut Vec<OscMessage>, depth: usize) {
    match packet {
        OscPacket::Message(m) => out.push(m),
        OscPacket::Bundle(b) if depth < MAX_BUNDLE_DEPTH => {
            for p in b.content {
                flatten(p, out, depth + 1);
            }
        }
        OscPacket::Bundle(_) => {}
    }
}

pub fn parse(msg: &OscMessage, state: &AppState) -> Result<OscRequest, &'static str> {
    let path = msg
        .addr
        .strip_prefix("/fauste/")
        .ok_or("not a /fauste address")?;
    let parts: Vec<&str> = path.split('/').collect();
    let args = msg.args.as_slice();
    let op = match parts.as_slice() {
        ["player", n, action] => {
            let p = player_at(state, n)?;
            match *action {
                "cue" => return Ok(OscRequest::Op(Operation::SetCue(p, switch(args)?))),
                "volume" => {
                    let v = number(args).ok_or("volume needs a number")?;
                    return Ok(OscRequest::Op(Operation::SetFader(p, v as f32)));
                }
                "play" => Operation::Play(p),
                "pause" => Operation::Pause(p),
                "stop" => Operation::Stop(p),
                "fade-stop" => Operation::FadeStop(p),
                "restart" => Operation::Restart(p),
                "previous" => Operation::Previous(p),
                _ => return Err("unknown player action"),
            }
        }
        ["cart", c, "fire"] => {
            let page = state.cartwall.shown_page().ok_or("no cart page")?;
            Operation::FireCart(cart_at(page, c)?)
        }
        ["cartwall", "page", p, "cart", c, "fire"] => {
            Operation::FireCart(cart_at(page_at(state, p)?, c)?)
        }
        ["cartwall", "stop-all"] => Operation::StopAllCarts,
        ["cartwall", "page", dir @ ("next" | "previous")] => {
            let pages = &state.cartwall.pages;
            let shown = state.cartwall.shown_page().map(|p| p.id);
            let i = pages.iter().position(|p| Some(p.id) == shown).unwrap_or(0);
            let j = if *dir == "next" {
                i + 1
            } else {
                i.checked_sub(1).ok_or("no previous page")?
            };
            Operation::ShowCartPage(pages.get(j).ok_or("no next page")?.id)
        }
        ["subscribe"] => return Ok(OscRequest::Subscribe(port(args)?)),
        ["unsubscribe"] => return Ok(OscRequest::Unsubscribe(port(args)?)),
        _ => return Err("unknown address"),
    };
    pressed(args)?;
    Ok(OscRequest::Op(op))
}

fn number(args: &[OscType]) -> Option<f64> {
    match args.first()? {
        OscType::Int(i) => Some(f64::from(*i)),
        OscType::Float(f) => Some(f64::from(*f)),
        OscType::Double(d) => Some(*d),
        OscType::Long(l) => Some(*l as f64),
        OscType::Bool(b) => Some(f64::from(u8::from(*b))),
        _ => None,
    }
}

/// A button: no argument, or a number above zero (a release sends zero).
fn pressed(args: &[OscType]) -> Result<(), &'static str> {
    if args.is_empty() {
        return Ok(());
    }
    match number(args) {
        Some(v) if v > 0.0 => Ok(()),
        Some(_) => Err("button released"),
        None => Err("a button takes a number or nothing"),
    }
}

fn switch(args: &[OscType]) -> Result<bool, &'static str> {
    number(args).map(|v| v > 0.0).ok_or("needs on or off")
}

fn port(args: &[OscType]) -> Result<Option<u16>, &'static str> {
    if args.is_empty() {
        return Ok(None);
    }
    number(args)
        .filter(|v| v.fract() == 0.0)
        .and_then(|v| u16::try_from(v as i64).ok())
        .filter(|p| *p != 0)
        .map(Some)
        .ok_or("bad port")
}

/// A 1-based position as an index.
fn index(raw: &str) -> Result<usize, &'static str> {
    raw.parse::<usize>()
        .ok()
        .and_then(|n| n.checked_sub(1))
        .ok_or("positions start at 1")
}

fn player_at(state: &AppState, raw: &str) -> Result<PlayerId, &'static str> {
    state
        .players
        .get(index(raw)?)
        .map(|p| p.id)
        .ok_or("no such player")
}

fn page_at<'a>(state: &'a AppState, raw: &str) -> Result<&'a CartPage, &'static str> {
    state
        .cartwall
        .pages
        .get(index(raw)?)
        .ok_or("no such cart page")
}

fn cart_at(page: &CartPage, raw: &str) -> Result<CartId, &'static str> {
    page.carts
        .get(index(raw)?)
        .map(|c| c.id)
        .ok_or("no such cart")
}

fn flag(on: bool) -> OscType {
    OscType::Int(i32::from(on))
}

fn text(s: Option<&str>) -> OscType {
    OscType::String(s.unwrap_or_default().to_owned())
}

/// Every address subscribers follow, with its current value (spec §4.2).
pub fn values(model: &AppState, playback: &Playback) -> Vec<(String, OscType)> {
    let mut out = Vec::new();
    for p in dto::players(model, playback) {
        let n = p.position;
        let at = |leaf: &str| format!("/fauste/player/{n}/{leaf}");
        let current = p.current.as_ref().map(|c| &c.track);
        let next = p.next.as_ref();
        out.push((at("transport"), OscType::String(p.transport.to_owned())));
        out.push((at("fading"), flag(p.fading)));
        out.push((at("cueing"), flag(p.cue.is_some())));
        out.push((at("stop-after-current"), flag(p.stop_after_current)));
        out.push((at("volume"), OscType::Float(p.fader)));
        out.push((at("title"), text(current.map(|t| t.title.as_str()))));
        out.push((at("artist"), text(current.map(|t| t.artist.as_str()))));
        out.push((
            at("elapsed"),
            OscType::Float(p.elapsed_secs.unwrap_or(0.0) as f32),
        ));
        out.push((
            at("remaining"),
            OscType::Float(p.remaining_secs.unwrap_or(0.0) as f32),
        ));
        let entry = next.map_or(-1, |e| i64::try_from(e.entry.0).unwrap_or(-1));
        out.push((at("next/entry"), OscType::Long(entry)));
        out.push((at("next/title"), text(next.map(|e| e.track.title.as_str()))));
        out.push((
            at("next/artist"),
            text(next.map(|e| e.track.artist.as_str())),
        ));
    }
    let cw = dto::cartwall(model, playback);
    let shown = cw
        .pages
        .iter()
        .enumerate()
        .find(|(_, page)| Some(page.id) == cw.shown_page);
    if let Some((i, page)) = shown {
        out.push((
            "/fauste/cartwall/page".to_owned(),
            OscType::Int(i32::try_from(i + 1).unwrap_or(i32::MAX)),
        ));
        for c in &page.carts {
            let n = c.index + 1;
            let playing = cw.playing.iter().any(|pc| pc.cart == c.id);
            out.push((format!("/fauste/cart/{n}/playing"), flag(playing)));
            out.push((
                format!("/fauste/cart/{n}/name"),
                OscType::String(c.name.clone()),
            ));
        }
    }
    out
}

struct Subscriber {
    expires: Instant,
    /// The last value sent to it, per address.
    sent: HashMap<String, OscType>,
    /// Send every value on the next `changes`, changed or not.
    resend: bool,
}

/// The empty value of `value`'s type, sent once to an address that no
/// longer exists (a removed player, a cart beyond a smaller grid) so a
/// surface does not keep showing its last value. A `Long` is an entry id,
/// whose "none" is -1.
pub fn cleared(value: &OscType) -> OscType {
    match value {
        OscType::String(_) => OscType::String(String::new()),
        OscType::Float(_) => OscType::Float(0.0),
        OscType::Double(_) => OscType::Double(0.0),
        OscType::Int(_) => OscType::Int(0),
        OscType::Long(_) => OscType::Long(-1),
        OscType::Bool(_) => OscType::Bool(false),
        _ => OscType::Nil,
    }
}

/// OSC subscribers and what each was last sent (spec §5.3).
pub struct Subscribers {
    max: usize,
    ttl: Duration,
    list: HashMap<SocketAddr, Subscriber>,
}

impl Subscribers {
    pub fn new(max: usize, ttl: Duration) -> Self {
        Self {
            max,
            ttl,
            list: HashMap::new(),
        }
    }

    /// Adds or renews `to`; false when the table is full.
    pub fn subscribe(&mut self, to: SocketAddr, now: Instant) -> bool {
        if let Some(s) = self.list.get_mut(&to) {
            s.expires = now + self.ttl;
            return true;
        }
        if self.list.len() >= self.max {
            return false;
        }
        self.list.insert(
            to,
            Subscriber {
                expires: now + self.ttl,
                sent: HashMap::new(),
                resend: false,
            },
        );
        true
    }

    pub fn unsubscribe(&mut self, to: SocketAddr) {
        self.list.remove(&to);
    }

    pub fn expire(&mut self, now: Instant) {
        self.list.retain(|_, s| s.expires > now);
    }

    /// Makes the next `changes` a full dump. What was sent is kept, so
    /// addresses that disappeared are still cleared.
    pub fn reset(&mut self) {
        for s in self.list.values_mut() {
            s.resend = true;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// For each subscriber, the messages whose value it has not been sent,
    /// and the empty value of each address it was sent that no longer
    /// exists.
    pub fn changes(&mut self, values: &[(String, OscType)]) -> Vec<(SocketAddr, Vec<OscMessage>)> {
        let current: HashSet<&str> = values.iter().map(|(a, _)| a.as_str()).collect();
        let mut out = Vec::new();
        for (to, s) in &mut self.list {
            let mut messages = Vec::new();
            let gone: Vec<String> = s
                .sent
                .keys()
                .filter(|a| !current.contains(a.as_str()))
                .cloned()
                .collect();
            for addr in gone {
                if let Some(old) = s.sent.remove(&addr) {
                    messages.push(OscMessage {
                        addr,
                        args: vec![cleared(&old)],
                    });
                }
            }
            let resend = std::mem::take(&mut s.resend);
            for (addr, value) in values {
                if resend || s.sent.get(addr) != Some(value) {
                    s.sent.insert(addr.clone(), value.clone());
                    messages.push(OscMessage {
                        addr: addr.clone(),
                        args: vec![value.clone()],
                    });
                }
            }
            if !messages.is_empty() {
                out.push((*to, messages));
            }
        }
        out
    }
}
