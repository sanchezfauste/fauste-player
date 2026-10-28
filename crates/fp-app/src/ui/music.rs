//! The phone's Music screen (phone layout spec §9): the device's music by
//! folder, artist or album, a search, and a selection to add to a player's
//! playlist. This part decides what to list; the screen draws it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use egui::{Align2, Rect, Role, Sense, Ui, WidgetInfo, vec2};
use egui_phosphor::regular as icon;
use fp_model::{Command, PlayerId, PlaylistId};

use super::app::{Scene, ViewState};
use super::media_browser::MediaFile;
use super::widgets::font;
use super::{format, theme, touch};

/// The selection bar's height (PL19): its two lines.
const BAR: f32 = 100.0;
/// The height of a group's row and of a track's row, for a finger.
const GROUP_ROW: f32 = 48.0;
const TRACK_ROW: f32 = 56.0;

/// How the screen groups the music (PL17).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Browse {
    #[default]
    Folders,
    Artists,
    Albums,
}

/// A row that opens more tracks: a sub-folder, an artist or an album
/// (`None`: the files without that tag).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Group {
    Folder(PathBuf),
    Artist(Option<String>),
    Album(Option<String>),
}

/// What the Music screen remembers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MusicView {
    pub browse: Browse,
    /// The folder shown; `None` is the top, where every file is.
    pub folder: Option<PathBuf>,
    /// The artist or album shown; `None` lists them.
    pub at: Option<Group>,
    pub search: String,
    /// The selected files, in the order they were tapped (PL19).
    pub ticked: Vec<PathBuf>,
    /// Where the selection goes: a player and a playlist (PL19).
    pub target: Option<(PlayerId, PlaylistId)>,
}

/// What to show: groups to open, then tracks (indices into the files).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Listing {
    pub groups: Vec<Group>,
    pub tracks: Vec<usize>,
}

impl MusicView {
    /// One step up (PL2): a folder to its parent, an artist or album to the
    /// list. False at the top.
    pub fn back(&mut self, files: &[MediaFile]) -> bool {
        if self.at.take().is_some() {
            return true;
        }
        let Some(folder) = self.folder.take() else {
            return false;
        };
        let top = top_folder(files);
        self.folder = folder
            .parent()
            .filter(|p| top.as_deref().is_none_or(|t| p.starts_with(t) && *p != t))
            .map(Path::to_path_buf);
        true
    }

    /// Selects `path`, or leaves it out when it was selected.
    pub fn toggle(&mut self, path: PathBuf) {
        match self.ticked.iter().position(|p| *p == path) {
            Some(at) => {
                self.ticked.remove(at);
            }
            None => self.ticked.push(path),
        }
    }

    /// Drops selected files that are not on the device any more.
    pub fn retain_listed(&mut self, files: &[MediaFile]) {
        if self.ticked.is_empty() {
            return;
        }
        let listed: BTreeSet<&Path> = files.iter().map(|f| f.path.as_path()).collect();
        self.ticked.retain(|p| listed.contains(p.as_path()));
    }
}

/// The deepest folder that holds every file, directly or below.
pub fn top_folder(files: &[MediaFile]) -> Option<PathBuf> {
    let mut parents = files.iter().filter_map(|f| f.path.parent());
    let mut top = parents.next()?.to_path_buf();
    for parent in parents {
        while !parent.starts_with(&top) {
            top = top.parent()?.to_path_buf();
        }
    }
    Some(top)
}

/// What the screen lists for `view`.
pub fn listing(files: &[MediaFile], view: &MusicView) -> Listing {
    let search = view.search.trim().to_lowercase();
    let mut listing = if !search.is_empty() {
        Listing {
            groups: Vec::new(),
            tracks: indices(files, |f| found(f, &search)),
        }
    } else {
        match (view.browse, &view.at) {
            (Browse::Folders, _) => folder_listing(files, view.folder.clone()),
            (Browse::Artists, Some(Group::Artist(name))) => Listing {
                groups: Vec::new(),
                tracks: indices(files, |f| f.artist == *name),
            },
            (Browse::Albums, Some(Group::Album(name))) => Listing {
                groups: Vec::new(),
                tracks: indices(files, |f| f.album == *name),
            },
            (Browse::Artists, _) => names(files, |f| &f.artist, Group::Artist),
            (Browse::Albums, _) => names(files, |f| &f.album, Group::Album),
        }
    };
    listing.tracks.sort_by_cached_key(|&i| {
        files
            .get(i)
            .map(|f| (f.title_or_name().to_lowercase(), f.path.clone()))
    });
    listing
}

