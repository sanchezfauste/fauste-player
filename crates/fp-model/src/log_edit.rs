//! Editing the Log live (automation rework AR17–AR19), and a template or a
//! date from Settings (AR20–AR22), with the same edits. An edit of a date
//! is a change of that date (AU11–AU12) and leaves the template alone;
//! rows of past dates, the row on air and played rows are locked. An edit
//! of a template changes the template and is never locked.

use chrono::NaiveDate;

use crate::automation::ItemRef;
use crate::command::EngineAction;
use crate::error::ModelError;
use crate::ids::{BlockId, TemplateId};
use crate::local_time::TimeOfDay;
use crate::schedule::{Block, Item};
use crate::state::AppState;

/// What an edit changes: a date's Log, or a template.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum EditTarget {
    Date(NaiveDate),
    Template(TemplateId),
}

/// One edit of a date's Log or a template.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LogEdit {
    /// Items inserted before `index` of the block (at its end, before a
    /// Music fill item, when `index` is past it); a Music fill (rotation)
    /// item replaces the block's.
    Insert {
        block: BlockId,
        index: usize,
        items: Vec<Item>,
    },
    Remove {
        block: BlockId,
        index: usize,
    },
    /// An item moved; `to` is the index before the move.
    Move {
        from: (BlockId, usize),
        to: (BlockId, usize),
    },
    SetHardStart {
        block: BlockId,
        index: usize,
        on: bool,
    },
    AddBlock {
        start: TimeOfDay,
    },
    MoveBlock {
        block: BlockId,
        start: TimeOfDay,
    },
    RemoveBlock {
        block: BlockId,
    },
    /// Back to the template (dates only).
    Revert {
        block: BlockId,
    },
}

/// The rows of `block` on `date` that may not change (AR19): `None` when
/// any may; `Some(n)`: the first `n` are locked (the row on air and those
/// before it); `Some(usize::MAX)`: the whole block (it already played).
fn locked(state: &AppState, date: NaiveDate, block: BlockId) -> Option<usize> {
    let a = state.automation.as_ref()?;
    let on_air = a.on_air.is_some_and(|r| r.date == date && r.block == block);
    if on_air {
        let current = crate::automation::on_air_item(state)
            .filter(|i| i.date == date && i.block == block)
            .map_or(0, |i| i.index + 1);
        return Some(current);
    }
    a.started.contains(&(date, block)).then_some(usize::MAX)
}

/// Whether a row (or, for `index` 0, a block) of `date`'s Log is locked
/// against live edits (AR19).
pub fn is_locked(state: &AppState, date: NaiveDate, block: BlockId, index: usize) -> bool {
    date < state.local_now.date() || check(state, date, block, index).is_err()
}

fn check(
    state: &AppState,
    date: NaiveDate,
    block: BlockId,
    index: usize,
) -> Result<(), ModelError> {
    match locked(state, date, block) {
        Some(n) if index < n => Err(ModelError::LogRowLocked),
        _ => Ok(()),
    }
}

/// The block as it is on the target.
fn block_on(state: &AppState, target: EditTarget, block: BlockId) -> Result<Block, ModelError> {
    match target {
        EditTarget::Date(date) => state
            .schedule
            .day(date)
            .into_iter()
            .find(|b| b.id == block)
            .map(|b| Block {
                id: b.id,
                start: b.start,
                items: b.items,
            }),
        EditTarget::Template(template) => state
            .schedule
            .template(template)
            .and_then(|t| t.block(block))
            .cloned(),
    }
    .ok_or(ModelError::UnknownBlock(block))
}

/// Stores a block on the target: a date change, or the template's block.
fn put(state: &mut AppState, target: EditTarget, block: Block) -> Result<(), ModelError> {
    match target {
        EditTarget::Date(date) => crate::schedule::change_block_on_date(state, date, block),
        EditTarget::Template(template) => crate::schedule::update_block(state, template, block),
    }
}

/// Fails when the row may not change: only a date's rows are ever locked.
fn check_target(
    state: &AppState,
    target: EditTarget,
    block: BlockId,
    index: usize,
) -> Result<(), ModelError> {
    match target {
        EditTarget::Date(date) => check(state, date, block, index),
        EditTarget::Template(_) => Ok(()),
    }
}

/// The blocks an edit changes.
fn blocks_of(edit: &LogEdit) -> Vec<BlockId> {
    match edit {
        LogEdit::Insert { block, .. }
        | LogEdit::Remove { block, .. }
        | LogEdit::SetHardStart { block, .. }
        | LogEdit::MoveBlock { block, .. }
        | LogEdit::RemoveBlock { block }
        | LogEdit::Revert { block } => vec![*block],
        LogEdit::Move { from, to } => vec![from.0, to.0],
        LogEdit::AddBlock { .. } => Vec::new(),
    }
}

/// A template edit to the block on air, while its date follows the
/// template for it: that date first keeps the block as it was loaded (a
/// change of the date, AU12), so the Log's rows still match what plays;
/// the template's change reaches the days after.
fn keep_on_air_as_loaded(
    state: &mut AppState,
    template: TemplateId,
    edit: &LogEdit,
) -> Result<(), ModelError> {
    let Some(run) = state.automation.as_ref().and_then(|a| a.on_air) else {
        return Ok(());
    };
    if !blocks_of(edit).contains(&run.block)
        || state.schedule.template_for(run.date) != Some(template)
    {
        return Ok(());
    }
    let Some(day_block) = state
        .schedule
        .day(run.date)
        .into_iter()
        .find(|b| b.id == run.block && b.origin == crate::schedule::BlockOrigin::Template)
    else {
        return Ok(());
    };
    crate::schedule::change_block_on_date(
        state,
        run.date,
        Block {
            id: day_block.id,
            start: day_block.start,
            items: day_block.items,
        },
    )
}

