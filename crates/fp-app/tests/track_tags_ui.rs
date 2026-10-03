#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O23: the row tooltip and the tag editor.

mod support;

use std::path::{Path, PathBuf};

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_analysis::tags::read_tag_sheet;
use fp_app::ui::controller::Controller;
use fp_model::{
    AppState, AudioFormat, Command, FileState, Limits, TagEditBlock, TagField, TagSheet,
};
use lofty::config::WriteOptions;
use lofty::picture::PictureType;
use lofty::prelude::*;
use lofty::tag::{ItemKey, Tag, TagType};
use support::{harness, state};

type Ui = egui_kittest::Harness<'static, fp_app::ui::app::AppUi>;

/// `state(1, 3)` whose first track has tags and a format.
fn tagged_state() -> AppState {
    let mut s = state(1, 3);
    let first = s.library.iter().next().unwrap().id;
    let t = s.library.get_mut(first).unwrap();
    t.title = "Song 1".into();
    t.artist = "The Artist".into();
    t.album = "An Album".into();
    t.date = Some("1999-03-07".into());
    t.genre = "Pop".into();
    t.duration_secs = 200.0;
    t.analyzed = true;
    t.tags_read = true;
    t.file_state = FileState::Ok;
    t.format = Some(AudioFormat {
        sample_rate: 44_100,
        bits: Some(16),
        channels: 2,
        dsd_rate: None,
    });
    s
}

#[test]
fn hovering_a_row_shows_the_tags_format_and_path() {
    let (mut h, _fake) = harness(tagged_state());
    h.get_all_by_label("Song 1").last().unwrap().hover();
    h.run_steps(40);
    // The field labels exist only in the tooltip.
    for label in ["Title", "Format", "Path", "Genre"] {
        assert!(
            h.query_by_label(label).is_some(),
            "the tooltip labels {label}"
        );
    }
    for text in [
        "The Artist",
        "An Album",
        "1999-03-07",
        "Pop",
        "03:20",
        "MP3 · 44.1 kHz · 16 bit",
        "/music/Song 1.mp3",
    ] {
        // The row shows some of these too, so more than one node can match.
        assert!(
            h.query_all_by_label_contains(text).next().is_some(),
            "the tooltip shows {text}"
        );
    }
}

#[test]
fn the_tooltip_waits_for_the_usual_delay() {
    let (mut h, _fake) = harness(tagged_state());
    h.get_all_by_label("Song 1").last().unwrap().hover();
    h.run_steps(2);
    assert!(h.query_by_label_contains("An Album").is_none());
}

// ---------------------------------------------------------------------------
// The editor.
// ---------------------------------------------------------------------------

fn wav(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for _ in 0..800 {
        w.write_sample(0i16).unwrap();
    }
    w.finalize().unwrap();
    path
}

/// A WAV tagged as a tagging tool would: an ID3v2 tag (lofty's primary tag
/// for WAV) with two artists, a track number and total, a BPM and a title.
fn tagged_wav(dir: &Path, name: &str) -> PathBuf {
    let path = wav(dir, name);
    let mut file = lofty::read_from_path(&path).unwrap();
    let ty = file.primary_tag_type();
    file.insert_tag(Tag::new(ty));
    let tag = file.primary_tag_mut().unwrap();
    tag.set_title("Song 1".into());
    tag.set_genre("Pop".into());
    tag.push(lofty::tag::TagItem::new(
        ItemKey::TrackArtist,
        lofty::tag::ItemValue::Text("First Artist".into()),
    ));
    tag.push(lofty::tag::TagItem::new(
        ItemKey::TrackArtist,
        lofty::tag::ItemValue::Text("Second Artist".into()),
    ));
    tag.insert_text(ItemKey::TrackNumber, "3".into());
    tag.insert_text(ItemKey::TrackTotal, "12".into());
    tag.insert_text(ItemKey::IntegerBpm, "128".into());
    file.save_to_path(&path, WriteOptions::default()).unwrap();
    path
}

/// A WAV whose tag is RIFF INFO, a format that cannot store an album artist.
fn riff_wav(dir: &Path, name: &str) -> PathBuf {
    let path = wav(dir, name);
    let mut file = lofty::read_from_path(&path).unwrap();
    file.insert_tag(Tag::new(TagType::RiffInfo));
    file.tag_mut(TagType::RiffInfo)
        .unwrap()
        .set_title("Song 1".into());
    file.save_to_path(&path, WriteOptions::default()).unwrap();
    path
}

/// `tagged_state` whose first track is the file at `path`.
fn state_at(path: &Path) -> AppState {
    let mut s = tagged_state();
    let first = s.library.iter().next().unwrap().id;
    s.library.get_mut(first).unwrap().path = path.to_path_buf();
    s
}

/// Right-clicks the table row of "Song 1": the lowest node with that text,
/// as the player header and a CUE window show it higher up.
fn right_click_the_row(h: &Ui) {
    h.get_all_by_label("Song 1")
        .max_by(|a, b| a.rect().min.y.total_cmp(&b.rect().min.y))
        .unwrap()
        .click_secondary();
}

fn open_editor(h: &mut Ui) {
    right_click_the_row(h);
    h.run_steps(3);
    h.get_by_label("Edit tags…").click();
    h.run_steps(2);
}

