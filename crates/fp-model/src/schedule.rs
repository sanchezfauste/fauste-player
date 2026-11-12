//! The schedule (automation spec AU5–AU14): blocks of items at a time of
//! day, day templates, the week plan, and the changes made for one date.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

use crate::error::ModelError;
use crate::ids::{BlockId, CategoryId, RotationId, TemplateId, TrackId};
use crate::local_time::{LocalZone, TimeOfDay};
use crate::state::AppState;
use crate::track::TrackKind;
use crate::wall_time::WallTime;

/// One thing in a block (AU6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Item {
    /// A track of the library, any kind.
    Track {
        track: TrackId,
        /// Marked time-critical by the operator (AU7).
        #[serde(default)]
        time_critical: bool,
    },
    /// One track of a category, chosen each time the item airs.
    Category {
        category: CategoryId,
        #[serde(default)]
        time_critical: bool,
    },
    /// From here to the end of the block, the music filler plays this
    /// rotation. It is always the block's last item.
    Rotation { rotation: RotationId },
}

impl Item {
    /// AU7: an Hourly ID track, a category of Hourly ID kind, or an item the
    /// operator marked. An unknown track or category is not.
    pub fn is_time_critical(&self, state: &AppState) -> bool {
        match self {
            Item::Track {
                track,
                time_critical,
            } => {
                *time_critical
                    || state
                        .library
                        .get(*track)
                        .is_some_and(|t| t.kind == TrackKind::HourlyId)
            }
            Item::Category {
                category,
                time_critical,
            } => {
                *time_critical
                    || state
                        .categories
                        .get(*category)
                        .is_some_and(|c| c.kind == TrackKind::HourlyId)
            }
            Item::Rotation { .. } => false,
        }
    }
}

/// A group of items with a start time (AU5). It lasts until the next
/// block's start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub start: TimeOfDay,
    #[serde(default)]
    pub items: Vec<Item>,
}

/// AU6: a rotation item, if any, is the last item.
pub fn check_items(items: &[Item]) -> Result<(), ModelError> {
    let rotation_at = items
        .iter()
        .position(|i| matches!(i, Item::Rotation { .. }));
    match rotation_at {
        Some(at) if at + 1 != items.len() => Err(ModelError::RotationNotLast),
        _ => Ok(()),
    }
}

/// A named list of blocks (AU9), sorted by start, no two at one start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayTemplate {
    pub id: TemplateId,
    pub name: String,
    #[serde(default)]
    pub blocks: Vec<Block>,
}

impl DayTemplate {
    pub fn block(&self, id: BlockId) -> Option<&Block> {
        self.blocks.iter().find(|b| b.id == id)
    }
}

/// The template of each weekday, Monday first (AU9).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeekPlan(pub [Option<TemplateId>; 7]);

impl WeekPlan {
    pub fn get(&self, day: Weekday) -> Option<TemplateId> {
        self.0
            .get(day.num_days_from_monday() as usize)
            .copied()
            .flatten()
    }

    fn set(&mut self, day: Weekday, template: TemplateId) {
        if let Some(slot) = self.0.get_mut(day.num_days_from_monday() as usize) {
            *slot = Some(template);
        }
    }

    fn uses(&self, template: TemplateId) -> bool {
        self.0.contains(&Some(template))
    }
}

/// One change a date makes to its template's blocks (AU11, AU12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateChange {
    /// The template block with `block.id` has this content on the date.
    /// `own_time`: its start was set on the date; otherwise it follows the
    /// template block's. The start is kept either way, for when the
    /// template block is deleted and this becomes a block of the date.
    Changed { block: Block, own_time: bool },
    /// The template block with this id does not play on the date.
    Removed(BlockId),
    /// A block of this date alone.
    Added(Block),
}

impl DateChange {
    fn block_id(&self) -> BlockId {
        match self {
            DateChange::Changed { block, .. } | DateChange::Added(block) => block.id,
            DateChange::Removed(id) => *id,
        }
    }
}

