#![allow(clippy::unwrap_used)]
mod support;

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use fp_model::{Command, PlayMode};
use fp_remote::api::Operation as O;
use fp_remote::control::Playback;
use fp_remote::osc::{OscRequest, Subscribers, messages, parse, values};
use rosc::{OscBundle, OscMessage, OscPacket, OscTime, OscType, encoder};
use support::demo_state;

fn msg(addr: &str, args: Vec<OscType>) -> OscMessage {
    OscMessage {
        addr: addr.into(),
        args,
    }
}

#[test]
fn player_buttons_act_on_press_and_ignore_release() {
    let s = demo_state();
    let p = s.players[1].id;
    assert_eq!(
        parse(&msg("/fauste/player/2/play", vec![]), &s),
        Ok(OscRequest::Op(O::Play(p)))
    );
    assert_eq!(
        parse(
            &msg("/fauste/player/2/fade-stop", vec![OscType::Float(1.0)]),
            &s
        ),
        Ok(OscRequest::Op(O::FadeStop(p)))
    );
    assert!(parse(&msg("/fauste/player/2/play", vec![OscType::Int(0)]), &s).is_err());
    assert!(
        parse(
            &msg("/fauste/player/2/play", vec![OscType::String("x".into())]),
            &s
        )
        .is_err()
    );
}

#[test]
fn cue_and_volume_take_values() {
    let s = demo_state();
    let p = s.players[0].id;
    assert_eq!(
        parse(&msg("/fauste/player/1/cue", vec![OscType::Bool(true)]), &s),
        Ok(OscRequest::Op(O::SetCue(p, true)))
    );
    assert_eq!(
        parse(&msg("/fauste/player/1/cue", vec![OscType::Int(0)]), &s),
        Ok(OscRequest::Op(O::SetCue(p, false)))
    );
    assert_eq!(
        parse(
            &msg("/fauste/player/1/volume", vec![OscType::Float(0.5)]),
            &s
        ),
        Ok(OscRequest::Op(O::SetFader(p, 0.5)))
    );
    assert!(parse(&msg("/fauste/player/1/volume", vec![]), &s).is_err());
}

#[test]
fn positions_outside_the_screen_are_refused() {
    let s = demo_state();
    for addr in [
        "/fauste/player/0/play",
        "/fauste/player/99/play",
        "/fauste/player/x/play",
        "/fauste/player/1/dance",
        "/fauste/cart/999/fire",
        "/other/thing",
        "/fauste/",
    ] {
        assert!(parse(&msg(addr, vec![]), &s).is_err(), "{addr}");
    }
}

#[test]
fn carts_fire_on_the_page_shown_or_a_given_page() {
    let s = demo_state();
    let first = s.cartwall.pages[0].carts[0].id;
    assert_eq!(
        parse(&msg("/fauste/cart/1/fire", vec![]), &s),
        Ok(OscRequest::Op(O::FireCart(first)))
    );
    assert_eq!(
        parse(&msg("/fauste/cartwall/page/1/cart/1/fire", vec![]), &s),
        Ok(OscRequest::Op(O::FireCart(first)))
    );
    assert_eq!(
        parse(&msg("/fauste/cartwall/stop-all", vec![]), &s),
        Ok(OscRequest::Op(O::StopAllCarts))
    );
    assert!(parse(&msg("/fauste/cartwall/page/previous", vec![]), &s).is_err());
}

#[test]
fn page_next_moves_to_the_following_page() {
    let mut s = demo_state();
    fp_model::apply(&mut s, Command::CreateCartPage { name: "B".into() }).unwrap();
    let second = s.cartwall.pages[1].id;
    assert_eq!(
        parse(&msg("/fauste/cartwall/page/next", vec![]), &s),
        Ok(OscRequest::Op(O::ShowCartPage(second)))
    );
}