/// Opens the editor and waits until its sheet has arrived and the modal,
/// which is centred and sizes itself over a few frames, has settled (a click
/// aimed at an earlier frame's layout would miss).
fn open_ready(h: &mut Ui) {
    open_editor(h);
    wait_for_text(h, "Add field");
    h.run_steps(10);
}

fn wait_for(h: &mut Ui, what: &str, done: impl Fn(&mut Ui) -> bool) {
    for _ in 0..300 {
        h.run_steps(1);
        if done(h) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("never happened: {what}");
}

fn wait_for_text(h: &mut Ui, text: &str) {
    wait_for(h, text, |h| {
        h.query_all_by_label_contains(text).next().is_some()
    });
}

/// The text boxes labelled `label`: one line or several.
fn inputs<'a>(h: &'a Ui, label: &'a str) -> impl Iterator<Item = egui_kittest::Node<'a>> + 'a {
    h.query_all_by_role_and_label(Role::TextInput, label)
        .chain(h.query_all_by_role_and_label(Role::MultilineTextInput, label))
}

fn input_value(h: &Ui, label: &str, nth: usize) -> String {
    inputs(h, label)
        .nth(nth)
        .unwrap()
        .accesskit_node()
        .value()
        .unwrap_or_default()
        .to_owned()
}

fn field(h: &Ui, label: &str) -> String {
    input_value(h, label, 0)
}

fn type_into(h: &mut Ui, label: &str, nth: usize, text: &str) {
    inputs(h, label).nth(nth).unwrap().focus();
    h.run_steps(2);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run_steps(1);
    if text.is_empty() {
        h.key_press(egui::Key::Delete);
        h.run_steps(1);
    }
    for c in text.chars() {
        inputs(h, label).nth(nth).unwrap().type_text(&c.to_string());
        h.run_steps(1);
    }
}

fn save_enabled(h: &Ui) -> bool {
    !h.get_by_role_and_label(Role::Button, "Save")
        .accesskit_node()
        .is_disabled()
}

fn sheet_of(path: &Path) -> TagSheet {
    read_tag_sheet(path, &Limits::default()).unwrap()
}

fn put_on_a_playing_cart(state: &mut AppState, track: fp_model::TrackId) {
    let cart = state.cartwall.pages[0].carts[0].id;
    state.cartwall.pages[0].carts[0].track = Some(track);
    state.cartwall.playing.push(fp_model::PlayingCart {
        cart,
        looped: false,
    });
}

#[test]
fn the_editor_says_it_is_reading_until_the_sheet_arrives() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, _fake) = harness(state_at(&tagged_wav(dir.path(), "a.wav")));
    open_editor(&mut h);
    // The sheet is requested on the frame that opens the editor and handled
    // on the next one, so this frame still waits for it.
    assert!(h.query_by_label("Reading tags…").is_some());
    assert!(!save_enabled(&h));
    wait_for_text(&mut h, "Add field");
    assert!(h.query_by_label("Reading tags…").is_none());
}

#[test]
fn the_editor_shows_what_the_file_holds() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, _fake) = harness(state_at(&tagged_wav(dir.path(), "a.wav")));
    open_ready(&mut h);
    assert!(h.query_by_label("Edit tags").is_some(), "the modal title");
    assert_eq!(field(&h, "Title"), "Song 1");
    assert_eq!(
        field(&h, "Artist"),
        "First Artist\nSecond Artist",
        "one value per line"
    );
    assert_eq!(field(&h, "Genre"), "Pop");
    assert_eq!(field(&h, "Track number"), "3");
    assert_eq!(field(&h, "Total"), "12", "the track total");
    assert_eq!(field(&h, "BPM"), "128", "an optional field the file has");
    for always in [
        "Album",
        "Album artist",
        "Date",
        "Disc number",
        "Composer",
        "Comment",
    ] {
        assert!(
            inputs(&h, always).next().is_some(),
            "{always} is always shown"
        );
    }
    assert!(
        inputs(&h, "Mood").next().is_none(),
        "an optional field the file lacks is not shown"
    );
}

#[test]
fn add_field_offers_what_the_format_can_store_and_shows_the_choice() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, _fake) = harness(state_at(&tagged_wav(dir.path(), "a.wav")));
    open_ready(&mut h);
    h.get_by_label("Add field").click();
    h.run_steps(3);
    let offered = |h: &Ui, name: &str| h.query_by_role_and_label(Role::Button, name).is_some();
    assert!(offered(&h, "Mood"));
    assert!(offered(&h, "Lyrics"));
    assert!(!offered(&h, "Performer"), "ID3v2 has no performer frame");
    assert!(!offered(&h, "BPM"), "the file already has the BPM");
    h.get_by_role_and_label(Role::Button, "Mood").click();
    h.run_steps(3);
    assert!(inputs(&h, "Mood").next().is_some());
}