/// What one date changes: another template (AU10) and block changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatePlan {
    pub date: NaiveDate,
    #[serde(default)]
    pub template: Option<TemplateId>,
    #[serde(default)]
    pub changes: Vec<DateChange>,
}

impl DatePlan {
    fn is_empty(&self) -> bool {
        self.template.is_none() && self.changes.is_empty()
    }
}

/// Where a block of a computed day comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockOrigin {
    /// The template's block, unchanged.
    Template,
    /// A template block changed for this date.
    Changed,
    /// A block of this date alone: added on it, or changed on it when its
    /// template block no longer exists.
    DateOnly,
}

/// A block of a date's computed schedule (AU13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayBlock {
    pub id: BlockId,
    pub start: TimeOfDay,
    pub items: Vec<Item>,
    pub origin: BlockOrigin,
}

/// Everything the schedule stores: the templates, the week plan and the
/// date plans.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schedule {
    templates: Vec<DayTemplate>,
    week: WeekPlan,
    dates: BTreeMap<NaiveDate, DatePlan>,
}

impl Schedule {
    pub fn templates(&self) -> &[DayTemplate] {
        &self.templates
    }

    pub fn template(&self, id: TemplateId) -> Option<&DayTemplate> {
        self.templates.iter().find(|t| t.id == id)
    }

    pub fn week(&self) -> &WeekPlan {
        &self.week
    }

    /// The schedule as loaded, repaired (rule 9), with one warning per
    /// repair: blocks out of the day or at a start taken by an earlier
    /// block are dropped; items after a rotation are dropped; a weekday with
    /// no known template gets the first; a date naming a missing template
    /// follows its weekday.
    pub fn from_parts(
        templates: Vec<DayTemplate>,
        week: WeekPlan,
        dates: Vec<DatePlan>,
    ) -> (Schedule, Vec<String>) {
        let mut warnings = Vec::new();
        let templates: Vec<DayTemplate> = templates
            .into_iter()
            .map(|mut t| {
                let mut blocks: Vec<Block> = Vec::with_capacity(t.blocks.len());
                for mut b in std::mem::take(&mut t.blocks) {
                    let what = format!("schedule: template \"{}\", block {}", t.name, b.id.0);
                    if !repaired_block(&mut b, &what, &mut warnings) {
                        continue;
                    }
                    if blocks.iter().any(|o| o.start == b.start) {
                        warnings.push(format!(
                            "{what}: another block starts at {}; dropped",
                            b.start
                        ));
                        continue;
                    }
                    blocks.push(b);
                }
                blocks.sort_by_key(|b| b.start);
                t.blocks = blocks;
                t
            })
            .collect();
        let known = |id: TemplateId| templates.iter().any(|t| t.id == id);
        let first = templates.first().map(|t| t.id);
        let mut week = week;
        for (day, slot) in week.0.iter_mut().enumerate() {
            if slot.is_some_and(known) {
                continue;
            }
            if first.is_some() {
                warnings.push(format!(
                    "schedule: weekday {} had no known template; it now uses the first",
                    day + 1
                ));
            }
            *slot = first;
        }
        let mut map = BTreeMap::new();
        for mut plan in dates {
            if plan.template.is_some_and(|t| !known(t)) {
                warnings.push(format!(
                    "schedule: {} named a missing template; it follows its weekday",
                    plan.date
                ));
                plan.template = None;
            }
            let what = format!("schedule: {}", plan.date);
            plan.changes.retain_mut(|c| match c {
                DateChange::Changed { block, .. } | DateChange::Added(block) => {
                    repaired_block(block, &what, &mut warnings)
                }
                DateChange::Removed(_) => true,
            });
            if !plan.is_empty() {
                map.insert(plan.date, plan);
            }
        }
        let schedule = Schedule {
            templates,
            week,
            dates: map,
        };
        (schedule, warnings)
    }