#[test]
fn subscriptions_take_an_optional_port() {
    let s = demo_state();
    assert_eq!(
        parse(&msg("/fauste/subscribe", vec![]), &s),
        Ok(OscRequest::Subscribe(None))
    );
    assert_eq!(
        parse(&msg("/fauste/subscribe", vec![OscType::Int(9000)]), &s),
        Ok(OscRequest::Subscribe(Some(9000)))
    );
    assert!(parse(&msg("/fauste/subscribe", vec![OscType::Int(70000)]), &s).is_err());
    assert_eq!(
        parse(&msg("/fauste/unsubscribe", vec![]), &s),
        Ok(OscRequest::Unsubscribe(None))
    );
}

#[test]
fn bundles_are_flattened_and_garbage_is_refused() {
    let bundle = OscPacket::Bundle(OscBundle {
        timetag: OscTime {
            seconds: 0,
            fractional: 1,
        },
        content: vec![
            OscPacket::Message(msg("/fauste/player/1/play", vec![])),
            OscPacket::Message(msg("/fauste/player/2/play", vec![])),
        ],
    });
    let got = messages(&encoder::encode(&bundle).unwrap()).unwrap();
    assert_eq!(got.len(), 2);
    assert!(messages(b"garbage").is_err());
    assert!(messages(&[]).is_err());
}

#[test]
fn values_describe_players_and_the_cart_page() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let second = s.players[1].id;
    fp_model::apply(&mut s, Command::SetMode(second, PlayMode::Single)).unwrap();
    let v = values(
        &s,
        &Playback {
            players: vec![(p, 10.0)],
            ..Default::default()
        },
    );
    let get = |a: &str| {
        v.iter()
            .find(|(addr, _)| addr == a)
            .map(|(_, v)| v.clone())
            .unwrap()
    };
    assert_eq!(
        get("/fauste/player/1/transport"),
        OscType::String("playing".into())
    );
    assert_eq!(get("/fauste/player/1/elapsed"), OscType::Float(10.0));
    assert_eq!(get("/fauste/player/1/remaining"), OscType::Float(170.0));
    assert_eq!(
        get("/fauste/player/2/transport"),
        OscType::String("stopped".into())
    );
    assert_eq!(
        get("/fauste/player/2/title"),
        OscType::String(String::new())
    );
    assert_eq!(get("/fauste/cartwall/page"), OscType::Int(1));
    assert_eq!(get("/fauste/cart/1/playing"), OscType::Int(0));
    assert!(matches!(get("/fauste/player/1/next/entry"), OscType::Long(n) if n > 0));
}

#[test]
fn subscribers_get_everything_first_then_only_changes() {
    let to: SocketAddr = "127.0.0.1:9000".parse().unwrap();
    let mut subs = Subscribers::new(2, Duration::from_secs(60));
    let now = Instant::now();
    assert!(subs.subscribe(to, now));
    let table = vec![
        ("/a".to_owned(), OscType::Int(1)),
        ("/b".to_owned(), OscType::Int(2)),
    ];
    let first = subs.changes(&table);
    assert_eq!(first[0].1.len(), 2);
    assert!(subs.changes(&table).is_empty());
    let changed = vec![
        ("/a".to_owned(), OscType::Int(1)),
        ("/b".to_owned(), OscType::Int(3)),
    ];
    let second = subs.changes(&changed);
    assert_eq!(second[0].1.len(), 1);
    assert_eq!(second[0].1[0].addr, "/b");
    subs.reset();
    assert_eq!(subs.changes(&changed)[0].1.len(), 2);
}

#[test]
fn subscriptions_expire_renew_and_are_capped() {
    let a: SocketAddr = "127.0.0.1:9000".parse().unwrap();
    let b: SocketAddr = "127.0.0.1:9001".parse().unwrap();
    let c: SocketAddr = "127.0.0.1:9002".parse().unwrap();
    let ttl = Duration::from_secs(60);
    let mut subs = Subscribers::new(2, ttl);
    let t0 = Instant::now();
    assert!(subs.subscribe(a, t0));
    assert!(subs.subscribe(b, t0));
    assert!(!subs.subscribe(c, t0), "capped at 2");
    assert!(
        subs.subscribe(a, t0 + Duration::from_secs(50)),
        "renewal always works"
    );
    subs.expire(t0 + Duration::from_secs(61));
    assert_eq!(subs.len(), 1, "b expired, a was renewed");
    subs.unsubscribe(a);
    assert!(subs.is_empty());
}