fn indices(files: &[MediaFile], keep: impl Fn(&MediaFile) -> bool) -> Vec<usize> {
    (0..files.len())
        .filter(|&i| files.get(i).is_some_and(&keep))
        .collect()
}

/// Title, artist, album or file name holds `search` (lower case).
fn found(file: &MediaFile, search: &str) -> bool {
    let name = file
        .path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned());
    [
        file.title.as_ref(),
        file.artist.as_ref(),
        file.album.as_ref(),
        name.as_ref(),
    ]
    .into_iter()
    .flatten()
    .any(|t| t.to_lowercase().contains(search))
}

fn folder_listing(files: &[MediaFile], folder: Option<PathBuf>) -> Listing {
    let Some(at) = folder.or_else(|| top_folder(files)) else {
        return Listing::default();
    };
    let mut subfolders = BTreeSet::new();
    let mut tracks = Vec::new();
    for (i, file) in files.iter().enumerate() {
        let Some(parent) = file.path.parent() else {
            continue;
        };
        if parent == at {
            tracks.push(i);
        } else if let Ok(rest) = parent.strip_prefix(&at)
            && let Some(first) = rest.components().next()
        {
            let sub = at.join(first);
            subfolders.insert((sub.to_string_lossy().to_lowercase(), sub));
        }
    }
    Listing {
        groups: subfolders
            .into_iter()
            .map(|(_, f)| Group::Folder(f))
            .collect(),
        tracks,
    }
}

fn names(
    files: &[MediaFile],
    tag: impl Fn(&MediaFile) -> &Option<String>,
    group: impl Fn(Option<String>) -> Group,
) -> Listing {
    let set: BTreeSet<(Option<String>, Option<String>)> = files
        .iter()
        .map(|f| {
            let name = tag(f).clone();
            (name.as_ref().map(|n| n.to_lowercase()), name)
        })
        .collect();
    Listing {
        groups: set.into_iter().map(|(_, n)| group(n)).collect(),
        tracks: Vec::new(),
    }
}

/// What a tap on the screen asked for, applied after the list is drawn.
enum Action {
    Open(Group),
    Folder(Option<PathBuf>),
    Toggle(PathBuf),
}

