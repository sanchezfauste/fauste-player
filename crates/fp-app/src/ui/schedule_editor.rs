//! The schedule editor (automation spec §8.2): a floating window, opened
//! from the Schedule column, that edits the day templates, the week plan
//! and single dates. Every edit is a model command.

use std::collections::{HashMap, HashSet};

use chrono::Weekday;
use egui::{Align, Layout, RichText, Ui, vec2};
use egui_phosphor::regular as icon;
use fp_model::local_time::TimeOfDay;
use fp_model::schedule::{Block, Item, overrun_secs};
use fp_model::{BlockId, Command, TemplateId};

use super::app::{Scene, ViewState};
use super::format;
use super::theme;
use super::widgets::{font, font_semibold};

/// Which part of the editor shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Tab {
    #[default]
    Templates,
    Week,
    Days,
}

/// What the editor remembers between frames (never saved).
#[derive(Debug, Default)]
pub(crate) struct EditorState {
    pub open: bool,
    pub tab: Tab,
    /// The template being edited (`None` or gone: the first).
    pub template: Option<TemplateId>,
    /// The template name being typed.
    pub name_draft: Option<(TemplateId, String)>,
    /// The block starts being typed, and those that did not parse.
    pub start_drafts: HashMap<BlockId, String>,
    pub bad_starts: HashSet<BlockId>,
}

/// What the operator did to a block.
pub(crate) enum BlockEdit {
    Update(Block),
    Remove,
}

pub(crate) const WEEKDAYS: [(Weekday, &str); 7] = [
    (Weekday::Mon, "weekday-monday"),
    (Weekday::Tue, "weekday-tuesday"),
    (Weekday::Wed, "weekday-wednesday"),
    (Weekday::Thu, "weekday-thursday"),
    (Weekday::Fri, "weekday-friday"),
    (Weekday::Sat, "weekday-saturday"),
    (Weekday::Sun, "weekday-sunday"),
];

/// Draws the editor while it is open (never on a phone).
pub(crate) fn draw(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState) {
    if !view_state.schedule_editor.open || view_state.phone_active {
        return;
    }
    let t = scene.i18n;
    let mut open = true;
    egui::Window::new(t.tr("schedule-editor-title"))
        .id(egui::Id::unique("schedule-editor"))
        .open(&mut open)
        .default_size(vec2(720.0, 560.0))
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let editor = &mut view_state.schedule_editor;
                for (tab, key) in [
                    (Tab::Templates, "schedule-editor-templates"),
                    (Tab::Week, "schedule-editor-week"),
                    (Tab::Days, "schedule-editor-days"),
                ] {
                    ui.selectable_value(&mut editor.tab, tab, t.tr(key));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t.tr("schedule-editor-rotations")).clicked() {
                        view_state.open_automation_settings = true;
                    }
                });
            });
            ui.separator();
            match view_state.schedule_editor.tab {
                Tab::Templates => templates(ui, scene, &mut view_state.schedule_editor),
                Tab::Week => week(ui, scene),
                Tab::Days => {}
            }
        });
    if !open {
        view_state.schedule_editor.open = false;
    }
}

/// The template's name, or "?" when it is gone.
pub(crate) fn template_name(scene: &Scene<'_>, id: TemplateId) -> String {
    scene
        .state
        .schedule
        .template(id)
        .map_or_else(|| "?".to_owned(), |t| t.name.clone())
}

/// The week plan (AU9): each weekday's template.
fn week(ui: &mut egui::Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    let schedule = &scene.state.schedule;
    ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
    for (weekday, key) in WEEKDAYS {
        let current = schedule.week().get(weekday);
        let mut choice = current;
        egui::ComboBox::from_label(RichText::new(t.tr(key)).font(font(13.0)).color(theme::TEXT))
            .width(200.0)
            .selected_text(current.map(|c| template_name(scene, c)).unwrap_or_default())
            .show_ui(ui, |ui| {
                for template in schedule.templates() {
                    ui.selectable_value(&mut choice, Some(template.id), template.name.as_str());
                }
            });
        if let Some(template) = choice
            && choice != current
        {
            scene
                .ctl
                .send(Command::SetWeekTemplate { weekday, template });
        }
    }
}

