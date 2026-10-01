#![allow(clippy::unwrap_used)]
mod support;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::{Duration, Instant};

use fp_remote::{RemoteHandle, ServerError, ServerStatus, spawn};
use support::{FakeControl, demo_state};

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn wait_for(handle: &RemoteHandle, what: &str, ok: impl Fn(&ServerStatus) -> bool) -> ServerStatus {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let s = handle.status().http;
        if ok(&s) {
            return s;
        }
        assert!(Instant::now() < deadline, "waiting for {what}; last {s:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn get(addr: SocketAddr, path: &str) -> u16 {
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(
        s,
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    out.split(' ').nth(1).unwrap().parse().unwrap()
}

#[test]
fn disabled_by_default_and_nothing_listens() {
    let fake = FakeControl::new(demo_state());
    let handle = spawn(fake).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(handle.status().http, ServerStatus::Off);
}

#[test]
fn enabling_it_at_run_time_starts_listening_and_answers() {
    let fake = FakeControl::new(demo_state());
    let handle = spawn(fake.clone()).unwrap();
    let port = free_port();
    fake.edit(|s| {
        s.config.remote.http.enabled = true;
        s.config.remote.http.port = port;
    });
    let s = wait_for(&handle, "listening", |s| {
        matches!(s, ServerStatus::Listening(_))
    });
    let ServerStatus::Listening(addr) = s else {
        unreachable!()
    };
    assert_eq!(addr.port(), port);
    assert_eq!(get(addr, "/api/v1/state"), 200);
    assert!(
        fake.take_sent().is_empty(),
        "rule 10: starting sends nothing"
    );
}

#[test]
fn a_port_change_moves_the_server_and_disabling_closes_it() {
    let fake = FakeControl::new(demo_state());
    let first = free_port();
    fake.edit(|s| {
        s.config.remote.http.enabled = true;
        s.config.remote.http.port = first;
    });
    let handle = spawn(fake.clone()).unwrap();
    wait_for(
        &handle,
        "first port",
        |s| matches!(s, ServerStatus::Listening(a) if a.port() == first),
    );
    let second = free_port();
    fake.edit(|s| s.config.remote.http.port = second);
    wait_for(
        &handle,
        "second port",
        |s| matches!(s, ServerStatus::Listening(a) if a.port() == second),
    );
    assert!(
        TcpStream::connect(("127.0.0.1", first)).is_err(),
        "old port still open"
    );
    fake.edit(|s| s.config.remote.http.enabled = false);
    wait_for(&handle, "off", |s| *s == ServerStatus::Off);
    assert!(TcpStream::connect(("127.0.0.1", second)).is_err());
}

#[test]
fn a_port_in_use_is_an_error_status_not_a_crash() {
    let taken = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = taken.local_addr().unwrap().port();
    let fake = FakeControl::new(demo_state());
    fake.edit(|s| {
        s.config.remote.http.enabled = true;
        s.config.remote.http.port = port;
    });
    let handle = spawn(fake).unwrap();
    let s = wait_for(&handle, "error", |s| matches!(s, ServerStatus::Error(_)));
    assert!(
        matches!(s, ServerStatus::Error(ServerError::Bind(_))),
        "{s:?}"
    );
}

#[test]
fn beyond_loopback_without_a_token_it_refuses_to_start() {
    let fake = FakeControl::new(demo_state());
    fake.edit(|s| {
        s.config.remote.http.enabled = true;
        s.config.remote.http.bind = "0.0.0.0".into();
        s.config.remote.http.port = free_port();
    });
    let handle = spawn(fake).unwrap();
    let s = wait_for(&handle, "error", |s| matches!(s, ServerStatus::Error(_)));
    assert_eq!(s, ServerStatus::Error(ServerError::TokenRequired));
}

#[test]
fn dropping_the_handle_stops_the_server() {
    let fake = FakeControl::new(demo_state());
    let port = free_port();
    fake.edit(|s| {
        s.config.remote.http.enabled = true;
        s.config.remote.http.port = port;
    });
    let handle = spawn(fake).unwrap();
    let ServerStatus::Listening(addr) = wait_for(&handle, "listening", |s| {
        matches!(s, ServerStatus::Listening(_))
    }) else {
        unreachable!()
    };
    // An open event stream must not keep the server alive.
    let mut stream = TcpStream::connect(addr).unwrap();
    write!(
        stream,
        "GET /api/v1/events HTTP/1.1\r\nHost: {addr}\r\n\r\n"
    )
    .unwrap();
    let mut buf = [0u8; 1024];
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    assert!(stream.read(&mut buf).unwrap() > 0);
    let started = Instant::now();
    drop(handle);
    assert!(started.elapsed() < Duration::from_secs(5));
    assert!(TcpStream::connect(("127.0.0.1", port)).is_err());
}
#[test]
fn an_event_stream_sees_a_change_made_after_it_connected() {
    let fake = FakeControl::new(demo_state());
    let port = free_port();
    fake.edit(|s| {
        s.config.remote.http.enabled = true;
        s.config.remote.http.port = port;
    });
    let handle = spawn(fake.clone()).unwrap();
    let ServerStatus::Listening(addr) = wait_for(&handle, "listening", |s| {
        matches!(s, ServerStatus::Listening(_))
    }) else {
        unreachable!()
    };
    let mut s = TcpStream::connect(addr).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(s, "GET /api/v1/events HTTP/1.1\r\nHost: {addr}\r\n\r\n").unwrap();
    let mut seen = String::new();
    let mut buf = [0u8; 4096];
    while !seen.contains("event: state") {
        let n = s.read(&mut buf).unwrap();
        seen.push_str(&String::from_utf8_lossy(&buf[..n]));
    }
    let night = fake.state.load().playlists.iter().nth(1).unwrap().id;
    fake.edit(|st| {
        fp_model::apply(st, fp_model::Command::DeletePlaylist(night)).unwrap();
    });
    while !seen.contains("event: playlist-removed") {
        let n = s.read(&mut buf).unwrap();
        assert!(n > 0, "stream closed: {seen}");
        seen.push_str(&String::from_utf8_lossy(&buf[..n]));
    }
    drop(handle);
}
