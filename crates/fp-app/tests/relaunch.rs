#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O4: how the application starts itself again.

use std::ffi::OsString;
use std::path::PathBuf;

use fp_app::restart::{Launcher, Relaunch, spawn};

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