/// The Music screen: the search, Folders | Artists | Albums, a breadcrumb
/// in Folders, and the list.
pub(crate) fn screen(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    let t = scene.i18n;
    let files = view_state
        .media
        .as_ref()
        .map(super::media_browser::MediaBrowser::files)
        .unwrap_or_default();
    let view = &mut view_state.music;
    view.retain_listed(&files);
    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
    let hint = t.tr("music-search");
    let search = ui.add(
        egui::TextEdit::singleline(&mut view.search)
            .hint_text(&hint)
            .font(font(15.0))
            .min_size(vec2(0.0, 40.0))
            .desired_width(f32::INFINITY),
    );
    search.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, hint.clone()));
    ui.horizontal(|ui| {
        for (browse, key) in [
            (Browse::Folders, "music-folders"),
            (Browse::Artists, "music-artists"),
            (Browse::Albums, "music-albums"),
        ] {
            if touch::chip(ui, &t.tr(key), view.browse == browse) && view.browse != browse {
                view.browse = browse;
                view.at = None;
            }
        }
    });
    let mut action = None;
    let searching = !view.search.trim().is_empty();
    if view.browse == Browse::Folders
        && !searching
        && let Some(top) = top_folder(&files)
    {
        action = breadcrumb(ui, &top, view.folder.as_deref()).or(action);
    }
    if files.is_empty() {
        ui.add(
            egui::Label::new(
                egui::RichText::new(t.tr("media-browser-empty"))
                    .font(font(13.0))
                    .color(theme::TEXT),
            )
            .selectable(false)
            .wrap(),
        );
        return;
    }
    let listed = listing(&files, view);
    let bar = !view.ticked.is_empty();
    let list_height = if bar {
        (ui.available_height() - BAR).max(0.0)
    } else {
        ui.available_height()
    };
    egui::ScrollArea::vertical()
        .id_salt("music-list")
        .max_height(list_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
            for group in listed.groups {
                let (glyph, name) = match &group {
                    Group::Folder(path) => (icon::FOLDER, file_name(path)),
                    Group::Artist(name) => (
                        icon::USER,
                        name.clone().unwrap_or_else(|| t.tr("music-unknown-artist")),
                    ),
                    Group::Album(name) => (
                        icon::VINYL_RECORD,
                        name.clone().unwrap_or_else(|| t.tr("music-unknown-album")),
                    ),
                };
                if group_row(ui, glyph, &name) {
                    action = Some(Action::Open(group));
                }
            }
            for index in listed.tracks {
                let Some(file) = files.get(index) else {
                    continue;
                };
                if track_row(ui, file, view.ticked.contains(&file.path)) {
                    action = Some(Action::Toggle(file.path.clone()));
                }
            }
        });
    if bar {
        let shown = view_state.phone.shown_player(&scene.state.player_ids());
        selection_bar(ui, scene, &mut view_state.music, shown);
    }
    let view = &mut view_state.music;
    match action {
        Some(Action::Open(Group::Folder(path))) => view.folder = Some(path),
        Some(Action::Open(group)) => view.at = Some(group),
        Some(Action::Folder(folder)) => view.folder = folder,
        Some(Action::Toggle(path)) => view.toggle(path),
        None => {}
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

/// The top folder, then each folder down to the one shown; a tap opens it.
fn breadcrumb(ui: &mut Ui, top: &Path, folder: Option<&Path>) -> Option<Action> {
    let mut crumbs = vec![(file_name(top), None)];
    if let Some(rest) = folder.and_then(|f| f.strip_prefix(top).ok()) {
        let mut at = top.to_path_buf();
        for part in rest.components() {
            at.push(part);
            crumbs.push((file_name(&at), Some(at.clone())));
        }
    }
    let last = crumbs.len().saturating_sub(1);
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        for (i, (name, path)) in crumbs.into_iter().enumerate() {
            if i > 0 {
                ui.label(egui::RichText::new(icon::CARET_RIGHT).color(theme::NEUTRAL_500));
            }
            let text = egui::RichText::new(&name)
                .font(font(13.0))
                .color(if i == last {
                    theme::TEXT
                } else {
                    theme::NEUTRAL_400
                });
            if ui
                .add(
                    egui::Button::new(text)
                        .frame(false)
                        .min_size(vec2(0.0, 32.0)),
                )
                .clicked()
            {
                action = Some(Action::Folder(path));
            }
        }
    });
    action
}

/// A folder, an artist or an album; true when tapped.
fn group_row(ui: &mut Ui, glyph: &str, name: &str) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), GROUP_ROW), Sense::click());
    let owned = name.to_owned();
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, owned.clone()));
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        if response.hovered() {
            p.rect_filled(rect, 4.0, theme::NEUTRAL_800);
        }
        p.text(
            rect.left_center() + vec2(14.0, 0.0),
            Align2::LEFT_CENTER,
            glyph,
            font(18.0),
            theme::NEUTRAL_400,
        );
        p.text(
            rect.left_center() + vec2(44.0, 0.0),
            Align2::LEFT_CENTER,
            name,
            font(15.0),
            theme::TEXT,
        );
    }
    response.clicked()
}