    pub fn into_parts(self) -> (Vec<DayTemplate>, WeekPlan, Vec<DatePlan>) {
        (
            self.templates,
            self.week,
            self.dates.into_values().collect(),
        )
    }

    /// The largest raw template or block id in use (0 when empty).
    pub fn max_raw_id(&self) -> u64 {
        let templates = self
            .templates
            .iter()
            .flat_map(|t| std::iter::once(t.id.0).chain(t.blocks.iter().map(|b| b.id.0)));
        let dates = self
            .dates
            .values()
            .flat_map(|p| p.changes.iter().map(|c| c.block_id().0));
        templates.chain(dates).max().unwrap_or(0)
    }

    pub fn date_plan(&self, date: NaiveDate) -> Option<&DatePlan> {
        self.dates.get(&date)
    }

    pub fn date_plans(&self) -> impl Iterator<Item = &DatePlan> {
        self.dates.values()
    }

    /// AU10: the date's own template if it still exists, else its weekday's.
    pub fn template_for(&self, date: NaiveDate) -> Option<TemplateId> {
        self.dates
            .get(&date)
            .and_then(|p| p.template)
            .filter(|t| self.template(*t).is_some())
            .or_else(|| self.week.get(date.weekday()))
    }

    /// AU13: the date's blocks, its template with its changes, in start
    /// order with no two at one start. When two share a start, a block of
    /// the date (changed or its own) wins over the template's; between two
    /// of the same kind, the lower id.
    pub fn day(&self, date: NaiveDate) -> Vec<DayBlock> {
        let template = self.template_for(date).and_then(|t| self.template(t));
        let changes = self
            .dates
            .get(&date)
            .map(|p| p.changes.as_slice())
            .unwrap_or_default();
        let mut day: Vec<DayBlock> = Vec::new();
        for b in template.map(|t| t.blocks.as_slice()).unwrap_or_default() {
            let change = changes.iter().find(|c| c.block_id() == b.id);
            match change {
                Some(DateChange::Removed(_)) => {}
                Some(DateChange::Changed { block, own_time }) => day.push(DayBlock {
                    id: b.id,
                    start: if *own_time { block.start } else { b.start },
                    items: block.items.clone(),
                    origin: BlockOrigin::Changed,
                }),
                _ => day.push(DayBlock {
                    id: b.id,
                    start: b.start,
                    items: b.items.clone(),
                    origin: BlockOrigin::Template,
                }),
            }
        }
        for c in changes {
            let own = match c {
                DateChange::Added(block) => Some(block),
                DateChange::Changed { block, .. }
                    if template.and_then(|t| t.block(block.id)).is_none() =>
                {
                    Some(block)
                }
                _ => None,
            };
            if let Some(block) = own {
                day.push(DayBlock {
                    id: block.id,
                    start: block.start,
                    items: block.items.clone(),
                    origin: BlockOrigin::DateOnly,
                });
            }
        }
        day.sort_by_key(|b| (b.start, b.origin == BlockOrigin::Template, b.id));
        day.dedup_by_key(|b| b.start);
        day
    }

    /// The date's plan, created empty when it has none.
    fn plan_mut(&mut self, date: NaiveDate) -> &mut DatePlan {
        self.dates.entry(date).or_insert_with(|| DatePlan {
            date,
            template: None,
            changes: Vec::new(),
        })
    }

    /// Drops the date's plan when it no longer changes anything.
    fn settle(&mut self, date: NaiveDate) {
        if self.dates.get(&date).is_some_and(DatePlan::is_empty) {
            self.dates.remove(&date);
        }
    }

    fn template_mut(&mut self, id: TemplateId) -> Result<&mut DayTemplate, ModelError> {
        self.templates
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or(ModelError::UnknownTemplate(id))
    }
}

fn checked_name(name: &str) -> Result<String, ModelError> {
    let name = name.trim();
    if name.is_empty() {
        Err(ModelError::EmptyTemplateName)
    } else {
        Ok(name.to_owned())
    }
}

