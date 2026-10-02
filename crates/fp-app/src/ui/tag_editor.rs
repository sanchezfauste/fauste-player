//! The tag editor (feedback 2 spec O23): a modal over the tag sheet of one
//! track. It edits a draft; reading the file and saving are the
//! application's job (`AppUi`), through the tag worker.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use egui::{RichText, vec2};
use egui_phosphor::regular as icon;
use fp_analysis::tags::CoverError;
use fp_model::{
    CoverArt, FieldProblem, Limits, TagEditBlock, TagField, TagFieldKind, TagSheet, TrackId,
    changed_fields, cover_blocked, cover_changed, cover_unstored, field_problem, invalid_fields,
    unstored_fields,
};

use super::app::Scene;
use super::theme;
use super::widgets::{self, TileStyle, font, font_medium};
use crate::tags::TagJob;

/// What the operator did this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditorAnswer {
    Open,
    Cancel,
    Save,
    /// **Change…** on the cover: the application opens the image dialog.
    ChangeCover,
}

/// The sheet and what the text boxes hold.
struct Form {
    /// What the file held when the sheet was read.
    original: TagSheet,
    /// `original` with the operator's changes.
    edited: TagSheet,
    /// The text of each box: the text itself, or the number and the total of
    /// a track or disc field.
    boxes: BTreeMap<TagField, (String, String)>,
}

impl Form {
    fn new(original: TagSheet) -> Self {
        let boxes = TagField::ALL
            .into_iter()
            .map(|field| {
                let text = if field.kind() == TagFieldKind::Pair {
                    original.pair(field)
                } else {
                    (original.text(field), String::new())
                };
                (field, text)
            })
            .collect();
        Self {
            edited: original.clone(),
            original,
            boxes,
        }
    }

    fn changed(&self) -> Vec<TagField> {
        changed_fields(&self.original, &self.edited)
    }

    fn invalid(&self) -> Vec<TagField> {
        invalid_fields(&self.original, &self.edited)
    }

    /// The operator changed a field or the cover.
    fn dirty(&self) -> bool {
        !self.changed().is_empty() || cover_changed(&self.original, &self.edited)
    }

    /// Everything that is changed can be written: values are valid and the
    /// cover, if changed, has a place in the format.
    fn writable(&self) -> bool {
        self.invalid().is_empty() && !cover_blocked(&self.original, &self.edited)
    }

    /// Clears the front cover (a picture only shown for lack of one stays).
    fn remove_cover(&mut self) {
        self.edited.remove_front_cover(&self.original);
    }

    /// Copies the boxes into the edited sheet; `true` if that changed it.
    fn sync(&mut self) -> bool {
        let before = self.edited.clone();
        for (field, (first, second)) in &self.boxes {
            // A field cut when read is shown read-only: it is never copied
            // back, so the draft cannot differ from the file's.
            if self.original.is_cut(*field) {
                continue;
            }
            if field.kind() == TagFieldKind::Pair {
                self.edited.set_pair(*field, first, second);
            } else {
                let as_lines = self.original.shows_lines(*field);
                self.edited.set_text(*field, first, as_lines);
            }
        }
        self.edited != before
    }
}

enum Phase {
    /// The tag worker is reading the file.
    Reading,
    /// The format has no writable tags or the file cannot be read.
    Unreadable,
    Ready(Box<Form>),
}

pub(crate) struct TagEditor {
    pub track: TrackId,
    /// Which opening of the editor this is: an answer to a request of an
    /// earlier opening (an image choice) is dropped.
    pub session: u64,
    /// A save is running on the tag worker.
    pub saving: bool,
    /// Why the last save failed, shown in the modal.
    pub error: Option<String>,
    /// Optional fields the operator added with **Add field**.
    added: BTreeSet<TagField>,
    /// The image dialog is open or the chosen image is being read: no second
    /// choice meanwhile, and no save.
    pub cover_busy: bool,
    /// Why the last image was not used, shown under the cover.
    cover_error: Option<CoverError>,
    /// The texture of the cover thumbnail on screen and the address of the
    /// bytes it was decoded from.
    cover_texture: Option<(usize, egui::TextureHandle)>,
    phase: Phase,
}

