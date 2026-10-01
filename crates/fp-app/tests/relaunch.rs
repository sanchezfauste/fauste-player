#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O4: how the application starts itself again.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use fp_app::instance;
use fp_app::restart::{Handoff, Launcher, Relaunch, spawn, wait_for_handoff};

fn plain(exe: &str) -> Launcher {
    Launcher {
        exe: Some(PathBuf::from(exe)),
        ..Launcher::default()
    }
}

#[test]
fn a_plain_install_starts_its_own_executable_without_arguments() {
    assert_eq!(
        plain("/usr/bin/fauste-player").plan(),
        Some(Relaunch {
            program: "/usr/bin/fauste-player".into(),
            args: Vec::new(),
        })
    );
}

#[test]
fn a_replaced_executable_starts_from_its_path() {
    assert_eq!(
        plain("/usr/bin/fauste-player (deleted)")
            .plan()
            .unwrap()
            .program,
        OsString::from("/usr/bin/fauste-player")
    );
}

#[test]
fn an_appimage_starts_the_image_not_the_mounted_copy() {
    let launcher = Launcher {
        appimage: Some("/home/op/Apps/Fauste_Player.AppImage".into()),
        appdir: Some("/tmp/.mount_FausteX".into()),
        ..plain("/tmp/.mount_FausteX/usr/bin/fauste-player")
    };
    assert_eq!(
        launcher.plan().unwrap().program,
        OsString::from("/home/op/Apps/Fauste_Player.AppImage")
    );
}

#[test]
fn a_flatpak_starts_through_the_portal() {
    let launcher = Launcher {
        flatpak: true,
        ..plain("/app/bin/fauste-player")
    };
    assert_eq!(
        launcher.plan(),
        Some(Relaunch {
            program: "flatpak-spawn".into(),
            args: vec!["/app/bin/fauste-player".into()],
        })
    );
}

#[test]
fn a_flatpak_keeps_fauste_home() {
    let launcher = Launcher {
        flatpak: true,
        home: Some("/var/home/op/show".into()),
        ..plain("/app/bin/fauste-player")
    };
    assert_eq!(
        launcher.plan().unwrap().args,
        vec![
            OsString::from("--env=FAUSTE_HOME=/var/home/op/show"),
            OsString::from("/app/bin/fauste-player"),
        ]
    );
}

#[test]
fn without_an_executable_there_is_no_plan() {
    assert_eq!(Launcher::default().plan(), None);
}

#[test]
fn starting_a_missing_program_is_an_error() {
    let plan = Relaunch {
        program: "/nonexistent/fauste-player-test".into(),
        args: Vec::new(),
    };
    assert!(spawn(&plan).is_err());
}

#[test]
fn an_inherited_appimage_from_another_app_is_ignored() {
    // Another AppImage started this one: its variables are inherited.
    let launcher = Launcher {
        appimage: Some("/home/op/Apps/Other.AppImage".into()),
        appdir: Some("/tmp/.mount_OtherY".into()),
        ..plain("/usr/bin/fauste-player")
    };
    assert_eq!(
        launcher.plan().unwrap().program,
        OsString::from("/usr/bin/fauste-player")
    );
}

#[test]
fn an_appimage_without_its_mount_folder_is_not_trusted() {
    let launcher = Launcher {
        appimage: Some("/home/op/Apps/Other.AppImage".into()),
        ..plain("/usr/bin/fauste-player")
    };
    assert_eq!(
        launcher.plan().unwrap().program,
        OsString::from("/usr/bin/fauste-player")
    );
}

#[test]
fn only_a_flatpak_waits_for_the_handoff() {
    let flatpak = Launcher {
        flatpak: true,
        ..plain("/app/bin/fauste-player")
    };
    assert!(flatpak.needs_handoff());
    assert!(!plain("/usr/bin/fauste-player").needs_handoff());
}

/// A program that is always there while the tests run.
fn cargo(arg: &str) -> Command {
    let mut command = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command.arg(arg).stdout(Stdio::null()).stderr(Stdio::null());
    command
}

#[test]
fn the_handoff_is_done_when_the_new_instance_holds_the_lock() {
    let dir = tempfile::tempdir().unwrap();
    let _new_instance = instance::acquire(dir.path()).unwrap().unwrap();
    let mut child = cargo("--version").spawn().unwrap();
    let handoff = wait_for_handoff(&mut child, dir.path(), Duration::from_secs(10));
    assert_eq!(handoff.unwrap(), Handoff::Taken);
    let _ = child.wait();
}

#[test]
fn a_launcher_that_fails_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let mut child = cargo("--no-such-option").spawn().unwrap();
    assert!(wait_for_handoff(&mut child, dir.path(), Duration::from_secs(10)).is_err());
}

#[test]
fn the_wait_for_the_handoff_is_bounded() {
    let dir = tempfile::tempdir().unwrap();
    let mut child = cargo("--version").spawn().unwrap();
    let started = std::time::Instant::now();
    let handoff = wait_for_handoff(&mut child, dir.path(), Duration::from_millis(300));
    assert_eq!(handoff.unwrap(), Handoff::TimedOut);
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[cfg(unix)]
#[test]
fn an_appimage_behind_a_symlinked_folder_is_still_ours() {
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("real");
    std::fs::create_dir_all(real.join("usr/bin")).unwrap();
    std::fs::write(real.join("usr/bin/fauste-player"), b"").unwrap();
    let link = dir.path().join("link");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let launcher = Launcher {
        appimage: Some("/home/op/Apps/Fauste_Player.AppImage".into()),
        appdir: Some(link),
        exe: Some(real.join("usr/bin/fauste-player")),
        ..Launcher::default()
    };
    assert_eq!(
        launcher.plan().unwrap().program,
        OsString::from("/home/op/Apps/Fauste_Player.AppImage")
    );
}
