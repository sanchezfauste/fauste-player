//! What the phone's Music screen lists (phone layout spec PL17).

use std::path::PathBuf;

use fp_app::ui::media_browser::MediaFile;
use fp_app::ui::music::{Browse, Group, MusicView, listing};

fn file(path: &str, title: &str, artist: &str, album: &str) -> MediaFile {
    let tag = |t: &str| (!t.is_empty()).then(|| t.to_owned());
    MediaFile {
        path: PathBuf::from(path),
        title: tag(title),
        artist: tag(artist),
        album: tag(album),
        duration_secs: Some(60.0),
    }
}

fn library() -> Vec<MediaFile> {
    vec![
        file("/sd/Music/Rock/b.mp3", "Beta", "Band", "First"),
        file("/sd/Music/Rock/Live/c.mp3", "Gamma", "Band", "Second"),
        file("/sd/Music/a.mp3", "alpha", "Other", "First"),
        file("/sd/Music/Jazz/d.mp3", "", "", ""),
    ]
}

fn titles(files: &[MediaFile], tracks: &[usize]) -> Vec<String> {
    tracks.iter().map(|&i| files[i].title_or_name()).collect()
}

#[test]
fn folders_start_where_every_file_is_and_list_sub_folders_then_tracks() {
    let files = library();
    let list = listing(&files, &MusicView::default());
    assert_eq!(
        list.groups,
        vec![
            Group::Folder(PathBuf::from("/sd/Music/Jazz")),
            Group::Folder(PathBuf::from("/sd/Music/Rock")),
        ]
    );
    assert_eq!(titles(&files, &list.tracks), ["alpha"]);
    let view = MusicView {
        folder: Some(PathBuf::from("/sd/Music/Rock")),
        ..MusicView::default()
    };
    let list = listing(&files, &view);
    assert_eq!(
        list.groups,
        vec![Group::Folder(PathBuf::from("/sd/Music/Rock/Live"))]
    );
    assert_eq!(titles(&files, &list.tracks), ["Beta"]);
}

#[test]
fn artists_and_albums_list_names_then_their_tracks() {
    let files = library();
    let view = MusicView {
        browse: Browse::Artists,
        ..MusicView::default()
    };
    assert_eq!(
        listing(&files, &view).groups,
        vec![
            Group::Artist(None),
            Group::Artist(Some("Band".to_owned())),
            Group::Artist(Some("Other".to_owned())),
        ]
    );
    let view = MusicView {
        browse: Browse::Artists,
        at: Some(Group::Artist(Some("Band".to_owned()))),
        ..MusicView::default()
    };
    let list = listing(&files, &view);
    assert!(list.groups.is_empty());
    assert_eq!(titles(&files, &list.tracks), ["Beta", "Gamma"]);
    let view = MusicView {
        browse: Browse::Albums,
        at: Some(Group::Album(Some("First".to_owned()))),
        ..MusicView::default()
    };
    assert_eq!(
        titles(&files, &listing(&files, &view).tracks),
        ["alpha", "Beta"]
    );
    let view = MusicView {
        browse: Browse::Albums,
        at: Some(Group::Album(None)),
        ..MusicView::default()
    };
    assert_eq!(titles(&files, &listing(&files, &view).tracks), ["d"]);
}

#[test]
fn a_search_finds_tracks_in_every_folder() {
    let files = library();
    let search = |text: &str| {
        let view = MusicView {
            search: text.to_owned(),
            folder: Some(PathBuf::from("/sd/Music/Jazz")),
            ..MusicView::default()
        };
        let list = listing(&files, &view);
        assert!(list.groups.is_empty());
        titles(&files, &list.tracks)
    };
    assert_eq!(search("BAND"), ["Beta", "Gamma"], "artist");
    assert_eq!(search("first"), ["alpha", "Beta"], "album");
    assert_eq!(search("gam"), ["Gamma"], "title");
    assert_eq!(search("d.mp3"), ["d"], "file name");
    assert_eq!(search("  "), ["d"], "blank: the folder");
}

#[test]
fn back_steps_up_then_reports_the_top() {
    let files = library();
    let mut view = MusicView {
        folder: Some(PathBuf::from("/sd/Music/Rock/Live")),
        ..MusicView::default()
    };
    assert!(view.back(&files));
    assert_eq!(view.folder, Some(PathBuf::from("/sd/Music/Rock")));
    assert!(view.back(&files));
    assert_eq!(view.folder, None);
    assert!(!view.back(&files));
    let mut view = MusicView {
        browse: Browse::Albums,
        at: Some(Group::Album(None)),
        ..MusicView::default()
    };
    assert!(view.back(&files));
    assert_eq!(view.at, None);
    assert!(!view.back(&files));
}

#[test]
fn the_selection_keeps_the_tap_order_and_follows_the_files() {
    let mut view = MusicView::default();
    view.toggle(PathBuf::from("/sd/Music/a.mp3"));
    view.toggle(PathBuf::from("/sd/Music/Jazz/d.mp3"));
    view.toggle(PathBuf::from("/sd/Music/Rock/b.mp3"));
    view.toggle(PathBuf::from("/sd/Music/a.mp3"));
    assert_eq!(
        view.ticked,
        [
            PathBuf::from("/sd/Music/Jazz/d.mp3"),
            PathBuf::from("/sd/Music/Rock/b.mp3")
        ]
    );
    view.toggle(PathBuf::from("/gone.mp3"));
    view.retain_listed(&library());
    assert_eq!(view.ticked.len(), 2);
}