impl TagEditor {
    /// An editor waiting for the sheet of `track`.
    pub fn reading(track: TrackId, session: u64) -> Self {
        Self {
            track,
            session,
            saving: false,
            error: None,
            added: BTreeSet::new(),
            cover_busy: false,
            cover_error: None,
            cover_texture: None,
            phase: Phase::Reading,
        }
    }

    /// The tag worker's answer to the read; ignored unless one is awaited.
    pub fn arrived(&mut self, sheet: Option<TagSheet>) {
        if matches!(self.phase, Phase::Reading) {
            self.phase = match sheet {
                Some(sheet) => Phase::Ready(Box::new(Form::new(sheet))),
                None => Phase::Unreadable,
            };
        }
    }

    fn form(&self) -> Option<&Form> {
        match &self.phase {
            Phase::Ready(form) => Some(form),
            _ => None,
        }
    }

    /// The fields the operator changed, in editor order.
    pub fn changed(&self) -> Vec<TagField> {
        self.form().map(Form::changed).unwrap_or_default()
    }

    /// The changed fields whose value is not valid.
    pub fn invalid(&self) -> Vec<TagField> {
        self.form().map(Form::invalid).unwrap_or_default()
    }

    /// Something changed: a field or the cover.
    pub fn dirty(&self) -> bool {
        !self.changed().is_empty() || self.cover_changed()
    }

    /// The front cover was replaced or removed.
    pub fn cover_changed(&self) -> bool {
        self.form()
            .is_some_and(|form| cover_changed(&form.original, &form.edited))
    }

    /// Something changed (a field or the cover), all of it can be written,
    /// and no save runs and no image is being read.
    pub fn can_save(&self) -> bool {
        !self.saving && !self.cover_busy && self.dirty() && self.form().is_some_and(Form::writable)
    }

    /// The job that writes the draft, if it can be saved. `thumb_px` is the
    /// size of the thumbnails (`analysis.cover_thumb_px`).
    pub fn save_job(&self, path: &Path, limits: &Limits, thumb_px: u32) -> Option<TagJob> {
        let form = self.form().filter(|_| self.can_save())?;
        Some(TagJob::WriteSheet {
            track: self.track,
            path: path.to_path_buf(),
            before: Box::new(form.original.clone()),
            after: Box::new(form.edited.clone()),
            limits: limits.clone(),
            thumb_px,
        })
    }

    /// **Change…** was pressed and the image dialog is opening.
    pub fn picking_cover(&mut self) {
        self.cover_busy = true;
        self.cover_error = None;
    }

    /// The dialog was closed without a choice.
    pub fn cover_not_picked(&mut self) {
        self.cover_busy = false;
    }

    /// The tag worker's answer to reading the chosen image: a good one
    /// becomes the draft's front cover (saved with **Save**); a bad one
    /// changes nothing and is reported under the cover.
    pub fn cover_loaded(&mut self, result: Result<CoverArt, CoverError>) {
        self.cover_busy = false;
        match (&mut self.phase, result) {
            (Phase::Ready(form), Ok(cover)) => {
                form.edited.set_cover(Some(cover));
                self.cover_error = None;
            }
            (_, Err(why)) => self.cover_error = Some(why),
            _ => {}
        }
    }

    /// Whether the file does not hold the cover as written after the save
    /// (judged against the sheet read back).
    pub fn cover_unstored(&self, saved: Option<&TagSheet>) -> bool {
        match (self.form(), saved) {
            (Some(form), Some(saved)) => cover_unstored(&form.original, &form.edited, saved),
            _ => false,
        }
    }

    /// The changed fields the file does not hold as written, judged against
    /// the sheet read back after the save.
    pub fn unstored(&self, saved: Option<&TagSheet>, limits: &Limits) -> Vec<TagField> {
        match (self.form(), saved) {
            (Some(form), Some(saved)) => {
                let written = form
                    .edited
                    .clone()
                    .clamped(limits.max_tag_chars, limits.max_tag_values);
                unstored_fields(&form.original, &written, saved)
            }
            _ => Vec::new(),
        }
    }
}

