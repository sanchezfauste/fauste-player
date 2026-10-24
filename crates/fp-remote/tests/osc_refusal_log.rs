#![allow(clippy::unwrap_used)]
//! Bit-perfect spec BF12: an OSC volume below 100 % on a locked player is
//! refused and noted in the remote log with the code `unavailable`. Its own
//! test binary: it installs the global log subscriber (the server logs from
//! its own thread).

#[path = "it/support/mod.rs"]
mod support;

use std::io::Write;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use fp_model::Command;
use fp_remote::{ServerStatus, spawn};
use rosc::{OscMessage, OscPacket, OscType, encoder};
use support::{FakeControl, demo_state};

#[derive(Clone, Default)]
struct Lines(Arc<Mutex<Vec<u8>>>);

impl Write for Lines {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn send(sock: &UdpSocket, to: SocketAddr, addr: &str, args: Vec<OscType>) {
    let bytes = encoder::encode(&OscPacket::Message(OscMessage {
        addr: addr.into(),
        args,
    }))
    .unwrap();
    sock.send_to(&bytes, to).unwrap();
}

#[test]
fn bf12_an_osc_volume_on_a_locked_player_is_refused_noted_and_play_still_works() {
    let lines = Lines::default();
    let writer = lines.clone();
    tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .init();
    let mut s = demo_state();
    support::lock_first_player(&mut s);
    let fake = FakeControl::new(s);
    let port = UdpSocket::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    fake.edit(|s| {
        s.config.remote.osc.enabled = true;
        s.config.remote.osc.port = port;
    });
    let handle = spawn(fake.clone()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let to = loop {
        if let ServerStatus::Listening(a) = handle.status().osc {
            break a;
        }
        assert!(Instant::now() < deadline, "{:?}", handle.status());
        std::thread::sleep(Duration::from_millis(10));
    };
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    send(
        &sock,
        to,
        "/fauste/player/1/volume",
        vec![OscType::Float(0.5)],
    );
    send(&sock, to, "/fauste/player/1/play", vec![]);
    let p = fake.state.load().players[0].id;
    // The two messages are handled in order: once Play is queued, the
    // refusal has been noted.
    let deadline = Instant::now() + Duration::from_secs(5);
    let sent = loop {
        let sent = fake.sent.lock().unwrap().clone();
        if !sent.is_empty() {
            break sent;
        }
        assert!(Instant::now() < deadline, "nothing queued");
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(sent, vec![Command::Play(p)], "the volume queued nothing");
    drop(handle);
    let text = String::from_utf8(lines.0.lock().unwrap().clone()).unwrap();
    let refused = text
        .lines()
        .filter(|l| l.contains("remote input refused"))
        .collect::<Vec<_>>();
    assert_eq!(refused.len(), 1, "{text}");
    assert!(refused[0].contains("unavailable"), "{text}");
}