/// The day templates (AU9): the list on the left, the selected one's
/// blocks on the right.
fn templates(ui: &mut Ui, scene: &Scene<'_>, editor: &mut EditorState) {
    let t = scene.i18n;
    let schedule = &scene.state.schedule;
    let selected = editor
        .template
        .filter(|id| schedule.template(*id).is_some())
        .or_else(|| schedule.templates().first().map(|t| t.id));
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(190.0);
            for template in schedule.templates() {
                if ui
                    .selectable_label(selected == Some(template.id), template.name.as_str())
                    .clicked()
                {
                    editor.template = Some(template.id);
                    editor.name_draft = None;
                }
            }
            ui.add_space(8.0);
            if ui.button(t.tr("schedule-template-new")).clicked() {
                scene.ctl.send(Command::CreateTemplate {
                    name: t.tr("schedule-template-new"),
                });
            }
            if let Some(id) = selected {
                if ui.button(t.tr("schedule-template-duplicate")).clicked() {
                    let name = t.tr_args(
                        "schedule-template-copy-name",
                        &[("name", template_name(scene, id).into())],
                    );
                    scene
                        .ctl
                        .send(Command::DuplicateTemplate { template: id, name });
                }
                if ui.button(t.tr("schedule-template-delete")).clicked() {
                    scene.ctl.send(Command::DeleteTemplate(id));
                }
                ui.add_space(8.0);
                rename(ui, scene, editor, id);
            }
        });
        ui.separator();
        if let Some(id) = selected {
            egui::ScrollArea::vertical()
                .id_salt("schedule-template-blocks")
                .auto_shrink([false, false])
                .show(ui, |ui| template_blocks(ui, scene, editor, id));
        }
    });
}

/// The template's name, sent when the field loses focus.
fn rename(ui: &mut Ui, scene: &Scene<'_>, editor: &mut EditorState, id: TemplateId) {
    let t = scene.i18n;
    let current = template_name(scene, id);
    let draft = match &mut editor.name_draft {
        Some((d, text)) if *d == id => text,
        slot => &mut slot.insert((id, current.clone())).1,
    };
    let label = ui.label(RichText::new(t.tr("schedule-template-name")).font(font(12.0)));
    let response = ui
        .add(egui::TextEdit::singleline(draft).desired_width(180.0))
        .labelled_by(label.id);
    if response.lost_focus() {
        let name = draft.trim().to_owned();
        if !name.is_empty() && name != current {
            scene
                .ctl
                .send(Command::RenameTemplate { template: id, name });
        }
        editor.name_draft = None;
    } else if !response.has_focus() {
        editor.name_draft = None;
    }
}

/// The template's blocks, each editable, and Add block.
fn template_blocks(ui: &mut Ui, scene: &Scene<'_>, editor: &mut EditorState, id: TemplateId) {
    let t = scene.i18n;
    let Some(template) = scene.state.schedule.template(id) else {
        return;
    };
    let mut blocks: Vec<&Block> = template.blocks.iter().collect();
    blocks.sort_by_key(|b| b.start);
    for (i, block) in blocks.iter().enumerate() {
        let next = blocks.get(i + 1).map(|b| b.start);
        match block_editor(ui, scene, editor, block, next) {
            Some(BlockEdit::Update(block)) => {
                scene.ctl.send(Command::UpdateBlock {
                    template: id,
                    block,
                });
            }
            Some(BlockEdit::Remove) => {
                scene.ctl.send(Command::RemoveBlock {
                    template: id,
                    block: block.id,
                });
            }
            None => {}
        }
    }
    ui.add_space(8.0);
    if ui.button(t.tr("schedule-block-add")).clicked()
        && let Some(start) = free_start(blocks.iter().map(|b| b.start))
    {
        scene.ctl.send(Command::AddBlock {
            template: id,
            start,
        });
    }
}

/// A whole hour after the last block, or the first free whole hour.
pub(crate) fn free_start(starts: impl Iterator<Item = TimeOfDay>) -> Option<TimeOfDay> {
    let taken: Vec<TimeOfDay> = starts.collect();
    let after_last = taken
        .iter()
        .max()
        .and_then(|last| TimeOfDay::hms(last.secs() / 3600 + 1, 0, 0));
    after_last.filter(|s| !taken.contains(s)).or_else(|| {
        (0..24)
            .filter_map(|h| TimeOfDay::hms(h, 0, 0))
            .find(|s| !taken.contains(s))
    })
}

/// One block: its start, AU37's overrun, its items and Remove block.
pub(crate) fn block_editor(
    ui: &mut Ui,
    scene: &Scene<'_>,
    editor: &mut EditorState,
    block: &Block,
    next_start: Option<TimeOfDay>,
) -> Option<BlockEdit> {
    let t = scene.i18n;
    let mut edit = None;
    egui::Frame::new()
        .fill(theme::NEUTRAL_900)
        .inner_margin(egui::Margin::same(8))
        .corner_radius(4.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                if let Some(start) = start_field(ui, scene, editor, block) {
                    edit = Some(BlockEdit::Update(Block {
                        start,
                        ..block.clone()
                    }));
                }
                if let Some(excess) =
                    overrun_secs(scene.state, &block.items, block.start, next_start)
                {
                    ui.label(
                        RichText::new(t.tr_args(
                            "schedule-editor-overrun",
                            &[("time", format::clock(excess.ceil()).into())],
                        ))
                        .font(font_semibold(12.0))
                        .color(theme::AMBER),
                    );
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button(t.tr("schedule-block-remove")).clicked() {
                        edit = Some(BlockEdit::Remove);
                    }
                });
            });
            if editor.bad_starts.contains(&block.id) {
                ui.label(
                    RichText::new(t.tr("schedule-editor-bad-time"))
                        .font(font(11.0))
                        .color(theme::AMBER),
                );
            }
            for (index, item) in block.items.iter().enumerate() {
                if let Some(items) = item_row(ui, scene, block, index, item) {
                    edit = Some(BlockEdit::Update(Block {
                        items,
                        ..block.clone()
                    }));
                }
            }
        });
    ui.add_space(6.0);
    edit
}