/// The Fluent key of the label of `field`.
pub(crate) fn field_key(field: TagField) -> String {
    format!("tag-field-{}", field.slug())
}

/// The Fluent key of the reason `block` gives for a disabled edit.
pub(crate) fn block_key(block: TagEditBlock) -> &'static str {
    match block {
        TagEditBlock::FileUnavailable => "menu-edit-tags-file",
        TagEditBlock::UnsupportedFormat => "menu-edit-tags-format",
        TagEditBlock::TagsNotRead => "menu-edit-tags-unread",
        TagEditBlock::OnAir => "menu-edit-tags-on-air",
        TagEditBlock::Cued => "menu-edit-tags-cued",
        TagEditBlock::OnCart => "menu-edit-tags-cart",
    }
}

/// A button as wide as its label; `accent` for the main action.
fn button(ui: &mut egui::Ui, text: &str, enabled: bool, accent: bool) -> bool {
    let w = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font_medium(13.0), theme::TEXT)
        .size()
        .x
        + 28.0;
    let style = if accent {
        TileStyle {
            fill: theme::ACCENT,
            border: theme::ACCENT,
            content: theme::NEUTRAL_900,
            hover_fill: theme::ACCENT_400,
            hover_content: theme::NEUTRAL_900,
            active_fill: theme::ACCENT_300,
            ..TileStyle::plain()
        }
    } else {
        TileStyle::plain()
    };
    widgets::tile(ui, vec2(w, 30.0), text, enabled, style, |p, r, c| {
        p.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            text,
            font_medium(13.0),
            c,
        );
    })
    .clicked()
}

fn note(ui: &mut egui::Ui, text: String, color: egui::Color32) {
    ui.add(
        egui::Label::new(RichText::new(text).font(font(12.0)).color(color))
            .wrap()
            .selectable(false),
    );
}

/// The boxes of one field: a number and a total, several lines, or one line.
fn field_boxes(
    ui: &mut egui::Ui,
    t: &crate::i18n::I18n,
    field: TagField,
    name: &egui::Response,
    boxes: &mut (String, String),
    lines: bool,
    max_chars: usize,
) {
    match field.kind() {
        TagFieldKind::Pair => {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut boxes.0)
                        .char_limit(10)
                        .desired_width(64.0),
                )
                .labelled_by(name.id);
                let total = ui.label(
                    RichText::new(t.tr("tags-total"))
                        .font(font(12.0))
                        .color(theme::NEUTRAL_400),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut boxes.1)
                        .char_limit(10)
                        .desired_width(64.0),
                )
                .labelled_by(total.id);
            });
        }
        TagFieldKind::Date => {
            ui.add(
                egui::TextEdit::singleline(&mut boxes.0)
                    .char_limit(19)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(180.0),
            )
            .labelled_by(name.id);
        }
        TagFieldKind::Whole => {
            ui.add(
                egui::TextEdit::singleline(&mut boxes.0)
                    .char_limit(10)
                    .desired_width(90.0),
            )
            .labelled_by(name.id);
        }
        TagFieldKind::LongText | TagFieldKind::Text if lines || field == TagField::Comment => {
            let rows = boxes.0.lines().count().clamp(2, 6);
            ui.add(
                egui::TextEdit::multiline(&mut boxes.0)
                    .char_limit(max_chars)
                    .desired_rows(rows)
                    .desired_width(320.0),
            )
            .labelled_by(name.id);
        }
        TagFieldKind::LongText => {
            ui.add(
                egui::TextEdit::multiline(&mut boxes.0)
                    .char_limit(max_chars)
                    .desired_rows(6)
                    .desired_width(320.0),
            )
            .labelled_by(name.id);
        }
        TagFieldKind::Text => {
            ui.add(
                egui::TextEdit::singleline(&mut boxes.0)
                    .char_limit(max_chars)
                    .desired_width(320.0),
            )
            .labelled_by(name.id);
        }
    }
}

/// Side of the square the cover thumbnail is drawn in.
const COVER_BOX: f32 = 80.0;

