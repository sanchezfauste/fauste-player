#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! External editor spec EE2-EE5: when "Open in external editor" is blocked.

use crate::common::fixture;
use fp_model::{ExternalEditBlock, FileState, TrackId, external_edit_block};
use std::path::PathBuf;

fn with_editor(s: &mut fp_model::AppState) {
    s.config.library.external_editor = Some(PathBuf::from("/usr/bin/editor"));
}

fn first_track(s: &fp_model::AppState) -> TrackId {
    let p = s.playlists.iter().next().unwrap();
    p.entries[0].track
}

#[test]
fn no_editor_blocks_first() {
    let mut s = fixture(2);
    let t = first_track(&s);
    s.library.get_mut(t).unwrap().file_state = FileState::Missing;
    assert_eq!(
        external_edit_block(&s, t),
        Some(ExternalEditBlock::NoEditor)
    );
}

#[test]
fn a_missing_file_blocks() {
    let mut s = fixture(2);
    with_editor(&mut s);
    let t = first_track(&s);
    s.library.get_mut(t).unwrap().file_state = FileState::Unreadable;
    assert_eq!(
        external_edit_block(&s, t),
        Some(ExternalEditBlock::FileUnavailable)
    );
    s.library.get_mut(t).unwrap().file_state = FileState::Missing;
    assert_eq!(
        external_edit_block(&s, t),
        Some(ExternalEditBlock::FileUnavailable)
    );
}

#[test]
fn a_sounding_track_blocks() {
    let mut s = fixture(2);
    with_editor(&mut s);
    let t = first_track(&s);
    let entry = s.playlists.iter().next().unwrap().entries[0].id;
    let player = s.players.first_mut().unwrap();
    player.current = Some(entry);
    assert_eq!(
        external_edit_block(&s, t),
        Some(ExternalEditBlock::Sounding)
    );
}

#[test]
fn a_quiet_track_with_an_editor_is_allowed() {
    let mut s = fixture(2);
    with_editor(&mut s);
    let t = first_track(&s);
    assert_eq!(external_edit_block(&s, t), None);
}