#[test]
fn a_format_that_cannot_store_a_field_shows_it_disabled_and_never_offers_it() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, _fake) = harness(state_at(&riff_wav(dir.path(), "a.wav")));
    open_ready(&mut h);
    assert!(
        inputs(&h, "Album artist")
            .next()
            .unwrap()
            .accesskit_node()
            .is_disabled()
    );
    assert_eq!(
        h.query_all_by_label_contains("cannot store this field")
            .count(),
        2,
        "the album artist and the disc number"
    );
    h.get_by_label("Add field").click();
    h.run_steps(3);
    assert!(
        h.query_by_role_and_label(Role::Button, "Copyright")
            .is_some()
    );
    for missing in ["Mood", "Lyrics", "Publisher", "Album artist"] {
        assert!(
            h.query_by_role_and_label(Role::Button, missing).is_none(),
            "{missing} is not offered"
        );
    }
}

#[test]
fn an_unreadable_file_cannot_be_edited() {
    let (mut h, _fake) = harness(state_at(Path::new("/nonexistent/folder/a.wav")));
    open_editor(&mut h);
    wait_for_text(&mut h, "cannot be read, so they cannot be edited");
    assert!(!save_enabled(&h));
    h.get_by_role_and_label(Role::Button, "Cancel").click();
    h.run_steps(3);
    assert!(h.query_by_label("Edit tags").is_none());
}

#[test]
fn the_menu_item_is_disabled_with_the_reason() {
    let reasons = [
        (
            TagEditBlock::FileUnavailable,
            "The file is missing or cannot be read",
        ),
        (
            TagEditBlock::UnsupportedFormat,
            "This format has no writable tags",
        ),
        (TagEditBlock::TagsNotRead, "The tags have not been read yet"),
        (
            TagEditBlock::OnAir,
            "A track that is on air cannot be edited",
        ),
        (
            TagEditBlock::Cued,
            "A track that is on CUE cannot be edited",
        ),
        (
            TagEditBlock::OnCart,
            "A track on a playing cart cannot be edited",
        ),
    ];
    for (block, text) in reasons {
        let mut s = tagged_state();
        let first = s.library.iter().next().unwrap().id;
        let p = s.players[0].id;
        let e = s
            .playlists
            .get(s.playlists.first_id().unwrap())
            .unwrap()
            .entries[0]
            .id;
        match block {
            TagEditBlock::FileUnavailable => {
                s.library.get_mut(first).unwrap().file_state = FileState::Missing;
            }
            TagEditBlock::UnsupportedFormat => {
                s.library.get_mut(first).unwrap().path = PathBuf::from("/music/Song 1.dsf");
            }
            TagEditBlock::TagsNotRead => s.library.get_mut(first).unwrap().tags_read = false,
            TagEditBlock::OnAir => {
                fp_model::apply(&mut s, Command::SetNext(p, e)).unwrap();
                fp_model::apply(&mut s, Command::Play(p)).unwrap();
            }
            TagEditBlock::Cued => {
                fp_model::apply(&mut s, Command::CueEntry(p, e)).unwrap();
            }
            TagEditBlock::OnCart => {
                put_on_a_playing_cart(&mut s, first);
                s.cartwall.pages[0].carts[0].name = "Jingle".into();
            }
        }
        let (mut h, _fake) = harness(s);
        right_click_the_row(&h);
        h.run_steps(3);
        let item = h
            .query_by_label("Edit tags…")
            .unwrap_or_else(|| panic!("no menu item for {block:?}"));
        assert!(item.accesskit_node().is_disabled(), "{block:?}");
        item.hover();
        h.run_steps(40);
        assert!(
            h.query_by_label_contains(text).is_some(),
            "{block:?}: {text}"
        );
    }
}

#[test]
fn saving_writes_the_file_and_the_library_takes_the_tags() {
    let dir = tempfile::tempdir().unwrap();
    let path = tagged_wav(dir.path(), "a.wav");
    let (mut h, fake) = harness(state_at(&path));
    open_ready(&mut h);
    type_into(&mut h, "Genre", 0, "Jazz");
    type_into(&mut h, "Total", 0, "14");
    assert!(save_enabled(&h));
    h.get_by_role_and_label(Role::Button, "Save").click();
    wait_for(&mut h, "the library takes the genre", |_| {
        fake.state.load().library.iter().next().unwrap().genre == "Jazz"
    });
    let sheet = sheet_of(&path);
    assert_eq!(sheet.values(TagField::Genre), ["Jazz"]);
    assert_eq!(sheet.pair(TagField::TrackNumber), ("3".into(), "14".into()));
    assert_eq!(
        sheet.values(TagField::Artist),
        ["First Artist", "Second Artist"],
        "untouched fields stay"
    );
    let track = fake.state.load().library.iter().next().unwrap().clone();
    assert_eq!(track.date, None, "the file has no date: the read-back wins");
    assert!(track.tags_read);
    assert_eq!(track.duration_secs, 200.0, "analysis data is kept");
    h.run_steps(3);
    assert!(h.query_by_label("Edit tags").is_none(), "the modal closed");
}

