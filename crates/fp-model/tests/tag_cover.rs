#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O23: the front cover in the tag sheet and its pure rules.

use fp_model::{CoverArt, TagField, TagSheet, cover_blocked, cover_changed, cover_unstored};

fn sheet(cover: Option<CoverArt>) -> TagSheet {
    let mut s = TagSheet::new(TagField::ALL, 0, false).with_cover_support(true);
    s.set_cover(cover);
    s
}

fn art(byte: u8, front: bool) -> CoverArt {
    CoverArt::new(vec![byte; 8], front)
}

#[test]
fn a_new_sheet_has_no_cover_and_cannot_store_one() {
    let s = TagSheet::new(TagField::ALL, 0, false);
    assert!(s.cover().is_none());
    assert!(!s.can_store_cover());
    assert!(TagSheet::default().cover().is_none());
    assert!(s.with_cover_support(true).can_store_cover());
}

#[test]
fn two_covers_are_equal_when_their_bytes_and_kind_are() {
    assert_eq!(art(1, true), art(1, true));
    assert_ne!(art(1, true), art(2, true));
    assert_ne!(
        art(1, true),
        art(1, false),
        "a back cover is another picture"
    );
    let with_thumb = art(1, true).with_thumb(Some(vec![9, 9]));
    assert_eq!(with_thumb, art(1, true), "the thumbnail is not part of it");
    assert_eq!(with_thumb.thumb_png(), Some(&[9u8, 9][..]));
    assert_eq!(art(1, true).thumb_png(), None);
    assert_eq!(art(7, true).data(), [7u8; 8]);
}

#[test]
fn replacing_or_removing_the_cover_is_a_change() {
    let before = sheet(Some(art(1, true)));
    let mut after = before.clone();
    assert!(!cover_changed(&before, &after));
    after.set_cover(Some(art(2, true)));
    assert!(cover_changed(&before, &after));
    after.remove_front_cover(&before);
    assert!(cover_changed(&before, &after));
    assert!(after.cover().is_none());
    let none = sheet(None);
    let mut added = none.clone();
    added.set_cover(Some(art(3, true)));
    assert!(cover_changed(&none, &added));
    let mut removed = added.clone();
    removed.remove_front_cover(&none);
    assert!(!cover_changed(&none, &removed), "back to what the file had");
}

#[test]
fn a_picture_that_is_only_shown_is_never_removed() {
    // The file has a back cover only: the editor shows it, Remove leaves it.
    let before = sheet(Some(art(5, false)));
    let mut after = before.clone();
    after.set_cover(Some(art(6, true)));
    assert!(cover_changed(&before, &after));
    after.remove_front_cover(&before);
    assert_eq!(after.cover(), before.cover());
    assert!(!cover_changed(&before, &after));
}

#[test]
fn a_cover_change_on_a_format_without_pictures_is_blocked() {
    let before = TagSheet::new(TagField::ALL, 0, false);
    let mut after = before.clone();
    assert!(
        !cover_blocked(&before, &after),
        "no change, nothing to block"
    );
    after.set_cover(Some(art(1, true)));
    assert!(cover_blocked(&before, &after));
    let storable = before.with_cover_support(true);
    let mut after = storable.clone();
    after.set_cover(Some(art(1, true)));
    assert!(!cover_blocked(&storable, &after));
}

#[test]
fn the_cover_the_file_did_not_keep_is_named() {
    let before = sheet(Some(art(1, true)));
    let mut after = before.clone();
    after.set_cover(Some(art(2, true)));
    assert!(!cover_unstored(&before, &after, &after.clone()));
    assert!(cover_unstored(&before, &after, &before));
    assert!(
        !cover_unstored(&before, &before, &sheet(None)),
        "an untouched cover is not judged"
    );
}

#[test]
fn the_cover_survives_clamping_and_cloning_cheaply() {
    let mut s = sheet(Some(art(1, true).with_thumb(Some(vec![1]))));
    s.set_text(TagField::Title, &"x".repeat(500), false);
    let clamped = s.clone().clamped(64, 2);
    assert_eq!(clamped.cover(), s.cover());
    assert_eq!(clamped.cover().unwrap().thumb_png(), Some(&[1u8][..]));
    assert!(clamped.can_store_cover());
}

#[test]
fn a_removed_front_cover_is_unstored_only_if_a_front_cover_is_still_there() {
    // The file has a back cover only: after Remove (nothing staged) the
    // file shows that back cover again, which is not a failure to remove.
    let front = sheet(Some(art(1, true)));
    let mut after = front.clone();
    after.remove_front_cover(&front);
    let back_only = sheet(Some(art(5, false)));
    assert!(!cover_unstored(&front, &after, &back_only));
    // A front cover still in the file after the removal is unstored.
    assert!(cover_unstored(&front, &after, &front));
    // No picture at all is the expected result.
    assert!(!cover_unstored(&front, &after, &sheet(None)));
}