/// Puts `block` in `blocks` in start order; another block at its start
/// (other than the one it replaces) refuses it.
fn place(blocks: &mut Vec<Block>, block: Block) -> Result<(), ModelError> {
    if blocks
        .iter()
        .any(|b| b.start == block.start && b.id != block.id)
    {
        return Err(ModelError::BlockTimeTaken);
    }
    blocks.retain(|b| b.id != block.id);
    let at = blocks.partition_point(|b| b.start < block.start);
    blocks.insert(at, block);
    Ok(())
}

/// Repairs a loaded block: false when it must be dropped (a start out of
/// the day); items after its first rotation are dropped.
fn repaired_block(block: &mut Block, what: &str, warnings: &mut Vec<String>) -> bool {
    if TimeOfDay::new(block.start.secs()).is_none() {
        warnings.push(format!("{what}: its start is out of the day; dropped"));
        return false;
    }
    if check_items(&block.items).is_err() {
        if let Some(at) = block
            .items
            .iter()
            .position(|i| matches!(i, Item::Rotation { .. }))
        {
            block.items.truncate(at + 1);
        }
        warnings.push(format!("{what}: items after its rotation were dropped"));
    }
    true
}

/// AU9: with no template at all, adds `name` (no blocks) for all seven days.
pub fn ensure_default(state: &mut AppState, name: &str) {
    if state.schedule.templates.is_empty() {
        let id = state.ids.template();
        state.schedule.templates.push(DayTemplate {
            id,
            name: name.to_owned(),
            blocks: Vec::new(),
        });
        state.schedule.week = WeekPlan([Some(id); 7]);
    }
}

pub(crate) fn create_template(state: &mut AppState, name: &str) -> Result<(), ModelError> {
    let name = checked_name(name)?;
    let id = state.ids.template();
    state.schedule.templates.push(DayTemplate {
        id,
        name,
        blocks: Vec::new(),
    });
    Ok(())
}

pub(crate) fn duplicate_template(
    state: &mut AppState,
    template: TemplateId,
    name: &str,
) -> Result<(), ModelError> {
    let name = checked_name(name)?;
    let source = state
        .schedule
        .template(template)
        .ok_or(ModelError::UnknownTemplate(template))?
        .blocks
        .clone();
    let id = state.ids.template();
    let blocks = source
        .into_iter()
        .map(|b| Block {
            id: state.ids.block(),
            ..b
        })
        .collect();
    state
        .schedule
        .templates
        .push(DayTemplate { id, name, blocks });
    Ok(())
}

pub(crate) fn rename_template(
    state: &mut AppState,
    template: TemplateId,
    name: &str,
) -> Result<(), ModelError> {
    let name = checked_name(name)?;
    state.schedule.template_mut(template)?.name = name;
    Ok(())
}

pub(crate) fn delete_template(
    state: &mut AppState,
    template: TemplateId,
) -> Result<(), ModelError> {
    if state.schedule.template(template).is_none() {
        return Err(ModelError::UnknownTemplate(template));
    }
    if state.schedule.templates.len() == 1 {
        return Err(ModelError::LastTemplate);
    }
    if state.schedule.week.uses(template) {
        return Err(ModelError::TemplateInUse);
    }
    state.schedule.templates.retain(|t| t.id != template);
    for plan in state.schedule.dates.values_mut() {
        if plan.template == Some(template) {
            plan.template = None;
        }
    }
    state.schedule.dates.retain(|_, p| !p.is_empty());
    Ok(())
}

pub(crate) fn add_block(
    state: &mut AppState,
    template: TemplateId,
    start: TimeOfDay,
) -> Result<(), ModelError> {
    let id = state.ids.block();
    let t = state.schedule.template_mut(template)?;
    place(
        &mut t.blocks,
        Block {
            id,
            start,
            items: Vec::new(),
        },
    )
}