#[test]
fn an_added_field_is_saved_and_an_empty_one_is_not() {
    let dir = tempfile::tempdir().unwrap();
    let path = tagged_wav(dir.path(), "a.wav");
    let (mut h, _fake) = harness(state_at(&path));
    open_ready(&mut h);
    for name in ["Mood", "ISRC"] {
        h.get_by_label("Add field").click();
        h.run_steps(3);
        h.get_by_role_and_label(Role::Button, name).click();
        h.run_steps(3);
    }
    assert!(!save_enabled(&h), "an empty added field changes nothing");
    type_into(&mut h, "Mood", 0, "Calm");
    h.get_by_role_and_label(Role::Button, "Save").click();
    wait_for(&mut h, "the file has the mood", |_| {
        sheet_of(&path).values(TagField::Mood) == ["Calm"]
    });
    assert!(!sheet_of(&path).has(TagField::Isrc));
}

#[test]
fn a_failed_save_keeps_the_modal_and_says_why() {
    let dir = tempfile::tempdir().unwrap();
    let path = tagged_wav(dir.path(), "a.wav");
    let (mut h, fake) = harness(state_at(&path));
    open_ready(&mut h);
    type_into(&mut h, "Genre", 0, "Jazz");
    // The file vanishes after the sheet was read.
    std::fs::remove_file(&path).unwrap();
    h.get_by_role_and_label(Role::Button, "Save").click();
    wait_for_text(&mut h, "were not saved");
    assert!(h.query_by_label("Edit tags").is_some(), "the modal stays");
    assert_eq!(field(&h, "Genre"), "Jazz", "the text is kept");
    assert!(
        h.query_all_by_label_contains("the file was not found")
            .count()
            >= 1,
        "the reason is shown"
    );
    assert_eq!(
        fake.state.load().library.iter().next().unwrap().genre,
        "Pop",
        "the library is unchanged"
    );
    assert!(save_enabled(&h), "the operator can try again");
}

#[test]
fn save_needs_a_change_and_valid_values() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, _fake) = harness(state_at(&tagged_wav(dir.path(), "a.wav")));
    open_ready(&mut h);
    assert!(!save_enabled(&h), "nothing changed");
    type_into(&mut h, "Date", 0, "2019-13");
    assert!(!save_enabled(&h));
    assert!(
        h.query_by_label_contains("Use YYYY, YYYY-MM or YYYY-MM-DD")
            .is_some()
    );
    type_into(&mut h, "Date", 0, "2019-05-14");
    assert!(save_enabled(&h), "a valid date");
    type_into(&mut h, "BPM", 0, "fast");
    assert!(!save_enabled(&h));
    assert!(h.query_by_label_contains("Use whole numbers").is_some());
    type_into(&mut h, "BPM", 0, "130");
    assert!(save_enabled(&h));
    type_into(&mut h, "Total", 0, "x");
    assert!(!save_enabled(&h), "a total that is not a number");
    type_into(&mut h, "Total", 0, "12");
    type_into(&mut h, "Track number", 0, "");
    assert!(!save_enabled(&h), "a total needs a number");
}

#[test]
fn cancel_closes_the_editor_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = tagged_wav(dir.path(), "a.wav");
    let before = std::fs::read(&path).unwrap();
    let (mut h, fake) = harness(state_at(&path));
    open_ready(&mut h);
    type_into(&mut h, "Genre", 0, "Jazz");
    h.get_by_role_and_label(Role::Button, "Cancel").click();
    h.run_steps(3);
    assert!(h.query_by_label("Edit tags").is_none());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(
        fake.take_sent()
            .iter()
            .all(|c| !matches!(c, Command::ApplyTags { .. }))
    );
}

#[test]
fn saving_is_refused_when_the_track_went_on_air() {
    let dir = tempfile::tempdir().unwrap();
    let path = tagged_wav(dir.path(), "a.wav");
    let before = std::fs::read(&path).unwrap();
    let (mut h, fake) = harness(state_at(&path));
    open_ready(&mut h);
    type_into(&mut h, "Genre", 0, "Jazz");
    // The operator (or a remote client) starts the track meanwhile.
    let p = fake.player(0);
    let e = fake.entries()[0];
    fake.send(Command::SetNext(p, e));
    fake.send(Command::Play(p));
    h.run_steps(3);
    assert!(!save_enabled(&h), "Save is off while the track is on air");
    assert!(
        h.query_by_label_contains("A track that is on air cannot be edited")
            .is_some()
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn the_modal_closes_when_the_track_is_removed() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, fake) = harness(state_at(&tagged_wav(dir.path(), "a.wav")));
    open_ready(&mut h);
    // The library loses the track (the tests' controller has no command for
    // it, so the snapshot is edited directly).
    let mut s = (**fake.state.load()).clone();
    let ids: Vec<_> = s.library.iter().map(|t| t.id).collect();
    for id in ids {
        s.library.remove(id);
    }
    fake.state.store(std::sync::Arc::new(s));
    h.run_steps(3);
    assert!(h.query_by_label("Edit tags").is_none());
}

#[test]
fn no_shortcut_acts_under_the_editor() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, fake) = harness(state_at(&tagged_wav(dir.path(), "a.wav")));
    open_ready(&mut h);
    fake.take_sent();
    h.key_press(egui::Key::Delete);
    h.key_press(egui::Key::Space);
    h.run_steps(3);
    assert!(
        fake.take_sent().is_empty(),
        "Delete must not remove the entry"
    );
}