/// The text of the reason an image was not used.
fn cover_error_text(t: &crate::i18n::I18n, why: &CoverError, limits: &Limits) -> String {
    let reason = match why {
        CoverError::TooLarge => {
            let mib = usize::try_from(limits.max_cover_bytes / (1024 * 1024)).unwrap_or(0);
            t.tr_args("tags-cover-too-large", &[("limit", mib.into())])
        }
        CoverError::UnsupportedFormat => t.tr("tags-cover-unsupported"),
        CoverError::Undecodable => t.tr("tags-cover-undecodable"),
        CoverError::Unreadable(detail) => detail.clone(),
    };
    t.tr_args("tags-cover-error", &[("reason", reason.into())])
}

/// The texture of `cover`'s thumbnail, decoded once per thumbnail.
fn cover_texture<'a>(
    ctx: &egui::Context,
    cover: &CoverArt,
    slot: &'a mut Option<(usize, egui::TextureHandle)>,
) -> Option<&'a egui::TextureHandle> {
    let png = cover.thumb_png()?;
    let key = png.as_ptr() as usize;
    if slot.as_ref().map(|(k, _)| *k) != Some(key) {
        let decoded = image::load_from_memory_with_format(png, image::ImageFormat::Png).ok()?;
        let rgba = decoded.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
        let handle = ctx.load_texture("tag-cover", image, egui::TextureOptions::LINEAR);
        *slot = Some((key, handle));
    }
    slot.as_ref().map(|(_, handle)| handle)
}

/// The cover area: the thumbnail, **Change…**, **Remove** and a note on
/// what Save will do. Returns `true` when **Change…** was pressed.
fn cover_area(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    t: &crate::i18n::I18n,
    limits: &Limits,
    form: &mut Form,
    state: (bool, Option<&CoverError>),
    texture: &mut Option<(usize, egui::TextureHandle)>,
) -> bool {
    let (busy, error) = state;
    let storable = form.edited.can_store_cover();
    let shown = form.edited.cover().cloned();
    let front = shown.as_ref().is_some_and(CoverArt::is_front);
    let mut change = false;
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(COVER_BOX, COVER_BOX), egui::Sense::hover());
        ui.painter().rect_filled(rect, 0.0, theme::NEUTRAL_900);
        let handle = shown
            .as_ref()
            .and_then(|cover| cover_texture(ctx, cover, texture));
        match handle {
            Some(handle) => {
                let size = handle.size_vec2();
                let scale = COVER_BOX / size.x.max(size.y).max(1.0);
                let fitted = egui::Rect::from_center_size(rect.center(), size * scale);
                ui.painter().image(
                    handle.id(),
                    fitted,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
            None => widgets::glyph(
                ui.painter(),
                rect,
                icon::VINYL_RECORD,
                32.0,
                theme::TEXT.gamma_multiply(0.35),
                false,
            ),
        }
        ui.vertical(|ui| {
            ui.label(
                RichText::new(t.tr("tags-cover"))
                    .font(font(12.0))
                    .color(theme::NEUTRAL_400),
            );
            ui.horizontal(|ui| {
                if button(ui, &t.tr("tags-cover-change"), storable && !busy, false) {
                    change = true;
                }
                if button(ui, &t.tr("tags-cover-remove"), storable && front, false) {
                    form.remove_cover();
                }
            });
            let key = match (storable, &shown) {
                (false, _) => "tags-cover-not-stored",
                (true, None) if form.original.cover().is_some_and(CoverArt::is_front) => {
                    "tags-cover-removed"
                }
                (true, None) => "tags-cover-none",
                (true, Some(_)) if cover_changed(&form.original, &form.edited) => "tags-cover-new",
                (true, Some(cover)) if cover.is_front() => "tags-cover-front",
                (true, Some(_)) => "tags-cover-first",
            };
            note(ui, t.tr(key), theme::NEUTRAL_400);
            if shown.as_ref().is_some_and(|c| c.thumb_png().is_none()) {
                note(ui, t.tr("tags-cover-hidden"), theme::NEUTRAL_500);
            }
            if let Some(why) = error {
                note(ui, cover_error_text(t, why, limits), theme::AMBER);
            }
        });
    });
    change
}

