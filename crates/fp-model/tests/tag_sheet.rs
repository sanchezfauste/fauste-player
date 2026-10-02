#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O23: the tag sheet, its fields and its pure rules.

use std::collections::BTreeSet;

use fp_model::{
    Config, FieldProblem, TagField, TagFieldKind, TagSheet, changed_fields, field_problem,
    invalid_fields, unstored_fields,
};

fn all_storable() -> TagSheet {
    TagSheet::new(TagField::ALL, 0, false)
}

fn lines(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).to_owned()).collect()
}

#[test]
fn there_are_exactly_the_fields_of_the_spec_in_order() {
    let labels: Vec<&str> = TagField::ALL.iter().map(|f| f.slug()).collect();
    assert_eq!(
        labels,
        [
            "title",
            "artist",
            "album",
            "album-artist",
            "date",
            "track-number",
            "disc-number",
            "genre",
            "composer",
            "comment",
            "subtitle",
            "grouping",
            "bpm",
            "initial-key",
            "mood",
            "isrc",
            "publisher",
            "catalog-number",
            "copyright",
            "original-artist",
            "original-album",
            "original-release-date",
            "lyricist",
            "conductor",
            "remixer",
            "arranger",
            "performer",
            "language",
            "encoded-by",
            "lyrics",
            "sort-title",
            "sort-artist",
            "sort-album",
            "sort-album-artist",
            "sort-composer",
            "artist-website",
        ]
    );
    let unique: BTreeSet<_> = TagField::ALL.iter().collect();
    assert_eq!(unique.len(), 36);
    let mut sorted = TagField::ALL;
    sorted.sort();
    assert_eq!(
        sorted,
        TagField::ALL,
        "the derived order is the shown order"
    );
}

#[test]
fn ten_fields_are_always_shown_and_the_first_ten() {
    let always: Vec<TagField> = TagField::ALL
        .into_iter()
        .filter(|f| f.always_shown())
        .collect();
    assert_eq!(always, TagField::ALL[..10]);
}

#[test]
fn field_kinds_follow_the_spec() {
    assert_eq!(TagField::Date.kind(), TagFieldKind::Date);
    assert_eq!(TagField::OriginalReleaseDate.kind(), TagFieldKind::Date);
    assert_eq!(TagField::TrackNumber.kind(), TagFieldKind::Pair);
    assert_eq!(TagField::DiscNumber.kind(), TagFieldKind::Pair);
    assert_eq!(TagField::Bpm.kind(), TagFieldKind::Whole);
    assert_eq!(TagField::Lyrics.kind(), TagFieldKind::LongText);
    assert_eq!(TagField::Comment.kind(), TagFieldKind::LongText);
    assert_eq!(TagField::Title.kind(), TagFieldKind::Text);
    assert!(TagField::Artist.is_multi_value());
    assert!(!TagField::Title.is_multi_value());
    assert!(!TagField::Lyrics.is_multi_value());
}

#[test]
fn values_are_tidied_when_set() {
    let mut s = all_storable();
    s.set_values(TagField::Artist, lines(&["  A  ", "", "B"]));
    assert_eq!(s.values(TagField::Artist), ["A", "B"]);
    s.set_values(TagField::Artist, lines(&["", "  "]));
    assert!(!s.has(TagField::Artist));
    s.set_text(TagField::Genre, "Pop\n\n Rock \n", true);
    assert_eq!(s.values(TagField::Genre), ["Pop", "Rock"]);
    assert_eq!(s.text(TagField::Genre), "Pop\nRock");
    s.set_text(TagField::Lyrics, "one\ntwo\n", false);
    assert_eq!(
        s.values(TagField::Lyrics),
        ["one\ntwo"],
        "one value, inner break kept"
    );
}

#[test]
fn a_number_and_its_total_are_one_field() {
    let mut s = all_storable();
    s.set_pair(TagField::TrackNumber, "03", " 12 ");
    assert_eq!(s.pair(TagField::TrackNumber), ("3".into(), "12".into()));
    s.set_pair(TagField::TrackNumber, "", "12");
    assert_eq!(s.pair(TagField::TrackNumber), (String::new(), "12".into()));
    s.set_pair(TagField::TrackNumber, "", "");
    assert!(!s.has(TagField::TrackNumber));
    assert_eq!(
        s.pair(TagField::TrackNumber),
        (String::new(), String::new())
    );
}