#[test]
fn every_field_has_a_label_in_both_languages() {
    for lang in ["en-US", "es-ES"] {
        let i18n = fp_app::i18n::I18n::new(Some(lang));
        for field in TagField::ALL {
            let key = format!("tag-field-{}", field.slug());
            assert_ne!(i18n.tr(&key), key, "{lang}: {key}");
        }
    }
}

// ---------------------------------------------------------------------------
// The cover.
// ---------------------------------------------------------------------------

fn png(width: u32, color: [u8; 3]) -> Vec<u8> {
    let img = image::RgbImage::from_pixel(width, 30, image::Rgb(color));
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

fn red() -> Vec<u8> {
    png(40, [200, 30, 30])
}

fn blue() -> Vec<u8> {
    png(50, [30, 30, 200])
}

/// `tagged_wav` with these pictures in its ID3v2 tag.
fn wav_with_pictures(dir: &Path, pictures: &[(lofty::picture::PictureType, Vec<u8>)]) -> PathBuf {
    let path = tagged_wav(dir, "a.wav");
    let mut file = lofty::read_from_path(&path).unwrap();
    let tag = file.primary_tag_mut().unwrap();
    for (kind, data) in pictures {
        tag.push_picture(
            lofty::picture::Picture::unchecked(data.clone())
                .pic_type(*kind)
                .mime_type(lofty::picture::MimeType::Png)
                .build(),
        );
    }
    file.save_to_path(&path, WriteOptions::default()).unwrap();
    path
}

fn pictures_of(path: &Path) -> Vec<(lofty::picture::PictureType, Vec<u8>)> {
    let file = lofty::read_from_path(path).unwrap();
    let mut pictures: Vec<_> = file
        .primary_tag()
        .unwrap()
        .pictures()
        .iter()
        .map(|p| (p.pic_type(), p.data().to_vec()))
        .collect();
    pictures.sort_by_key(|(kind, _)| kind.as_u8());
    pictures
}

/// The interface over `state`, with the image dialog replaced by `picker`
/// and a media cache the test can see.
fn harness_with_picker(
    state: AppState,
    media: fp_app::services::MediaCache,
    picker: impl Fn() -> Option<PathBuf> + Send + Sync + 'static,
) -> (Ui, std::sync::Arc<support::Fake>) {
    let fake = support::Fake::new(state);
    let mut app =
        fp_app::ui::app::AppUi::new(fake.clone(), fp_app::i18n::I18n::new(Some("en-US")), media);
    app.set_cover_picker(picker);
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1000.0, 700.0))
        .with_step_dt(0.02)
        .build_ui_state(|ui, app: &mut fp_app::ui::app::AppUi| app.ui(ui), app);
    h.run_steps(2);
    (h, fake)
}

fn button_enabled(h: &Ui, label: &str) -> bool {
    !h.get_by_role_and_label(Role::Button, label)
        .accesskit_node()
        .is_disabled()
}

fn click(h: &mut Ui, label: &str) {
    h.get_by_role_and_label(Role::Button, label).click();
    h.run_steps(3);
}

fn write_file(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn the_cover_area_says_what_the_file_has() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, _fake) = harness(state_at(&tagged_wav(dir.path(), "a.wav")));
    open_ready(&mut h);
    assert!(h.query_by_label("Cover").is_some());
    assert!(h.query_by_label("No cover.").is_some());
    assert!(button_enabled(&h, "Change…"));
    assert!(!button_enabled(&h, "Remove"), "nothing to remove");

    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_pictures(dir.path(), &[(PictureType::CoverFront, red())]);
    let (mut h, _fake) = harness(state_at(&path));
    open_ready(&mut h);
    assert!(h.query_by_label("Front cover.").is_some());
    assert!(button_enabled(&h, "Change…"));
    assert!(button_enabled(&h, "Remove"));
    assert!(!save_enabled(&h), "looking changes nothing");
}

#[test]
fn a_picture_that_is_not_a_front_cover_is_shown_but_cannot_be_removed() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_pictures(dir.path(), &[(PictureType::CoverBack, blue())]);
    let (mut h, _fake) = harness(state_at(&path));
    open_ready(&mut h);
    assert!(h.query_by_label_contains("No front cover").is_some());
    assert!(!button_enabled(&h, "Remove"), "it is not the front cover");
    assert!(button_enabled(&h, "Change…"), "a front cover can be added");
}

#[test]
fn a_format_that_cannot_store_a_cover_disables_the_area() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, _fake) = harness(state_at(&riff_wav(dir.path(), "a.wav")));
    open_ready(&mut h);
    assert!(
        h.query_by_label("This format cannot store a cover.")
            .is_some()
    );
    assert!(!button_enabled(&h, "Change…"));
    assert!(!button_enabled(&h, "Remove"));
    assert!(!save_enabled(&h));
}