pub(crate) fn show(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    editor: &mut TagEditor,
    block: Option<TagEditBlock>,
) -> EditorAnswer {
    let t = scene.i18n;
    let limits = &scene.state.config.limits;
    let max = limits.max_tag_chars;
    let file_name = scene
        .state
        .library
        .get(editor.track)
        .and_then(|track| track.path.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let width = (ctx.content_rect().width() - 48.0).clamp(360.0, 560.0);
    let list_height = (ctx.content_rect().height() - 360.0).clamp(140.0, 560.0);
    let invalid = editor.invalid();
    let mut can_save = false;
    let TagEditor {
        saving,
        error,
        added,
        phase,
        cover_busy,
        cover_error,
        cover_texture,
        ..
    } = editor;
    let saving = *saving;
    let mut answer = EditorAnswer::Open;
    let modal = egui::Modal::new(egui::Id::new("tag-editor"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("tags-editor-title"))
                        .font(font_medium(18.0))
                        .color(theme::TEXT),
                )
                .selectable(false),
            );
            ui.add(
                egui::Label::new(
                    RichText::new(file_name)
                        .font(font(11.0))
                        .color(theme::NEUTRAL_400),
                )
                .selectable(false)
                .truncate(),
            );
            match phase {
                Phase::Reading => note(ui, t.tr("tags-reading"), theme::NEUTRAL_300),
                Phase::Unreadable => note(ui, t.tr("tags-unreadable"), theme::AMBER),
                Phase::Ready(form) => {
                    ui.add_enabled_ui(!saving, |ui| {
                        if cover_area(
                            ui,
                            ctx,
                            t,
                            limits,
                            form,
                            (*cover_busy, cover_error.as_ref()),
                            cover_texture,
                        ) {
                            answer = EditorAnswer::ChangeCover;
                        }
                        egui::ScrollArea::vertical()
                            .max_height(list_height)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                egui::Grid::new("tag-editor-fields")
                                    .num_columns(2)
                                    .spacing(vec2(10.0, 6.0))
                                    .show(ui, |ui| {
                                        for field in form.original.visible_fields(added) {
                                            let problem =
                                                field_problem(&form.original, &form.edited, field);
                                            let bad = invalid.contains(&field);
                                            let cut = form.original.is_cut(field);
                                            let storable = form.original.can_store(field);
                                            let label = ui.label(
                                                RichText::new(t.tr(&field_key(field)))
                                                    .font(font(12.0))
                                                    .color(if bad {
                                                        theme::AMBER
                                                    } else {
                                                        theme::NEUTRAL_400
                                                    }),
                                            );
                                            let lines = form.original.shows_lines(field);
                                            ui.vertical(|ui| {
                                                let frame = egui::Frame::new()
                                                    .inner_margin(2.0)
                                                    .corner_radius(3.0)
                                                    .stroke(if bad {
                                                        egui::Stroke::new(1.0, theme::AMBER)
                                                    } else {
                                                        egui::Stroke::NONE
                                                    });
                                                frame.show(ui, |ui| {
                                                    ui.add_enabled_ui(storable && !cut, |ui| {
                                                        if let Some(boxes) =
                                                            form.boxes.get_mut(&field)
                                                        {
                                                            field_boxes(
                                                                ui, t, field, &label, boxes, lines,
                                                                max,
                                                            );
                                                        }
                                                    });
                                                });
                                                match problem {
                                                    Some(FieldProblem::InvalidValue) => {
                                                        let key =
                                                            if field.kind() == TagFieldKind::Date {
                                                                "tags-date-invalid"
                                                            } else {
                                                                "tags-number-invalid"
                                                            };
                                                        note(ui, t.tr(key), theme::AMBER);
                                                    }
                                                    Some(FieldProblem::NotStorable) => note(
                                                        ui,
                                                        t.tr("tags-not-stored"),
                                                        theme::AMBER,
                                                    ),
                                                    Some(FieldProblem::TooLongToEdit) => {
                                                        note(ui, t.tr("tags-cut"), theme::AMBER)
                                                    }
                                                    None if cut => note(
                                                        ui,
                                                        t.tr("tags-cut"),
                                                        theme::NEUTRAL_400,
                                                    ),
                                                    None if !storable => note(
                                                        ui,
                                                        t.tr("tags-not-stored"),
                                                        theme::NEUTRAL_500,
                                                    ),
                                                    None => {}
                                                }
                                            });
                                            ui.end_row();
                                        }
                                    });
                            });
                        // The boxes were edited above: bring the sheet and
                        // what depends on it (the marks, Save) up to date.
                        if form.sync() {
                            ctx.request_repaint();
                        }
                        can_save = !saving
                            && !*cover_busy
                            && block.is_none()
                            && form.dirty()
                            && form.writable();
                        let addable = form.original.addable_fields(added);
                        ui.add_enabled_ui(!addable.is_empty(), |ui| {
                            ui.menu_button(t.tr("tags-add-field"), |ui| {
                                for field in &addable {
                                    if ui.button(t.tr(&field_key(*field))).clicked() {
                                        added.insert(*field);
                                        ui.close();
                                    }
                                }
                            });
                        });
                        let (kept, more) =
                            (form.original.other_kept, form.original.other_kept_more);
                        let key = match (kept, more) {
                            (0, false) => None,
                            (0, true) => Some("tags-others-kept-uncounted"),
                            (_, false) => Some("tags-others-kept"),
                            (_, true) => Some("tags-others-kept-more"),
                        };
                        if let Some(key) = key {
                            note(
                                ui,
                                t.tr_args(key, &[("count", kept.into())]),
                                theme::NEUTRAL_400,
                            );
                        }
                    });
                }
            }
            if let Some(reason) = block {
                note(ui, t.tr(block_key(reason)), theme::AMBER);
            }
            if let Some(text) = error.as_deref() {
                note(ui, text.to_owned(), theme::AMBER);
            }
            if saving {
                note(ui, t.tr("tags-saving"), theme::NEUTRAL_300);
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if button(ui, &t.tr("tags-save"), can_save, true) {
                    answer = EditorAnswer::Save;
                }
                if button(ui, &t.tr("tags-cancel"), !saving, false) {
                    answer = EditorAnswer::Cancel;
                }
            });
        });
    // Escape or a click on the backdrop cancels, unless a save is running.
    if modal.should_close() && !saving {
        answer = EditorAnswer::Cancel;
    }
    answer
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet() -> TagSheet {
        let mut s = TagSheet::new(TagField::ALL, 2, false);
        s.set_text(TagField::Title, "Song", false);
        s.set_text(TagField::Artist, "One\nTwo", true);
        s.set_pair(TagField::TrackNumber, "3", "12");
        s
    }

    fn ready() -> TagEditor {
        let mut e = TagEditor::reading(TrackId(1), 1);
        e.arrived(Some(sheet()));
        e
    }

    /// Types `text` into the box of `field`, as a frame of `show` would.
    fn type_into(e: &mut TagEditor, field: TagField, text: &str) {
        let Phase::Ready(form) = &mut e.phase else {
            panic!("not ready");
        };
        form.boxes.get_mut(&field).unwrap().0 = text.to_owned();
        form.sync();
    }

    #[test]
    fn the_sheet_is_taken_once_while_reading() {
        let mut e = TagEditor::reading(TrackId(1), 1);
        assert!(e.form().is_none());
        e.arrived(Some(sheet()));
        assert!(e.form().is_some());
        e.arrived(None);
        assert!(e.form().is_some(), "a late answer changes nothing");
        let mut e = TagEditor::reading(TrackId(1), 1);
        e.arrived(None);
        assert!(matches!(e.phase, Phase::Unreadable));
    }

    #[test]
    fn an_untouched_draft_cannot_be_saved() {
        let e = ready();
        assert!(e.changed().is_empty());
        assert!(!e.can_save());
        assert!(
            e.save_job(Path::new("/x.mp3"), &Limits::default(), 64)
                .is_none()
        );
    }

    #[test]
    fn the_boxes_become_the_sheet() {
        let mut e = ready();
        type_into(&mut e, TagField::Artist, "One\n\nTwo\nThree");
        type_into(&mut e, TagField::Comment, "a\nb");
        let Phase::Ready(form) = &e.phase else {
            panic!()
        };
        assert_eq!(
            form.edited.values(TagField::Artist),
            ["One", "Two", "Three"]
        );
        assert_eq!(
            form.edited.values(TagField::Comment),
            ["a\nb"],
            "a comment is one value over two lines"
        );
        assert_eq!(e.changed(), [TagField::Artist, TagField::Comment]);
    }

    #[test]
    fn an_invalid_value_blocks_the_save() {
        let mut e = ready();
        type_into(&mut e, TagField::Date, "2019-13");
        assert_eq!(e.invalid(), [TagField::Date]);
        assert!(!e.can_save());
        type_into(&mut e, TagField::Date, "2019-05-14");
        assert!(e.invalid().is_empty());
        assert!(e.can_save());
        e.saving = true;
        assert!(!e.can_save(), "no second save while one runs");
    }

    #[test]
    fn the_save_job_carries_what_was_read_and_what_is_wanted() {
        let mut e = ready();
        type_into(&mut e, TagField::Genre, "Jazz");
        let job = e
            .save_job(Path::new("/x.mp3"), &Limits::default(), 64)
            .unwrap();
        let TagJob::WriteSheet {
            track,
            path,
            before,
            after,
            ..
        } = job
        else {
            panic!("not a sheet write");
        };
        assert_eq!(track, TrackId(1));
        assert_eq!(path, Path::new("/x.mp3"));
        assert!(!before.has(TagField::Genre));
        assert_eq!(after.values(TagField::Genre), ["Jazz"]);
    }

    #[test]
    fn what_the_file_did_not_keep_is_named() {
        let mut e = ready();
        type_into(&mut e, TagField::Genre, "Jazz");
        type_into(&mut e, TagField::Mood, "Calm");
        let mut saved = sheet();
        saved.set_text(TagField::Genre, "Jazz", true);
        let limits = Limits::default();
        assert_eq!(e.unstored(Some(&saved), &limits), [TagField::Mood]);
        assert!(
            e.unstored(None, &limits).is_empty(),
            "no read-back, no claim"
        );
    }

    #[test]
    fn a_cut_field_is_never_copied_back_into_the_draft() {
        let mut long = sheet();
        long.set_text(TagField::Comment, &"c".repeat(40), false);
        let mut e = TagEditor::reading(TrackId(1), 1);
        e.arrived(Some(long.clamped(10, 5)));
        let Phase::Ready(form) = &e.phase else {
            panic!("not ready");
        };
        assert!(form.original.is_cut(TagField::Comment));
        type_into(&mut e, TagField::Comment, "changed");
        assert!(e.changed().is_empty(), "the box of a cut field is ignored");
        assert!(!e.can_save());
        // Another field still saves.
        type_into(&mut e, TagField::Genre, "Jazz");
        assert_eq!(e.changed(), [TagField::Genre]);
        assert!(e.can_save());
    }

    #[test]
    fn every_field_and_block_has_a_key() {
        assert_eq!(field_key(TagField::AlbumArtist), "tag-field-album-artist");
        assert_eq!(block_key(TagEditBlock::OnAir), "menu-edit-tags-on-air");
    }

    /// A ready editor whose file can hold a cover; `cover` is what it has.
    fn ready_with_cover(cover: Option<CoverArt>) -> TagEditor {
        let mut sheet = sheet().with_cover_support(true);
        sheet.set_cover(cover);
        let mut e = TagEditor::reading(TrackId(1), 1);
        e.arrived(Some(sheet));
        e
    }

    fn art(byte: u8, front: bool) -> CoverArt {
        CoverArt::new(vec![byte; 4], front).with_thumb(Some(vec![byte]))
    }

    fn shown(e: &TagEditor) -> Option<CoverArt> {
        e.form().and_then(|f| f.edited.cover().cloned())
    }

    #[test]
    fn a_staged_cover_is_a_change_that_can_be_saved() {
        let mut e = ready_with_cover(Some(art(1, true)));
        assert!(!e.cover_changed() && !e.dirty() && !e.can_save());
        e.cover_loaded(Ok(art(2, true)));
        assert!(e.cover_changed() && e.dirty());
        assert!(e.changed().is_empty(), "no field changed");
        assert!(e.can_save());
        assert_eq!(shown(&e), Some(art(2, true)));
        // Choosing the file's own cover again is no change.
        e.cover_loaded(Ok(art(1, true)));
        assert!(!e.cover_changed() && !e.can_save());
    }

    #[test]
    fn removing_the_front_cover_is_a_change() {
        let mut e = ready_with_cover(Some(art(1, true)));
        let Phase::Ready(form) = &mut e.phase else {
            panic!("not ready");
        };
        form.remove_cover();
        assert!(e.cover_changed() && e.can_save());
        assert_eq!(shown(&e), None);
    }

    #[test]
    fn a_picture_that_is_only_shown_stays_when_remove_is_used() {
        let mut e = ready_with_cover(Some(art(5, false)));
        e.cover_loaded(Ok(art(6, true)));
        assert!(e.cover_changed());
        let Phase::Ready(form) = &mut e.phase else {
            panic!("not ready");
        };
        form.remove_cover();
        assert!(!e.cover_changed(), "back to showing the file's picture");
        assert_eq!(shown(&e), Some(art(5, false)));
    }

    #[test]
    fn a_bad_image_changes_nothing_and_is_remembered() {
        let mut e = ready_with_cover(Some(art(1, true)));
        e.picking_cover();
        assert!(e.cover_busy && e.cover_error.is_none());
        e.cover_loaded(Err(CoverError::Undecodable));
        assert!(!e.cover_busy);
        assert_eq!(e.cover_error, Some(CoverError::Undecodable));
        assert!(!e.cover_changed() && !e.can_save());
        assert_eq!(shown(&e), Some(art(1, true)));
        // The next choice starts clean.
        e.picking_cover();
        assert!(e.cover_error.is_none());
        e.cover_loaded(Ok(art(2, true)));
        assert!(e.cover_error.is_none() && e.cover_changed());
    }

    #[test]
    fn nothing_is_saved_while_an_image_is_being_read() {
        let mut e = ready_with_cover(Some(art(1, true)));
        e.cover_loaded(Ok(art(2, true)));
        assert!(e.can_save());
        e.picking_cover();
        assert!(!e.can_save(), "the choice is not in yet");
        e.cover_not_picked();
        assert!(e.can_save(), "a closed dialog keeps the earlier choice");
    }

    #[test]
    fn a_cover_change_in_a_format_without_pictures_cannot_be_saved() {
        let mut e = TagEditor::reading(TrackId(1), 1);
        e.arrived(Some(sheet()));
        e.cover_loaded(Ok(art(2, true)));
        assert!(e.cover_changed());
        assert!(!e.can_save(), "there is no place for it in the file");
    }

    #[test]
    fn the_save_job_carries_the_cover_and_the_thumbnail_size() {
        let mut e = ready_with_cover(Some(art(1, true)));
        e.cover_loaded(Ok(art(2, true)));
        let job = e
            .save_job(Path::new("/x.mp3"), &Limits::default(), 96)
            .unwrap();
        let TagJob::WriteSheet {
            before,
            after,
            thumb_px,
            ..
        } = job
        else {
            panic!("not a sheet write");
        };
        assert_eq!(before.cover(), Some(&art(1, true)));
        assert_eq!(after.cover(), Some(&art(2, true)));
        assert_eq!(thumb_px, 96);
    }

    #[test]
    fn a_cover_the_file_did_not_keep_is_named() {
        let mut e = ready_with_cover(Some(art(1, true)));
        e.cover_loaded(Ok(art(2, true)));
        let mut kept = sheet().with_cover_support(true);
        kept.set_cover(Some(art(2, true)));
        assert!(!e.cover_unstored(Some(&kept)));
        let mut dropped = sheet().with_cover_support(true);
        dropped.set_cover(Some(art(1, true)));
        assert!(e.cover_unstored(Some(&dropped)));
        assert!(!e.cover_unstored(None), "no read-back, no claim");
    }
}