#[test]
fn bpm_loses_leading_zeros_but_keeps_other_text() {
    let mut s = all_storable();
    s.set_text(TagField::Bpm, "0128", false);
    assert_eq!(s.values(TagField::Bpm), ["128"]);
    s.set_text(TagField::Bpm, "fast", false);
    assert_eq!(s.values(TagField::Bpm), ["fast"]);
}

#[test]
fn the_editor_shows_the_always_shown_fields_and_the_ones_the_file_has() {
    let mut s = TagSheet::new(
        [
            TagField::Title,
            TagField::Bpm,
            TagField::Mood,
            TagField::Isrc,
        ],
        0,
        false,
    );
    s.set_text(TagField::Bpm, "120", false);
    let none = BTreeSet::new();
    let visible = s.visible_fields(&none);
    assert_eq!(
        visible[..10],
        TagField::ALL[..10],
        "even those it cannot store"
    );
    assert!(visible.contains(&TagField::Bpm), "the file has it");
    assert!(!visible.contains(&TagField::Mood), "not present, not added");
    let added = BTreeSet::from([TagField::Mood, TagField::Lyrics]);
    let visible = s.visible_fields(&added);
    assert!(visible.contains(&TagField::Mood));
    assert!(
        !visible.contains(&TagField::Lyrics),
        "an added field the format cannot store is not shown"
    );
}

#[test]
fn add_field_offers_only_what_the_format_can_store_and_is_missing() {
    let mut s = TagSheet::new(
        [
            TagField::Title,
            TagField::Bpm,
            TagField::Mood,
            TagField::Isrc,
        ],
        0,
        false,
    );
    s.set_text(TagField::Bpm, "120", false);
    let mut added = BTreeSet::new();
    assert_eq!(
        s.addable_fields(&added),
        [TagField::Mood, TagField::Isrc],
        "Title is always shown, Bpm is present, the rest cannot be stored"
    );
    added.insert(TagField::Mood);
    assert_eq!(s.addable_fields(&added), [TagField::Isrc]);
}

#[test]
fn text_is_cut_and_the_values_limited() {
    let mut s = all_storable();
    s.set_values(TagField::Artist, lines(&["abcdefghij", "b", "c", "d"]));
    s.set_pair(TagField::TrackNumber, "1", "99999");
    let s = s.clamped(4, 2);
    assert_eq!(s.values(TagField::Artist), ["abcd", "b"]);
    assert_eq!(
        s.pair(TagField::TrackNumber),
        ("1".into(), "9999".into()),
        "a pair keeps both its entries"
    );
}

#[test]
fn the_changed_fields_come_in_editor_order() {
    let before = all_storable();
    let mut after = before.clone();
    after.set_text(TagField::Genre, "Jazz", true);
    after.set_text(TagField::Title, "New", false);
    assert_eq!(
        changed_fields(&before, &after),
        [TagField::Title, TagField::Genre]
    );
    assert!(changed_fields(&before, &before).is_empty());
}

#[test]
fn the_values_of_a_changed_field_are_validated() {
    let before = all_storable();
    let invalid = |f: &dyn Fn(&mut TagSheet)| {
        let mut after = before.clone();
        f(&mut after);
        invalid_fields(&before, &after)
    };
    assert_eq!(
        invalid(&|s| s.set_text(TagField::Date, "2019-13", false)),
        [TagField::Date]
    );
    assert!(invalid(&|s| s.set_text(TagField::Date, "2019-05-14T10:30", false)).is_empty());
    assert_eq!(
        invalid(&|s| s.set_text(TagField::OriginalReleaseDate, "May 1999", false)),
        [TagField::OriginalReleaseDate]
    );
    assert_eq!(
        invalid(&|s| s.set_pair(TagField::TrackNumber, "3a", "")),
        [TagField::TrackNumber]
    );
    assert_eq!(
        invalid(&|s| s.set_pair(TagField::DiscNumber, "1", "-2")),
        [TagField::DiscNumber]
    );
    assert_eq!(
        invalid(&|s| s.set_pair(TagField::TrackNumber, "", "12")),
        [TagField::TrackNumber],
        "a total needs a number (the file would get track 0)"
    );
    assert!(invalid(&|s| s.set_pair(TagField::TrackNumber, "3", "12")).is_empty());
    assert_eq!(
        invalid(&|s| s.set_text(TagField::Bpm, "120.5", false)),
        [TagField::Bpm]
    );
    assert_eq!(
        invalid(&|s| s.set_text(TagField::Bpm, "99999999999", false)),
        [TagField::Bpm],
        "beyond u32"
    );
    assert!(invalid(&|s| s.set_text(TagField::Bpm, "128", false)).is_empty());
    assert_eq!(
        invalid(&|s| s.set_text(TagField::Date, "1999\n2000", true)),
        [TagField::Date],
        "one value only"
    );
}