/// A bundle nested `depth` times around one message, built by hand (the
/// encoder itself would recurse that deep).
fn nested_bundle(depth: usize) -> Vec<u8> {
    let mut inner =
        encoder::encode(&OscPacket::Message(msg("/fauste/player/1/play", vec![]))).unwrap();
    for _ in 0..depth {
        let mut b = b"#bundle\0".to_vec();
        b.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]);
        b.extend_from_slice(&u32::try_from(inner.len()).unwrap().to_be_bytes());
        b.extend_from_slice(&inner);
        inner = b;
    }
    inner
}

/// Runs `f` on a thread with the remote thread's default stack (2 MiB).
fn on_small_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap()
}

#[test]
fn a_deeply_nested_bundle_is_refused_without_overflowing_the_stack() {
    let packet = nested_bundle(3000);
    assert!(on_small_stack(move || messages(&packet).is_err()));
    let shallow = nested_bundle(3);
    assert_eq!(messages(&shallow).unwrap().len(), 1);
}

#[test]
fn deeply_nested_arrays_are_refused() {
    let mut packet = b"/fauste/player/1/play\0\0\0".to_vec();
    let mut tags = String::from(",");
    tags.push_str(&"[".repeat(30_000));
    tags.push_str(&"]".repeat(30_000));
    packet.extend_from_slice(tags.as_bytes());
    packet.push(0);
    while !packet.len().is_multiple_of(4) {
        packet.push(0);
    }
    assert!(on_small_stack(move || messages(&packet).is_err()));
}

#[test]
fn an_address_that_disappears_is_sent_an_empty_value_once() {
    let mut subs = Subscribers::new(4, Duration::from_secs(10));
    let to: SocketAddr = "127.0.0.1:9000".parse().unwrap();
    subs.subscribe(to, Instant::now());
    let all = vec![
        ("/a".to_owned(), OscType::String("x".into())),
        ("/b".to_owned(), OscType::Float(0.5)),
        ("/c".to_owned(), OscType::Int(3)),
        ("/d".to_owned(), OscType::Bool(true)),
        ("/e".to_owned(), OscType::Long(7)),
    ];
    subs.changes(&all);
    let only_a = vec![("/a".to_owned(), OscType::String("x".into()))];
    let out = subs.changes(&only_a);
    let mut got: Vec<(String, Vec<OscType>)> = out[0]
        .1
        .iter()
        .map(|m| (m.addr.clone(), m.args.clone()))
        .collect();
    got.sort_by(|x, y| x.0.cmp(&y.0));
    assert_eq!(
        got,
        vec![
            ("/b".into(), vec![OscType::Float(0.0)]),
            ("/c".into(), vec![OscType::Int(0)]),
            ("/d".into(), vec![OscType::Bool(false)]),
            // An entry id: -1 is "none", as `next/entry` sends it.
            ("/e".into(), vec![OscType::Long(-1)]),
        ]
    );
    assert!(
        subs.changes(&only_a).is_empty(),
        "cleared once, then forgotten"
    );
}

#[test]
fn a_reset_still_clears_what_disappeared() {
    let mut subs = Subscribers::new(4, Duration::from_secs(10));
    let to: SocketAddr = "127.0.0.1:9000".parse().unwrap();
    subs.subscribe(to, Instant::now());
    subs.changes(&[
        ("/a".to_owned(), OscType::Int(1)),
        ("/b".to_owned(), OscType::Int(2)),
    ]);
    subs.reset();
    let out = subs.changes(&[("/a".to_owned(), OscType::Int(1))]);
    let mut got: Vec<(String, Vec<OscType>)> = out[0]
        .1
        .iter()
        .map(|m| (m.addr.clone(), m.args.clone()))
        .collect();
    got.sort_by(|x, y| x.0.cmp(&y.0));
    assert_eq!(
        got,
        vec![
            ("/a".into(), vec![OscType::Int(1)]),
            ("/b".into(), vec![OscType::Int(0)]),
        ],
        "a full dump, plus the cleared address"
    );
}