#[test]
fn a_new_cover_is_staged_and_written_by_save() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_pictures(dir.path(), &[(PictureType::CoverFront, red())]);
    let image = write_file(dir.path(), "new.png", &blue());
    let before = std::fs::read(&path).unwrap();
    let (mut h, fake) = harness_with_picker(
        state_at(&path),
        fp_app::services::MediaCache::default(),
        move || Some(image.clone()),
    );
    open_ready(&mut h);
    click(&mut h, "Change…");
    wait_for_text(&mut h, "New cover, written when you save.");
    assert!(save_enabled(&h), "a changed cover is a change");
    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "nothing is written before Save"
    );
    click(&mut h, "Save");
    wait_for(&mut h, "the file has the new cover", |_| {
        pictures_of(&path) == [(PictureType::CoverFront, blue())]
    });
    wait_for(&mut h, "the modal closes", |h| {
        h.query_by_label("Edit tags").is_none()
    });
    let sheet = sheet_of(&path);
    assert_eq!(
        sheet.values(TagField::Genre),
        ["Pop"],
        "the text is as it was"
    );
    assert!(
        fake.take_sent()
            .iter()
            .any(|c| matches!(c, Command::ApplyTags { .. }))
    );
}

#[test]
fn the_cover_can_be_removed_and_save_writes_that() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_pictures(dir.path(), &[(PictureType::CoverFront, red())]);
    let (mut h, _fake) = harness(state_at(&path));
    open_ready(&mut h);
    click(&mut h, "Remove");
    assert!(
        h.query_by_label("The cover is removed when you save.")
            .is_some()
    );
    assert!(!button_enabled(&h, "Remove"));
    assert!(save_enabled(&h));
    click(&mut h, "Save");
    wait_for(&mut h, "the cover is gone from the file", |_| {
        pictures_of(&path).is_empty()
    });
    assert_eq!(sheet_of(&path).values(TagField::Title), ["Song 1"]);
}

#[test]
fn cancel_discards_a_staged_cover() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_pictures(dir.path(), &[(PictureType::CoverFront, red())]);
    let image = write_file(dir.path(), "new.png", &blue());
    let before = std::fs::read(&path).unwrap();
    let (mut h, fake) = harness_with_picker(
        state_at(&path),
        fp_app::services::MediaCache::default(),
        move || Some(image.clone()),
    );
    open_ready(&mut h);
    click(&mut h, "Change…");
    wait_for_text(&mut h, "New cover, written when you save.");
    click(&mut h, "Cancel");
    assert!(h.query_by_label("Edit tags").is_none());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(
        fake.take_sent()
            .iter()
            .all(|c| !matches!(c, Command::ApplyTags { .. }))
    );
    // Opened again, the editor starts from the file.
    open_ready(&mut h);
    assert!(h.query_by_label("Front cover.").is_some());
}

#[test]
fn a_back_cover_is_untouched_by_a_front_cover_change() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_pictures(
        dir.path(),
        &[
            (PictureType::CoverFront, red()),
            (PictureType::CoverBack, blue()),
        ],
    );
    let image = write_file(dir.path(), "new.png", &png(60, [30, 200, 30]));
    let (mut h, _fake) = harness_with_picker(
        state_at(&path),
        fp_app::services::MediaCache::default(),
        move || Some(image.clone()),
    );
    open_ready(&mut h);
    assert!(
        h.query_all_by_label_contains("1 other tag is kept")
            .next()
            .is_some(),
        "the back cover is one of the other tags"
    );
    click(&mut h, "Change…");
    wait_for_text(&mut h, "New cover, written when you save.");
    click(&mut h, "Save");
    wait_for(&mut h, "the front cover changed", |_| {
        pictures_of(&path)
            .iter()
            .any(|(kind, data)| *kind == PictureType::CoverFront && *data == png(60, [30, 200, 30]))
    });
    assert!(
        pictures_of(&path).contains(&(PictureType::CoverBack, blue())),
        "the back cover is as it was"
    );
}

#[test]
fn an_image_that_cannot_be_used_changes_nothing_and_says_why() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_pictures(dir.path(), &[(PictureType::CoverFront, red())]);
    let junk = write_file(dir.path(), "junk.png", b"this is not an image");
    let truncated = write_file(dir.path(), "cut.png", &blue()[..40]);
    let big = write_file(dir.path(), "big.png", &vec![0u8; 1024 * 1024 + 1]);
    for (file, why) in [
        (junk, "it is not a JPEG or PNG image"),
        (truncated, "it cannot be read as an image"),
        (big, "it is larger than 1 MiB"),
    ] {
        let mut s = state_at(&path);
        s.config.limits.max_cover_bytes = 1024 * 1024;
        let (mut h, _fake) =
            harness_with_picker(s, fp_app::services::MediaCache::default(), move || {
                Some(file.clone())
            });
        open_ready(&mut h);
        click(&mut h, "Change…");
        wait_for_text(&mut h, "The image was not used");
        assert!(h.query_by_label_contains(why).is_some(), "{why}");
        assert!(
            h.query_by_label("Front cover.").is_some(),
            "the cover is as it was"
        );
        assert!(!save_enabled(&h), "nothing changed");
        assert!(
            button_enabled(&h, "Change…"),
            "the operator can choose again"
        );
    }
}

#[test]
fn closing_the_dialog_without_a_choice_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_pictures(dir.path(), &[(PictureType::CoverFront, red())]);
    let (mut h, _fake) = harness_with_picker(
        state_at(&path),
        fp_app::services::MediaCache::default(),
        || None,
    );
    open_ready(&mut h);
    click(&mut h, "Change…");
    wait_for(&mut h, "Change is available again", |h| {
        button_enabled(h, "Change…")
    });
    assert!(h.query_by_label("Front cover.").is_some());
    assert!(!save_enabled(&h));
}

