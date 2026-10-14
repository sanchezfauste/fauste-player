#![allow(clippy::unwrap_used)]
mod support;

use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use fp_model::{Command, Transport};
use fp_remote::{RemoteHandle, ServerStatus, spawn};
use rosc::{OscMessage, OscPacket, OscType, decoder, encoder};
use support::{FakeControl, demo_state};

fn free_udp_port() -> u16 {
    UdpSocket::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn osc_addr(handle: &RemoteHandle) -> SocketAddr {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let ServerStatus::Listening(a) = handle.status().osc {
            return a;
        }
        assert!(Instant::now() < deadline, "{:?}", handle.status());
        std::thread::sleep(Duration::from_millis(10));
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

fn enabled(fake: &FakeControl) -> u16 {
    let port = free_udp_port();
    fake.edit(|s| {
        s.config.remote.osc.enabled = true;
        s.config.remote.osc.port = port;
    });
    port
}

fn wait_sent(fake: &FakeControl, n: usize) -> Vec<Command> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let sent = fake.sent.lock().unwrap().clone();
        if sent.len() >= n {
            return sent;
        }
        assert!(Instant::now() < deadline, "only {sent:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn a_play_message_queues_play() {
    let fake = FakeControl::new(demo_state());
    enabled(&fake);
    let handle = spawn(fake.clone()).unwrap();
    let to = osc_addr(&handle);
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    send(&sock, to, "/fauste/player/1/play", vec![OscType::Int(0)]);
    send(&sock, to, "/fauste/player/1/play", vec![OscType::Int(1)]);
    let p = fake.state.load().players[0].id;
    assert_eq!(wait_sent(&fake, 1), vec![Command::Play(p)]);
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(fake.take_sent().len(), 1, "the release did nothing");
}

#[test]
fn packets_from_other_sources_are_dropped() {
    let fake = FakeControl::new(demo_state());
    enabled(&fake);
    fake.edit(|s| s.config.remote.osc.allowed_sources = vec!["10.0.0.0/8".into()]);
    let handle = spawn(fake.clone()).unwrap();
    let to = osc_addr(&handle);
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    send(&sock, to, "/fauste/player/1/play", vec![]);
    sock.send_to(b"garbage", to).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    assert!(fake.take_sent().is_empty());
    assert!(
        matches!(handle.status().osc, ServerStatus::Listening(_)),
        "still running"
    );
}

#[test]
fn a_subscriber_gets_a_dump_then_changes() {
    let fake = FakeControl::new(demo_state());
    enabled(&fake);
    let handle = spawn(fake.clone()).unwrap();
    let to = osc_addr(&handle);
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    send(&sock, to, "/fauste/subscribe", vec![]);
    let mut buf = [0u8; 65536];
    let mut seen: Vec<(String, Vec<OscType>)> = Vec::new();
    let mut read_until = |want: &str, seen: &mut Vec<(String, Vec<OscType>)>| {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !seen
            .iter()
            .any(|(a, v)| format!("{a} {v:?}").contains(want))
        {
            assert!(Instant::now() < deadline, "never saw {want}: {seen:?}");
            let (n, _) = sock.recv_from(&mut buf).unwrap();
            if let Ok((_, OscPacket::Message(m))) = decoder::decode_udp(&buf[..n]) {
                seen.push((m.addr, m.args));
            }
        }
    };
    read_until(
        "/fauste/player/1/transport [String(\"stopped\")]",
        &mut seen,
    );
    // Drain the rest of the dump before changing anything.
    sock.set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    let mut spill = [0u8; 65536];
    while sock.recv_from(&mut spill).is_ok() {}
    sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let p = fake.state.load().players[0].id;
    fake.edit(|s| {
        fp_model::apply(s, Command::Play(p)).unwrap();
        assert_eq!(s.players[0].transport, Transport::Playing);
    });
    seen.clear();
    read_until(
        "/fauste/player/1/transport [String(\"playing\")]",
        &mut seen,
    );
    assert!(
        !seen.iter().any(|(a, _)| a == "/fauste/player/2/transport"),
        "unchanged values are not resent: {seen:?}"
    );
}

#[test]
fn disabling_osc_closes_the_socket_and_dropping_stops_it() {
    let fake = FakeControl::new(demo_state());
    let port = enabled(&fake);
    let handle = spawn(fake.clone()).unwrap();
    osc_addr(&handle);
    fake.edit(|s| s.config.remote.osc.enabled = false);
    let deadline = Instant::now() + Duration::from_secs(5);
    while handle.status().osc != ServerStatus::Off {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(UdpSocket::bind(("127.0.0.1", port)).is_ok(), "port freed");
    drop(handle);
}

#[test]
fn a_grid_resize_sends_everything_again() {
    let fake = FakeControl::new(demo_state());
    enabled(&fake);
    let handle = spawn(fake.clone()).unwrap();
    let to = osc_addr(&handle);
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    send(&sock, to, "/fauste/subscribe", vec![]);
    let mut buf = [0u8; 65536];
    sock.set_read_timeout(Some(Duration::from_millis(300)))
        .unwrap();
    let mut first = 0;
    while sock.recv_from(&mut buf).is_ok() {
        first += 1;
    }
    assert!(first > 0, "no dump");
    fake.edit(|s| {
        let page = s.cartwall.pages[0].id;
        let (rows, cols) = (s.cartwall.pages[0].rows, s.cartwall.pages[0].cols);
        fp_model::apply(
            s,
            Command::ResizeCartPage {
                page,
                rows: rows + 1,
                cols,
            },
        )
        .unwrap();
    });
    sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        assert!(Instant::now() < deadline, "no full dump after the resize");
        let (n, _) = sock.recv_from(&mut buf).unwrap();
        if let Ok((_, OscPacket::Message(m))) = decoder::decode_udp(&buf[..n])
            && m.addr == "/fauste/player/1/transport"
        {
            break;
        }
    }
}

#[test]
fn fewer_players_clear_the_addresses_of_the_removed_ones() {
    let fake = FakeControl::new(demo_state());
    enabled(&fake);
    let handle = spawn(fake.clone()).unwrap();
    let to = osc_addr(&handle);
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    send(&sock, to, "/fauste/subscribe", vec![]);
    let mut buf = [0u8; 65536];
    sock.set_read_timeout(Some(Duration::from_millis(300)))
        .unwrap();
    while sock.recv_from(&mut buf).is_ok() {}
    fake.edit(|s| {
        fp_model::apply(s, Command::SetPlayerCount(2)).unwrap();
        assert_eq!(s.players.len(), 2);
    });
    sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        assert!(Instant::now() < deadline, "player 4 was never cleared");
        let (n, _) = sock.recv_from(&mut buf).unwrap();
        if let Ok((_, OscPacket::Message(m))) = decoder::decode_udp(&buf[..n])
            && m.addr == "/fauste/player/4/transport"
        {
            assert_eq!(m.args, vec![OscType::String(String::new())]);
            break;
        }
    }
    drop(handle);
}
