#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Command-line arguments (Phase 5): what a package, a file manager or a
//! smoke test passes to the binary.

use std::ffi::OsString;
use std::path::PathBuf;

use fp_app::cli::{Invocation, parse};

fn args(list: &[&str]) -> Vec<OsString> {
    list.iter().map(OsString::from).collect()
}

#[test]
fn version_and_help_are_recognised() {
    assert_eq!(parse(args(&["--version"])).unwrap(), Invocation::Version);
    assert_eq!(parse(args(&["-V"])).unwrap(), Invocation::Version);
    assert_eq!(parse(args(&["--help"])).unwrap(), Invocation::Help);
    assert_eq!(parse(args(&["-h"])).unwrap(), Invocation::Help);
}

#[test]
fn no_arguments_just_run() {
    assert_eq!(
        parse(args(&[])).unwrap(),
        Invocation::Run {
            playlists: vec![],
            ignored: vec![]
        }
    );
}

#[test]
fn playlist_files_are_imported_and_other_files_ignored() {
    let parsed = parse(args(&["Morning.M3U", "/x/b.pls", "song.mp3", "c.m3u8"])).unwrap();
    assert_eq!(
        parsed,
        Invocation::Run {
            playlists: vec![
                PathBuf::from("Morning.M3U"),
                PathBuf::from("/x/b.pls"),
                PathBuf::from("c.m3u8")
            ],
            ignored: vec![PathBuf::from("song.mp3")],
        }
    );
}

#[test]
fn unknown_options_are_refused_but_paths_after_a_double_dash_are_not() {
    assert!(parse(args(&["--frobnicate"])).is_err());
    assert_eq!(
        parse(args(&["--", "-odd name.m3u"])).unwrap(),
        Invocation::Run {
            playlists: vec![PathBuf::from("-odd name.m3u")],
            ignored: vec![]
        }
    );
}

#[test]
fn the_macos_process_serial_number_is_ignored() {
    assert_eq!(
        parse(args(&["-psn_0_12345"])).unwrap(),
        Invocation::Run {
            playlists: vec![],
            ignored: vec![]
        }
    );
}

#[test]
fn the_window_icon_is_a_valid_square_image() {
    let icon = fp_app::cli::window_icon().expect("the embedded icon decodes");
    assert_eq!(icon.width, icon.height);
    assert!(icon.width >= 128);
}

#[test]
fn the_window_title_is_the_name_and_the_version() {
    assert_eq!(
        fp_app::cli::window_title(),
        format!("Fauste Player {}", fp_app::ui::about::VERSION)
    );
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn console_is_the_default_off_windows() {
    // Only a Windows release build is a GUI program (feedback 2 spec O36);
    // the test suite itself always runs with a console.
    assert!(fp_app::cli::CONSOLE);
}

#[test]
fn emit_prints_on_a_console_build() {
    // Must not panic or open a box where a console exists.
    fp_app::cli::emit(fp_app::cli::Stream::Out, "");
    fp_app::cli::emit(fp_app::cli::Stream::Err, "");
}