pub(crate) fn update_block(
    state: &mut AppState,
    template: TemplateId,
    block: Block,
) -> Result<(), ModelError> {
    check_items(&block.items)?;
    let t = state.schedule.template_mut(template)?;
    if t.block(block.id).is_none() {
        return Err(ModelError::UnknownBlock(block.id));
    }
    place(&mut t.blocks, block)
}

pub(crate) fn remove_block(
    state: &mut AppState,
    template: TemplateId,
    block: BlockId,
) -> Result<(), ModelError> {
    let t = state.schedule.template_mut(template)?;
    if t.block(block).is_none() {
        return Err(ModelError::UnknownBlock(block));
    }
    t.blocks.retain(|b| b.id != block);
    Ok(())
}

pub(crate) fn set_week_template(
    state: &mut AppState,
    weekday: Weekday,
    template: TemplateId,
) -> Result<(), ModelError> {
    if state.schedule.template(template).is_none() {
        return Err(ModelError::UnknownTemplate(template));
    }
    state.schedule.week.set(weekday, template);
    Ok(())
}

pub(crate) fn set_date_template(
    state: &mut AppState,
    date: NaiveDate,
    template: Option<TemplateId>,
) -> Result<(), ModelError> {
    if let Some(t) = template
        && state.schedule.template(t).is_none()
    {
        return Err(ModelError::UnknownTemplate(t));
    }
    state.schedule.plan_mut(date).template = template;
    state.schedule.settle(date);
    Ok(())
}

/// A block the date commands may name: one of the date's computed day, or
/// one the date's plan holds (a block of the date can be hidden from the
/// day by another at the same start, after a template block moved there).
fn known_on(schedule: &Schedule, date: NaiveDate, block: BlockId) -> bool {
    let in_plan = schedule.dates.get(&date).is_some_and(|p| {
        p.changes.iter().any(|c| {
            matches!(c, DateChange::Changed { .. } | DateChange::Added(_)) && c.block_id() == block
        })
    });
    in_plan || schedule.day(date).iter().any(|b| b.id == block)
}

/// A start no other block of the date's computed day has.
fn free_on(
    schedule: &Schedule,
    date: NaiveDate,
    start: TimeOfDay,
    own: Option<BlockId>,
) -> Result<(), ModelError> {
    let taken = schedule
        .day(date)
        .iter()
        .any(|b| b.start == start && Some(b.id) != own);
    if taken {
        Err(ModelError::BlockTimeTaken)
    } else {
        Ok(())
    }
}

pub(crate) fn change_block_on_date(
    state: &mut AppState,
    date: NaiveDate,
    block: Block,
) -> Result<(), ModelError> {
    check_items(&block.items)?;
    let schedule = &mut state.schedule;
    if !known_on(schedule, date, block.id) {
        return Err(ModelError::UnknownBlock(block.id));
    }
    free_on(schedule, date, block.start, Some(block.id))?;
    let template_start = schedule
        .template_for(date)
        .and_then(|t| schedule.template(t))
        .and_then(|t| t.block(block.id))
        .map(|b| b.start);
    let plan = schedule.plan_mut(date);
    if let Some(added) = plan.changes.iter_mut().find_map(|c| match c {
        DateChange::Added(b) if b.id == block.id => Some(b),
        _ => None,
    }) {
        *added = block;
        return Ok(());
    }
    let own_time = template_start != Some(block.start);
    plan.changes.retain(|c| c.block_id() != block.id);
    plan.changes.push(DateChange::Changed { block, own_time });
    Ok(())
}

pub(crate) fn add_block_on_date(
    state: &mut AppState,
    date: NaiveDate,
    start: TimeOfDay,
) -> Result<(), ModelError> {
    free_on(&state.schedule, date, start, None)?;
    let id = state.ids.block();
    state
        .schedule
        .plan_mut(date)
        .changes
        .push(DateChange::Added(Block {
            id,
            start,
            items: Vec::new(),
        }));
    Ok(())
}