/// A track: a check box, the title over the artist, and the duration
/// (PL18). Its label joins the three; true when tapped.
fn track_row(ui: &mut Ui, file: &MediaFile, ticked: bool) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), TRACK_ROW), Sense::click());
    let title = file.title_or_name();
    let duration = file.duration_secs.map(format::clock);
    let label = [Some(title.clone()), file.artist.clone(), duration.clone()]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ");
    response.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, ticked, &label));
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        if ticked {
            p.rect_filled(rect, 4.0, theme::NEUTRAL_800);
        }
        let check = Rect::from_center_size(rect.left_center() + vec2(22.0, 0.0), vec2(20.0, 20.0));
        p.rect_stroke(
            check,
            3.0,
            egui::Stroke::new(1.5, theme::NEUTRAL_400),
            egui::StrokeKind::Inside,
        );
        if ticked {
            p.text(
                check.center(),
                Align2::CENTER_CENTER,
                icon::CHECK,
                font(14.0),
                theme::TEXT,
            );
        }
        let right = rect.right() - 12.0;
        let text_right = match &duration {
            Some(d) => {
                let galley = p.text(
                    egui::pos2(right, rect.center().y),
                    Align2::RIGHT_CENTER,
                    d,
                    font(13.0),
                    theme::NEUTRAL_400,
                );
                galley.left() - 8.0
            }
            None => right,
        };
        let left = rect.left() + 44.0;
        let width = (text_right - left).max(0.0);
        let line = |text: &str, size: f32, color, y: f32| {
            let mut job =
                egui::text::LayoutJob::simple_singleline(text.to_owned(), font(size), color);
            job.wrap = egui::text::TextWrapping::truncate_at_width(width);
            let galley = p.layout_job(job);
            p.galley(egui::pos2(left, y), galley, color);
        };
        let artist = file.artist.as_deref().unwrap_or_default();
        if artist.is_empty() {
            line(&title, 15.0, theme::TEXT, rect.center().y - 9.0);
        } else {
            line(&title, 15.0, theme::TEXT, rect.top() + 9.0);
            line(artist, 12.0, theme::NEUTRAL_400, rect.top() + 31.0);
        }
    }
    response.clicked()
}

/// The target's name: "P1 · Playlist 1".
fn target_name(scene: &Scene<'_>, player: PlayerId, playlist: PlaylistId) -> String {
    let index = scene
        .state
        .player_ids()
        .iter()
        .position(|p| *p == player)
        .unwrap_or_default();
    let name = scene
        .state
        .playlists
        .get(playlist)
        .map(|l| l.name.clone())
        .unwrap_or_default();
    format!(
        "{} · {name}",
        scene
            .i18n
            .tr_args("player-label", &[("n", (index + 1).into())])
    )
}

/// The bar under the list while something is selected (PL19): the count,
/// the target (a menu of every player and playlist) and the add buttons.
fn selection_bar(ui: &mut Ui, scene: &Scene<'_>, view: &mut MusicView, shown: Option<PlayerId>) {
    let t = scene.i18n;
    let state = scene.state;
    // The chosen target while it exists, else the Playlists screen's player
    // with the playlist it shows.
    let target = view
        .target
        .filter(|(player, playlist)| {
            state.player(*player).is_ok() && state.playlists.get(*playlist).is_some()
        })
        .or_else(|| {
            let player = shown?;
            Some((player, state.player(player).ok()?.playlist))
        });
    let Some((player, playlist)) = target else {
        return;
    };
    let len = state.playlists.get(playlist).map_or(0, |l| l.entries.len());
    // PL19: after the next only when the target holds the player's next.
    let after = fp_model::after_next(state, player)
        .filter(|(shown, index)| *shown == playlist && *index < len)
        .map(|(_, index)| index);
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(
                t.tr_args("music-selected", &[("count", view.ticked.len().into())]),
            )
            .font(font(13.0))
            .color(theme::TEXT),
        );
        let name = target_name(scene, player, playlist);
        ui.menu_button(egui::RichText::new(&name).font(font(13.0)), |ui| {
            for p in state.player_ids() {
                for list in state.playlists.iter() {
                    let item = target_name(scene, p, list.id);
                    let on = (p, list.id) == (player, playlist);
                    if ui
                        .add(egui::Button::selectable(on, item).min_size(vec2(200.0, 40.0)))
                        .clicked()
                    {
                        view.target = Some((p, list.id));
                        ui.close();
                    }
                }
            }
        });
    });
    let mut add = None;
    ui.horizontal(|ui| {
        let button = |ui: &mut Ui, key: &str| {
            ui.add(
                egui::Button::new(egui::RichText::new(t.tr(key)).font(font(14.0)))
                    .min_size(vec2(0.0, touch::TARGET)),
            )
            .clicked()
        };
        if let Some(index) = after
            && button(ui, "music-add-after-next")
        {
            add = Some(index);
        }
        if button(ui, "music-add-end") {
            add = Some(len);
        }
    });
    if let Some(index) = add {
        scene.ctl.send(Command::InsertPaths {
            playlist,
            index,
            paths: std::mem::take(&mut view.ticked),
        });
    }
}