/// The block's start as `HH:MM:SS`, sent when the field loses focus and
/// parses; a bad value stays in the field with a note.
fn start_field(
    ui: &mut Ui,
    scene: &Scene<'_>,
    editor: &mut EditorState,
    block: &Block,
) -> Option<TimeOfDay> {
    let t = scene.i18n;
    let current = block.start.to_string();
    let draft = editor
        .start_drafts
        .entry(block.id)
        .or_insert_with(|| current.clone());
    let label = ui.label(RichText::new(t.tr("schedule-block-start")).font(font(12.0)));
    let response = ui
        .add(
            egui::TextEdit::singleline(draft)
                .desired_width(80.0)
                .font(egui::TextStyle::Monospace),
        )
        .labelled_by(label.id);
    if response.lost_focus() {
        let typed = draft.clone();
        return match TimeOfDay::parse(&typed) {
            Some(start) => {
                editor.start_drafts.remove(&block.id);
                editor.bad_starts.remove(&block.id);
                (start != block.start).then_some(start)
            }
            None => {
                editor.bad_starts.insert(block.id);
                None
            }
        };
    }
    if !response.has_focus() && !editor.bad_starts.contains(&block.id) {
        editor.start_drafts.insert(block.id, current);
    }
    None
}

/// One item: its text and length, Time-critical (track and category
/// items), Up, Down and Remove. Returns the block's new items.
fn item_row(
    ui: &mut Ui,
    scene: &Scene<'_>,
    block: &Block,
    index: usize,
    item: &Item,
) -> Option<Vec<Item>> {
    let t = scene.i18n;
    let mut items = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
        let text = super::schedule::item_text(scene, item);
        ui.add(
            egui::Label::new(RichText::new(text).font(font(12.0)).color(theme::TEXT)).truncate(),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let moved = |ui: &mut Ui, glyph: &str, key: &str, to: Option<usize>| {
                let tip = t.tr(key);
                let response = ui
                    .add_enabled(to.is_some(), egui::Button::new(glyph).small())
                    .on_hover_text(&tip);
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Button, to.is_some(), &tip)
                });
                response.clicked().then_some(to).flatten()
            };
            let last = block.items.len().saturating_sub(1);
            if let Some(Some(_)) = Some(moved(ui, icon::X, "schedule-item-remove", Some(index))) {
                let mut new = block.items.clone();
                new.remove(index);
                items = Some(new);
            }
            if let Some(to) = moved(
                ui,
                icon::ARROW_DOWN,
                "schedule-item-down",
                (index < last).then_some(index + 1),
            ) {
                let mut new = block.items.clone();
                new.swap(index, to);
                items = Some(new);
            }
            if let Some(to) = moved(ui, icon::ARROW_UP, "schedule-item-up", index.checked_sub(1)) {
                let mut new = block.items.clone();
                new.swap(index, to);
                items = Some(new);
            }
            let flag = match item {
                Item::Track { time_critical, .. } | Item::Category { time_critical, .. } => {
                    Some(*time_critical)
                }
                Item::Rotation { .. } => None,
            };
            if let Some(mut on) = flag
                && ui
                    .checkbox(&mut on, t.tr("schedule-item-time-critical-check"))
                    .changed()
            {
                let mut new = block.items.clone();
                if let Some(
                    Item::Track { time_critical, .. } | Item::Category { time_critical, .. },
                ) = new.get_mut(index)
                {
                    *time_critical = on;
                }
                items = Some(new);
            }
            if let Item::Track { track, .. } = item
                && let Some(secs) = scene
                    .state
                    .library
                    .get(*track)
                    .map(|tr| tr.play_range(scene.state.config.players.use_cue_markers))
                    .and_then(|r| r.known_end().map(|_| r.length()))
            {
                ui.label(
                    RichText::new(format::clock(secs))
                        .font(font(12.0))
                        .color(theme::NEUTRAL_400),
                );
            }
        });
    });
    items
}