#[test]
fn the_image_dialog_never_blocks_the_interface() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_pictures(dir.path(), &[(PictureType::CoverFront, red())]);
    let image = write_file(dir.path(), "new.png", &blue());
    let ui_thread = std::thread::current().id();
    let (entered_tx, entered_rx) = crossbeam_channel::bounded::<bool>(1);
    let (release_tx, release_rx) = crossbeam_channel::bounded::<()>(1);
    let (mut h, _fake) = harness_with_picker(
        state_at(&path),
        fp_app::services::MediaCache::default(),
        move || {
            let _ = entered_tx.send(std::thread::current().id() != ui_thread);
            let _ = release_rx.recv();
            Some(image.clone())
        },
    );
    open_ready(&mut h);
    click(&mut h, "Change…");
    assert!(
        entered_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap(),
        "the dialog runs on a helper thread"
    );
    // The dialog is open: frames keep running, a second Change is off, and
    // Save waits for the image.
    h.run_steps(5);
    assert!(!button_enabled(&h, "Change…"));
    assert!(!save_enabled(&h));
    assert!(button_enabled(&h, "Cancel"));
    release_tx.send(()).unwrap();
    wait_for_text(&mut h, "New cover, written when you save.");
    assert!(button_enabled(&h, "Change…"));
    assert!(save_enabled(&h));
}

#[test]
fn a_saved_cover_reaches_the_cover_the_player_shows() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_pictures(dir.path(), &[(PictureType::CoverFront, red())]);
    let image = write_file(dir.path(), "new.png", &blue());
    let s = state_at(&path);
    let track = s.library.iter().next().unwrap().id;
    let media = fp_app::services::MediaCache::default();
    let old: std::sync::Arc<[u8]> = std::sync::Arc::from(&b"old thumbnail"[..]);
    media.seed(
        track,
        fp_app::services::TrackMedia {
            peaks: Vec::new(),
            peak_bucket_secs: 0.1,
            cover_png: Some(old.clone()),
        },
    );
    let version = media.version();
    let (mut h, _fake) = harness_with_picker(s, media.clone(), move || Some(image.clone()));
    open_ready(&mut h);
    click(&mut h, "Change…");
    wait_for_text(&mut h, "New cover, written when you save.");
    click(&mut h, "Save");
    wait_for(&mut h, "the cache has the new thumbnail", |_| {
        media
            .get(track)
            .is_some_and(|m| m.cover_png.as_deref() != Some(&old[..]))
    });
    assert!(
        media.version() > version,
        "the interface drops its textures"
    );
    let thumb = media.get(track).unwrap().cover_png.clone().unwrap();
    let decoded = image::load_from_memory_with_format(&thumb, image::ImageFormat::Png).unwrap();
    assert!(decoded.width() <= 128 && decoded.height() <= 128);
    // Removing it clears the thumbnail but keeps the rest of the entry.
    open_ready(&mut h);
    click(&mut h, "Remove");
    click(&mut h, "Save");
    wait_for(&mut h, "the cache has no cover", |_| {
        media.get(track).is_some_and(|m| m.cover_png.is_none())
    });
    assert_eq!(
        media.get(track).unwrap().peak_bucket_secs,
        0.1,
        "peaks stay"
    );
}

#[test]
fn a_track_the_cache_does_not_hold_is_left_alone_after_a_cover_save() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_pictures(dir.path(), &[(PictureType::CoverFront, red())]);
    let image = write_file(dir.path(), "new.png", &blue());
    let media = fp_app::services::MediaCache::default();
    let (mut h, _fake) =
        harness_with_picker(state_at(&path), media.clone(), move || Some(image.clone()));
    open_ready(&mut h);
    click(&mut h, "Change…");
    wait_for_text(&mut h, "New cover, written when you save.");
    click(&mut h, "Save");
    wait_for(&mut h, "the file has the new cover", |_| {
        pictures_of(&path) == [(PictureType::CoverFront, blue())]
    });
    h.run_steps(5);
    assert_eq!(media.version(), 0, "no entry, nothing to refresh");
}

// ---------------------------------------------------------------------------
// A field that was cut when read, and a picture that cannot be shown.
// ---------------------------------------------------------------------------

/// `tagged_wav` with a comment of `chars` characters.
fn wav_with_comment(dir: &Path, chars: usize) -> PathBuf {
    let path = tagged_wav(dir, "a.wav");
    let mut file = lofty::read_from_path(&path).unwrap();
    let tag = file.primary_tag_mut().unwrap();
    tag.set_comment("c".repeat(chars));
    file.save_to_path(&path, WriteOptions::default()).unwrap();
    path
}