#[test]
fn clearing_a_field_is_always_valid() {
    let mut before = all_storable();
    before.set_text(TagField::Date, "1999", false);
    before.set_pair(TagField::TrackNumber, "3", "12");
    let mut after = before.clone();
    after.set_text(TagField::Date, "", false);
    after.set_pair(TagField::TrackNumber, "", "");
    assert!(invalid_fields(&before, &after).is_empty());
}

#[test]
fn a_value_the_operator_did_not_touch_never_blocks_the_save() {
    // The file holds a track number that is not a number; leaving it alone
    // keeps it as it is.
    let mut before = all_storable();
    before.set_pair(TagField::TrackNumber, "A1", "");
    let mut after = before.clone();
    after.set_text(TagField::Genre, "Jazz", true);
    assert!(invalid_fields(&before, &after).is_empty());
    after.set_pair(TagField::TrackNumber, "A2", "");
    assert_eq!(invalid_fields(&before, &after), [TagField::TrackNumber]);
}

#[test]
fn a_change_to_a_field_the_format_cannot_store_is_invalid() {
    let before = TagSheet::new([TagField::Title], 0, false);
    let mut after = before.clone();
    after.set_text(TagField::AlbumArtist, "X", true);
    assert_eq!(invalid_fields(&before, &after), [TagField::AlbumArtist]);
}

#[test]
fn unstored_fields_are_the_changes_the_file_did_not_keep() {
    let before = all_storable();
    let mut after = before.clone();
    after.set_text(TagField::Genre, "Jazz", true);
    after.set_text(TagField::Mood, "Calm", true);
    let mut read_back = before.clone();
    read_back.set_text(TagField::Genre, "Jazz", true);
    assert_eq!(
        unstored_fields(&before, &after, &read_back),
        [TagField::Mood]
    );
    let mut other = read_back.clone();
    other.set_text(TagField::Title, "Changed by someone else", false);
    assert_eq!(
        unstored_fields(&before, &after, &other),
        [TagField::Mood],
        "only the fields the operator changed are judged"
    );
}

#[test]
fn the_value_cap_is_a_validated_config_field() {
    let mut c = Config::default();
    assert_eq!(c.limits.max_tag_values, 32);
    c.limits.max_tag_values = 0;
    let warnings = c.validate();
    assert_eq!(c.limits.max_tag_values, 1);
    assert!(warnings.iter().any(|w| w.field == "limits.max_tag_values"));
    c.limits.max_tag_values = 5_000;
    c.validate();
    assert_eq!(c.limits.max_tag_values, 1000);
}

#[test]
fn a_field_cut_when_read_is_marked_and_never_written() {
    let mut s = all_storable();
    s.set_text(TagField::Comment, &"x".repeat(100), false);
    s.set_values(TagField::Artist, lines(&["a", "b", "c"]));
    s.set_text(TagField::Title, "short", false);
    let before = s.clamped(10, 2);
    assert!(before.is_cut(TagField::Comment));
    assert!(before.is_cut(TagField::Artist));
    assert!(!before.is_cut(TagField::Title));

    let mut after = before.clone();
    after.set_text(TagField::Comment, "new comment", false);
    assert_eq!(
        invalid_fields(&before, &after),
        [TagField::Comment],
        "a cut field cannot be changed"
    );
    assert_eq!(
        field_problem(&before, &after, TagField::Comment),
        Some(FieldProblem::TooLongToEdit)
    );
    let mut after = before.clone();
    after.set_text(TagField::Title, "other", false);
    assert!(invalid_fields(&before, &after).is_empty());
    assert_eq!(field_problem(&before, &after, TagField::Title), None);
    let mut after = before.clone();
    after.set_text(TagField::Date, "2019-13", false);
    assert_eq!(
        field_problem(&before, &after, TagField::Date),
        Some(FieldProblem::InvalidValue)
    );
}