pub(crate) fn remove_block_on_date(
    state: &mut AppState,
    date: NaiveDate,
    block: BlockId,
) -> Result<(), ModelError> {
    let schedule = &mut state.schedule;
    if !known_on(schedule, date, block) {
        return Err(ModelError::UnknownBlock(block));
    }
    let in_template = schedule
        .template_for(date)
        .and_then(|t| schedule.template(t))
        .is_some_and(|t| t.block(block).is_some());
    let plan = schedule.plan_mut(date);
    plan.changes.retain(|c| c.block_id() != block);
    if in_template {
        plan.changes.push(DateChange::Removed(block));
    }
    schedule.settle(date);
    Ok(())
}

pub(crate) fn revert_block_on_date(
    state: &mut AppState,
    date: NaiveDate,
    block: BlockId,
) -> Result<(), ModelError> {
    let schedule = &mut state.schedule;
    let found = schedule.dates.get(&date).is_some_and(|p| {
        p.changes.iter().any(|c| {
            matches!(c, DateChange::Changed { .. } | DateChange::Removed(_))
                && c.block_id() == block
        })
    });
    if !found {
        return Err(ModelError::UnknownBlock(block));
    }
    schedule
        .plan_mut(date)
        .changes
        .retain(|c| matches!(c, DateChange::Added(_)) || c.block_id() != block);
    schedule.settle(date);
    Ok(())
}

/// AU14: when a block at `start` on `date` begins; `None` when that local
/// time does not exist on the date (the spring change), the first moment
/// when it occurs twice (the autumn change).
pub fn block_start(zone: &impl LocalZone, date: NaiveDate, start: TimeOfDay) -> Option<WallTime> {
    let time = chrono::NaiveTime::from_num_seconds_from_midnight_opt(start.secs(), 0)?;
    zone.earliest(date.and_time(time))
}

/// AU38: deletes the date plans dated more than
/// `automation.overrides_keep_days` before `today`; returns how many.
pub fn prune_dates(state: &mut AppState, today: NaiveDate) -> usize {
    let keep = u64::from(state.config.automation.overrides_keep_days);
    let Some(oldest) = today.checked_sub_days(chrono::Days::new(keep)) else {
        return 0;
    };
    let before = state.schedule.dates.len();
    state.schedule.dates.retain(|d, _| *d >= oldest);
    before - state.schedule.dates.len()
}

/// AU37: by how many seconds `items` overrun the time from `start` to
/// `next_start` (the next block's start; `None`: midnight; at or before
/// `start`: the next day's). A track counts its play range when known, a
/// category its shortest playable track, a rotation nothing. `None` when
/// the block fits.
pub fn overrun_secs(
    state: &AppState,
    items: &[Item],
    start: TimeOfDay,
    next_start: Option<TimeOfDay>,
) -> Option<f64> {
    let use_markers = state.config.players.use_cue_markers;
    let length = |track: &crate::track::Track| {
        let range = track.play_range(use_markers);
        range.known_end().map(|_| range.length())
    };
    let total: f64 = items
        .iter()
        .filter_map(|item| match item {
            Item::Track { track, .. } => state.library.get(*track).and_then(length),
            Item::Category { category, .. } => {
                let tree: std::collections::HashSet<_> =
                    state.categories.subtree(*category).into_iter().collect();
                state
                    .library
                    .iter()
                    .filter(|t| t.category.is_some_and(|c| tree.contains(&c)))
                    .filter(|t| t.file_state.is_playable())
                    .filter_map(length)
                    .min_by(f64::total_cmp)
            }
            Item::Rotation { .. } => None,
        })
        .sum();
    let from = f64::from(start.secs());
    let to = match next_start {
        Some(next) if next.secs() > start.secs() => f64::from(next.secs()),
        Some(next) => f64::from(next.secs()) + f64::from(crate::local_time::SECS_PER_DAY),
        None => f64::from(crate::local_time::SECS_PER_DAY),
    };
    let excess = total - (to - from);
    (excess > 0.0).then_some(excess)
}