#[test]
fn a_field_cut_when_read_is_read_only_and_saving_another_field_keeps_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_comment(dir.path(), 200);
    let mut s = state_at(&path);
    s.config.limits.max_tag_chars = 64;
    let (mut h, _fake) = harness(s);
    open_ready(&mut h);
    assert_eq!(
        h.query_all_by_label_contains("Too long to edit here; kept as it is in the file")
            .count(),
        1,
        "the note is under the comment only"
    );
    assert!(
        inputs(&h, "Comment")
            .next()
            .unwrap()
            .accesskit_node()
            .is_disabled(),
        "the comment cannot be edited"
    );
    assert!(
        !inputs(&h, "Title")
            .next()
            .unwrap()
            .accesskit_node()
            .is_disabled()
    );
    assert!(!save_enabled(&h), "looking changes nothing");
    type_into(&mut h, "Genre", 0, "Jazz");
    assert!(save_enabled(&h));
    h.get_by_role_and_label(Role::Button, "Save").click();
    wait_for(&mut h, "the file has the genre", |_| {
        sheet_of(&path).values(TagField::Genre) == ["Jazz"]
    });
    let full = Limits {
        max_tag_chars: 100_000,
        ..Limits::default()
    };
    let saved = read_tag_sheet(&path, &full).unwrap();
    assert_eq!(
        saved.values(TagField::Comment),
        ["c".repeat(200)],
        "the long comment is kept whole"
    );
}

#[test]
fn a_comment_within_the_limit_is_still_editable() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_comment(dir.path(), 20);
    let (mut h, _fake) = harness(state_at(&path));
    open_ready(&mut h);
    assert!(h.query_by_label_contains("Too long to edit here").is_none());
    assert!(
        !inputs(&h, "Comment")
            .next()
            .unwrap()
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn a_picture_that_cannot_be_decoded_is_said_to_be_kept() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_pictures(
        dir.path(),
        &[(
            PictureType::CoverFront,
            b"GIF89a not decodable here".to_vec(),
        )],
    );
    let (mut h, _fake) = harness(state_at(&path));
    open_ready(&mut h);
    assert!(
        h.query_by_label_contains("This cover cannot be shown; it is kept as it is")
            .is_some()
    );
    assert!(h.query_by_label("Front cover.").is_some());
    assert!(button_enabled(&h, "Change…"));
    assert!(button_enabled(&h, "Remove"), "it is a front cover");
    assert!(!save_enabled(&h), "looking changes nothing");
    click(&mut h, "Remove");
    assert!(save_enabled(&h));
    click(&mut h, "Save");
    wait_for(&mut h, "the cover is gone from the file", |_| {
        pictures_of(&path).is_empty()
    });
}

#[test]
fn a_picture_that_can_be_shown_has_no_such_note() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_pictures(dir.path(), &[(PictureType::CoverFront, red())]);
    let (mut h, _fake) = harness(state_at(&path));
    open_ready(&mut h);
    assert!(h.query_by_label_contains("cannot be shown").is_none());
}

// ---------------------------------------------------------------------------
// The editor's session: nothing from an earlier one reaches a later one.
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct Dropped(PathBuf);

impl egui::DroppedFile for Dropped {
    fn path(&self) -> &Path {
        &self.0
    }
    fn bytes(&self) -> Result<Vec<u8>, String> {
        std::fs::read(&self.0).map_err(|e| e.to_string())
    }
}

#[test]
fn a_stale_image_choice_never_reaches_a_later_editor() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav_with_pictures(dir.path(), &[(PictureType::CoverFront, red())]);
    let image = write_file(dir.path(), "new.png", &blue());
    let (entered_tx, entered_rx) = crossbeam_channel::bounded::<()>(1);
    let (release_tx, release_rx) = crossbeam_channel::bounded::<()>(1);
    let (mut h, _fake) = harness_with_picker(
        state_at(&path),
        fp_app::services::MediaCache::default(),
        move || {
            let _ = entered_tx.send(());
            let _ = release_rx.recv();
            Some(image.clone())
        },
    );
    open_ready(&mut h);
    click(&mut h, "Change…");
    entered_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    // Cancel works while the dialog is open; the same track is opened again.
    click(&mut h, "Cancel");
    assert!(h.query_by_label("Edit tags").is_none());
    open_ready(&mut h);
    assert!(button_enabled(&h, "Change…"), "a new session is not busy");
    // The first session's answer arrives now.
    release_tx.send(()).unwrap();
    for _ in 0..40 {
        h.run_steps(1);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(
        h.query_by_label("New cover, written when you save.")
            .is_none(),
        "the old choice staged nothing"
    );
    assert!(h.query_by_label("Front cover.").is_some());
    assert!(!save_enabled(&h));
    assert!(button_enabled(&h, "Change…"), "and freed nothing it held");
}

#[test]
fn files_dropped_under_the_editor_are_discarded() {
    let dir = tempfile::tempdir().unwrap();
    let path = tagged_wav(dir.path(), "a.wav");
    let file = write_file(dir.path(), "new.wav", b"x");
    let (mut h, fake) = harness(state_at(&path));
    open_ready(&mut h);
    fake.take_sent();
    h.input_mut()
        .dropped_files
        .push(std::sync::Arc::new(Dropped(file)));
    for _ in 0..40 {
        h.run_steps(1);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(
        fake.take_sent()
            .iter()
            .all(|c| !matches!(c, Command::InsertPaths { .. })),
        "nothing is inserted under the modal"
    );
}
