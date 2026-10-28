//! The media browser: where a platform without file dialogs (Android) lets
//! the operator pick music (mobile spec MB7). The platform hands over the
//! device's audio files (`set_files`); its pickers call `browse` on their
//! helper thread, which waits until the interface's window answers.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

use egui::accesskit::Role;
use egui::{RichText, Ui, WidgetInfo, vec2};

use super::theme;
use super::widgets::font;
use crate::i18n::I18n;

/// What a request asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowseMode {
    /// Any number of audio files.
    AudioFiles,
    /// One file with one of these extensions.
    OneFile { extensions: Vec<String> },
    /// One folder holding audio files.
    Folder,
}

impl BrowseMode {
    /// One file with one of `extensions`, when one of them is an audio
    /// file's: the browser lists only audio (images, playlists and cart
    /// pages are not there to pick).
    pub fn one_file(extensions: &[&str]) -> Option<Self> {
        extensions
            .iter()
            .any(|e| {
                super::files::AUDIO_EXTENSIONS
                    .iter()
                    .any(|a| a.eq_ignore_ascii_case(e))
            })
            .then(|| Self::OneFile {
                extensions: extensions.iter().map(|e| (*e).to_owned()).collect(),
            })
    }
}

/// One of the device's audio files with the tags MediaStore knows (phone
/// layout spec PL18); `None` where a tag is missing.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MediaFile {
    pub path: PathBuf,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_secs: Option<f64>,
}

impl MediaFile {
    /// A record from the platform: `path\0title\0artist\0album\0duration_ms`,
    /// or a bare path. Empty or unreadable fields are missing; `None`
    /// without a path.
    pub fn from_record(record: &str) -> Option<Self> {
        let mut fields = record.split('\0');
        let path = fields.next().filter(|p| !p.is_empty())?;
        let mut tag = || {
            fields
                .next()
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .map(str::to_owned)
        };
        let (title, artist, album) = (tag(), tag(), tag());
        let duration_secs = tag()
            .and_then(|ms| ms.parse::<f64>().ok())
            .filter(|ms| ms.is_finite() && *ms > 0.0)
            .map(|ms| ms / 1000.0);
        Some(Self {
            path: PathBuf::from(path),
            title,
            artist,
            album,
            duration_secs,
        })
    }

