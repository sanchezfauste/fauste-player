//! The schedule (automation spec AU5–AU14): blocks of items at a time of
//! day, day templates, the week plan, and the changes made for one date.

use chrono::Weekday;
use serde::{Deserialize, Serialize};

use crate::error::ModelError;
use crate::ids::{BlockId, CategoryId, RotationId, TemplateId, TrackId};
use crate::local_time::TimeOfDay;
use crate::state::AppState;
use crate::track::TrackKind;

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

/// Everything the schedule stores: the templates, the week plan and the
/// date plans.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Schedule {
    templates: Vec<DayTemplate>,
    week: WeekPlan,
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

    /// The largest raw template or block id in use (0 when empty).
    pub fn max_raw_id(&self) -> u64 {
        self.templates
            .iter()
            .flat_map(|t| std::iter::once(t.id.0).chain(t.blocks.iter().map(|b| b.id.0)))
            .max()
            .unwrap_or(0)
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