/// `items` inserted before `index`, a Music fill kept last (AU6).
fn inserted(mut list: Vec<Item>, index: usize, items: Vec<Item>) -> Vec<Item> {
    let (rotations, plain): (Vec<Item>, Vec<Item>) = items
        .into_iter()
        .partition(|i| matches!(i, Item::Rotation { .. }));
    let tail = match list.last() {
        Some(Item::Rotation { .. }) => list.len() - 1,
        _ => list.len(),
    };
    let at = index.min(tail);
    list.splice(at..at, plain);
    if let Some(rotation) = rotations.into_iter().last() {
        list.retain(|i| !matches!(i, Item::Rotation { .. }));
        list.push(rotation);
    }
    list
}

pub(crate) fn apply(
    state: &mut AppState,
    target: EditTarget,
    edit: LogEdit,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    if let EditTarget::Date(date) = target
        && date < state.local_now.date()
    {
        return Err(ModelError::PastDate);
    }
    if let EditTarget::Template(template) = target {
        keep_on_air_as_loaded(state, template, &edit)?;
    }
    let touched: Vec<BlockId> = match edit {
        LogEdit::Insert {
            block,
            index,
            items,
        } => {
            let mut b = block_on(state, target, block)?;
            let at = index.min(b.items.len());
            check_target(state, target, block, at)?;
            b.items = inserted(b.items, at, items);
            put(state, target, b)?;
            vec![block]
        }
        LogEdit::Remove { block, index } => {
            check_target(state, target, block, index)?;
            let mut b = block_on(state, target, block)?;
            if index >= b.items.len() {
                return Err(ModelError::UnknownItem);
            }
            b.items.remove(index);
            put(state, target, b)?;
            vec![block]
        }
        // Onto itself or the row below it: nothing moves.
        LogEdit::Move { from, to } if from.0 == to.0 && (to.1 == from.1 || to.1 == from.1 + 1) => {
            return Ok(());
        }
        LogEdit::Move { from, to } => {
            check_target(state, target, from.0, from.1)?;
            check_target(state, target, to.0, to.1)?;
            let mut source = block_on(state, target, from.0)?;
            if from.1 >= source.items.len() {
                return Err(ModelError::UnknownItem);
            }
            let item = source.items.remove(from.1);
            if from.0 == to.0 {
                let at = if to.1 > from.1 { to.1 - 1 } else { to.1 };
                source.items = inserted(source.items, at, vec![item]);
                put(state, target, source)?;
                vec![from.0]
            } else {
                let mut dest = block_on(state, target, to.0)?;
                dest.items = inserted(dest.items, to.1, vec![item]);
                put(state, target, source)?;
                put(state, target, dest)?;
                vec![from.0, to.0]
            }
        }
        LogEdit::SetHardStart { block, index, on } => {
            check_target(state, target, block, index)?;
            let mut b = block_on(state, target, block)?;
            match b.items.get_mut(index) {
                Some(Item::Track { time_critical, .. } | Item::Category { time_critical, .. }) => {
                    *time_critical = on;
                }
                _ => return Err(ModelError::UnknownItem),
            }
            put(state, target, b)?;
            vec![block]
        }
        LogEdit::AddBlock { start } => {
            match target {
                EditTarget::Date(date) => crate::schedule::add_block_on_date(state, date, start)?,
                EditTarget::Template(template) => {
                    crate::schedule::add_block(state, template, start)?;
                }
            }
            Vec::new()
        }
        LogEdit::MoveBlock { block, start } => {
            check_target(state, target, block, 0)?;
            let mut b = block_on(state, target, block)?;
            b.start = start;
            put(state, target, b)?;
            vec![block]
        }
        LogEdit::RemoveBlock { block } => {
            check_target(state, target, block, 0)?;
            match target {
                EditTarget::Date(date) => {
                    crate::schedule::remove_block_on_date(state, date, block)?;
                }
                EditTarget::Template(template) => {
                    crate::schedule::remove_block(state, template, block)?;
                }
            }
            Vec::new()
        }
        LogEdit::Revert { block } => {
            check_target(state, target, block, 0)?;
            let EditTarget::Date(date) = target else {
                return Err(ModelError::UnknownBlock(block));
            };
            crate::schedule::revert_block_on_date(state, date, block)?;
            vec![block]
        }
    };
    // AR19b: an edit to the block on air takes effect at once, after the
    // row on air. A template edit never reloads (AU12: it reaches a date
    // through the computed day, not the loaded block).
    if let EditTarget::Date(date) = target
        && let Some(item) = crate::automation::on_air_item(state)
        && item.date == date
        && touched.contains(&item.block)
    {
        crate::automation::reload_after(state, item, out);
    }
    Ok(())
}

/// The item after `item` in its block, for tests and the UI.
pub fn after(item: ItemRef) -> ItemRef {
    ItemRef {
        index: item.index + 1,
        ..item
    }
}