    /// The title, or the file name without its extension when untagged.
    pub fn title_or_name(&self) -> String {
        self.title.clone().unwrap_or_else(|| {
            self.path
                .file_stem()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
    }
}

impl From<PathBuf> for MediaFile {
    fn from(path: PathBuf) -> Self {
        Self {
            path,
            ..Self::default()
        }
    }
}

/// The device's audio files and at most one pending request. Clones share
/// them: the platform, the pickers' threads and the interface hold one each.
#[derive(Clone, Default)]
pub struct MediaBrowser {
    shared: Arc<Shared>,
}

#[derive(Default)]
struct Shared {
    inner: Mutex<Inner>,
    answered: Condvar,
}

#[derive(Default)]
struct Inner {
    files: Arc<Vec<MediaFile>>,
    next_id: u64,
    request: Option<(u64, BrowseMode, Option<PathBuf>)>,
    /// Answers not yet collected by their `browse` call.
    answers: Vec<(u64, Vec<PathBuf>)>,
    repaint: Option<egui::Context>,
}

impl MediaBrowser {
    /// The device's audio files, replacing the earlier list.
    ///
    /// True when the list was empty and is not any more: the app can read
    /// the device's music now (the media permission was granted), so files
    /// that failed before may play.
    pub fn set_files(&self, files: Vec<MediaFile>) -> bool {
        let (gained, repaint) = {
            let mut inner = self.lock();
            let gained = inner.files.is_empty() && !files.is_empty();
            inner.files = Arc::new(files);
            (gained, inner.repaint.clone())
        };
        if let Some(ctx) = repaint {
            ctx.request_repaint();
        }
        gained
    }

    pub fn files(&self) -> Arc<Vec<MediaFile>> {
        self.lock().files.clone()
    }

    /// Asks the operator and waits for the answer; nothing chosen when
    /// cancelled or when another request replaces this one. Blocks: call it
    /// from a helper thread only (rule 8).
    pub fn browse(&self, mode: BrowseMode, start: Option<PathBuf>) -> Vec<PathBuf> {
        let mut inner = self.lock();
        inner.next_id += 1;
        let id = inner.next_id;
        if let Some((replaced, _, _)) = inner.request.take() {
            inner.answers.push((replaced, Vec::new()));
            self.shared.answered.notify_all();
        }
        inner.request = Some((id, mode, start));
        let repaint = inner.repaint.clone();
        drop(inner);
        if let Some(ctx) = repaint {
            ctx.request_repaint();
        }
        let mut inner = self.lock();
        loop {
            if let Some(at) = inner.answers.iter().position(|(a, _)| *a == id) {
                return inner.answers.swap_remove(at).1;
            }
            inner = self
                .shared
                .answered
                .wait(inner)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// The request waiting for an answer: its id, what it asks for and
    /// where it starts.
    pub fn pending(&self) -> Option<(u64, BrowseMode, Option<PathBuf>)> {
        self.lock().request.clone()
    }

    /// Answers request `request`; ignored when it is no longer pending.
    pub fn answer(&self, request: u64, chosen: Vec<PathBuf>) {
        let mut inner = self.lock();
        if inner
            .request
            .as_ref()
            .is_some_and(|(id, _, _)| *id == request)
        {
            inner.request = None;
            inner.answers.push((request, chosen));
            self.shared.answered.notify_all();
        }
    }

    /// Answers the pending request, if any, with nothing: its picker's
    /// interface is gone (a new activity), so a choice could reach no one.
    pub fn cancel_pending(&self) {
        let mut inner = self.lock();
        if let Some((id, _, _)) = inner.request.take() {
            inner.answers.push((id, Vec::new()));
            self.shared.answered.notify_all();
        }
    }

    /// The interface to repaint when a request arrives or the files change.
    pub fn set_repaint(&self, ctx: egui::Context) {
        self.lock().repaint = Some(ctx);
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.shared
            .inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// The folders holding the files `mode` can choose, each with those files,
/// both sorted without regard to case.
pub fn folders(files: &[PathBuf], mode: &BrowseMode) -> Vec<(PathBuf, Vec<PathBuf>)> {
    let mut grouped: BTreeMap<(String, PathBuf), Vec<PathBuf>> = BTreeMap::new();
    for file in files.iter().filter(|f| matches(f, mode)) {
        let folder = file.parent().map(Path::to_path_buf).unwrap_or_default();
        grouped
            .entry((folder.to_string_lossy().to_lowercase(), folder))
            .or_default()
            .push(file.clone());
    }
    grouped
        .into_iter()
        .map(|((_, folder), mut files)| {
            files.sort_by_cached_key(|f| (sort_key(f), f.clone()));
            (folder, files)
        })
        .collect()
}

fn sort_key(file: &Path) -> String {
    file.file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

fn matches(file: &Path, mode: &BrowseMode) -> bool {
    match mode {
        BrowseMode::AudioFiles | BrowseMode::Folder => super::files::is_audio(file),
        BrowseMode::OneFile { extensions } => file
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| extensions.iter().any(|x| x.eq_ignore_ascii_case(e))),
    }
}

/// The height of a row, for a finger.
const ROW: f32 = 48.0;

/// What the window remembers while it answers one request.
#[derive(Default)]
pub(crate) struct BrowseView {
    request: u64,
    open: HashSet<PathBuf>,
    ticked: BTreeSet<PathBuf>,
}

/// Draws the window while `browser` has a request, and answers it.
pub(crate) fn show(ctx: &egui::Context, t: &I18n, browser: &MediaBrowser, view: &mut BrowseView) {
    let Some((request, mode, start)) = browser.pending() else {
        return;
    };
    if view.request != request {
        *view = BrowseView {
            request,
            open: start.into_iter().collect(),
            ticked: BTreeSet::new(),
        };
    }
    let files: Vec<PathBuf> = browser.files().iter().map(|f| f.path.clone()).collect();
    let grouped = folders(&files, &mode);
    // A file that left the list (a scan finished) leaves the selection.
    let listed: HashSet<&PathBuf> = grouped.iter().flat_map(|(_, f)| f).collect();
    view.ticked.retain(|f| listed.contains(f));
    let names = display_names(&grouped);
    let screen = ctx.content_rect();
    let width = (screen.width() - 32.0).max(240.0);
    let height = (screen.height() * 0.85 - 40.0).max(200.0);
    let mut answer: Option<Vec<PathBuf>> = None;
    let modal = egui::Modal::new(egui::Id::unique("media-browser"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(16.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.5))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            ui.label(
                RichText::new(t.tr("media-browser-title"))
                    .font(font(16.0))
                    .color(theme::TEXT),
            );
            let list_height = height - 2.0 * ROW;
            if grouped.is_empty() {
                ui.allocate_ui(vec2(width, list_height), |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(t.tr("media-browser-empty"))
                                .font(font(13.0))
                                .color(theme::TEXT),
                        )
                        .selectable(false)
                        .wrap(),
                    );
                });
            } else {
                egui::ScrollArea::vertical()
                    .max_height(list_height)
                    .min_scrolled_height(list_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for ((folder, files), name) in grouped.iter().zip(&names) {
                            if let Some(chosen) =
                                folder_rows(ui, t, &mode, folder, files, name, view)
                            {
                                answer = Some(chosen);
                            }
                        }
                    });
            }
            ui.horizontal(|ui| {
                if button(ui, &t.tr("media-browser-cancel"), true) {
                    answer = Some(Vec::new());
                }
                if mode == BrowseMode::AudioFiles {
                    let count = view.ticked.len();
                    let label = t.tr_args("media-browser-add", &[("count", count.into())]);
                    if button(ui, &label, count > 0) {
                        answer = Some(view.ticked.iter().cloned().collect());
                    }
                }
            });
        });
    if answer.is_none() && modal.should_close() {
        answer = Some(Vec::new());
    }
    if let Some(chosen) = answer {
        browser.answer(request, chosen);
    }
}

/// One folder: its row and, when open, its files. Returns the answer when a
/// tap gives one (a file in one-file mode, the folder in folder mode).
/// A frameless button `width` wide with its text on the left, next to the
/// tick box (`add_sized` would centre it).
fn left_aligned(text: RichText, width: f32) -> egui::Button<'static> {
    egui::Button::new((text, egui::Atom::grow()))
        .frame(false)
        .min_size(vec2(width, ROW))
}

fn folder_rows(
    ui: &mut Ui,
    t: &I18n,
    mode: &BrowseMode,
    folder: &Path,
    files: &[PathBuf],
    name: &str,
    view: &mut BrowseView,
) -> Option<Vec<PathBuf>> {
    let mut chosen = None;
    let count = t.tr_args("media-browser-files", &[("count", files.len().into())]);
    let open = view.open.contains(folder);
    ui.horizontal(|ui| {
        if *mode == BrowseMode::AudioFiles {
            let all = files.iter().all(|f| view.ticked.contains(f));
            if tick(ui, all, name, ROW) {
                if all {
                    files.iter().for_each(|f| {
                        view.ticked.remove(f);
                    });
                } else {
                    view.ticked.extend(files.iter().cloned());
                }
            }
        }
        let use_width = if *mode == BrowseMode::Folder {
            160.0
        } else {
            0.0
        };
        let row_width = (ui.available_width() - use_width).max(ROW);
        let arrow = if open {
            egui_phosphor::regular::CARET_DOWN
        } else {
            egui_phosphor::regular::CARET_RIGHT
        };
        let text = format!("{arrow} {name}  ·  {count}");
        let response = ui.add(left_aligned(
            RichText::new(text).font(font(13.0)).color(theme::TEXT),
            row_width,
        ));
        let label = format!("{name}, {count}");
        response.widget_info(|| WidgetInfo::labeled(Role::Button, true, label.clone()));
        if response.clicked() {
            if open {
                view.open.remove(folder);
            } else {
                view.open.insert(folder.to_path_buf());
            }
        }
        if *mode == BrowseMode::Folder && button(ui, &t.tr("media-browser-use-folder"), true) {
            chosen = Some(vec![folder.to_path_buf()]);
        }
    });
    if open {
        for file in files {
            let name = file
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            ui.horizontal(|ui| {
                ui.add_space(ROW);
                match mode {
                    BrowseMode::AudioFiles => {
                        let on = view.ticked.contains(file);
                        if tick(ui, on, &name, ui.available_width()) {
                            if on {
                                view.ticked.remove(file);
                            } else {
                                view.ticked.insert(file.clone());
                            }
                        }
                    }
                    BrowseMode::OneFile { .. } => {
                        let response = ui.add(left_aligned(
                            RichText::new(&name).font(font(13.0)).color(theme::TEXT),
                            ui.available_width(),
                        ));
                        if response.clicked() {
                            chosen = Some(vec![file.clone()]);
                        }
                    }
                    BrowseMode::Folder => {
                        ui.add_sized(
                            [ui.available_width(), ROW],
                            egui::Label::new(
                                RichText::new(&name)
                                    .font(font(13.0))
                                    .color(theme::NEUTRAL_400),
                            )
                            .selectable(false),
                        );
                    }
                }
            });
        }
    }
    chosen
}

/// A tick box a finger can hit, `width` wide, named `label` for screen
/// readers; the label is written beside the box when there is room. True
/// when tapped.
fn tick(ui: &mut Ui, on: bool, label: &str, width: f32) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(width.max(ROW), ROW), egui::Sense::click());
    let owned = label.to_owned();
    response.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, on, owned.clone()));
    let square = egui::Rect::from_center_size(
        egui::pos2(rect.left() + ROW / 2.0, rect.center().y),
        vec2(20.0, 20.0),
    );
    let painter = ui.painter();
    if on {
        painter.rect_filled(square, 0.0, theme::ACCENT);
    } else {
        let stroke = egui::Stroke::new(1.5, theme::NEUTRAL_400);
        painter.rect_stroke(square, 0.0, stroke, egui::StrokeKind::Inside);
    }
    if rect.width() > ROW {
        painter.text(
            egui::pos2(rect.left() + ROW, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            font(13.0),
            theme::TEXT,
        );
    }
    response.clicked()
}

fn button(ui: &mut Ui, label: &str, enabled: bool) -> bool {
    let w = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font(13.0), theme::TEXT)
        .size()
        .x
        + 32.0;
    super::widgets::tile(
        ui,
        vec2(w, ROW),
        label,
        enabled,
        super::widgets::TileStyle::plain(),
        |p, r, c| {
            p.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                label,
                font(13.0),
                c,
            );
        },
    )
    .clicked()
}

/// The folders' names without the part they all share, so `Music/A/CD1`
/// and `Music/B/CD1` read `A/CD1` and `B/CD1`; a folder always keeps its own
/// name.
fn display_names(grouped: &[(PathBuf, Vec<PathBuf>)]) -> Vec<String> {
    let parents: Vec<&Path> = grouped
        .iter()
        .map(|(folder, _)| folder.parent().unwrap_or(folder))
        .collect();
    let mut common: Option<PathBuf> = parents.first().map(|p| p.to_path_buf());
    for parent in parents.iter().skip(1) {
        while let Some(c) = common.as_ref() {
            if parent.starts_with(c) {
                break;
            }
            common = c.parent().map(Path::to_path_buf);
        }
    }
    grouped
        .iter()
        .map(|(folder, _)| {
            common
                .as_ref()
                .and_then(|c| folder.strip_prefix(c).ok())
                .filter(|rest| !rest.as_os_str().is_empty())
                .unwrap_or(folder)
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}
