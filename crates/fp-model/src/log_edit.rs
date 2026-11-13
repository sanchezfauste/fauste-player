//! Editing the Log live (automation rework AR17–AR19): every edit is a
//! change of its date (AU11–AU12); the template is never touched. Rows of
//! past dates, the row on air and played rows are locked.

use chrono::NaiveDate;

use crate::automation::ItemRef;
use crate::command::EngineAction;
use crate::error::ModelError;
use crate::ids::BlockId;
use crate::local_time::TimeOfDay;
use crate::schedule::{Block, Item};
use crate::state::AppState;

/// One live edit of a date's Log.
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
    /// Back to the template.
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

/// The block as it is on `date`.
fn block_on(state: &AppState, date: NaiveDate, block: BlockId) -> Result<Block, ModelError> {
    state
        .schedule
        .day(date)
        .into_iter()
        .find(|b| b.id == block)
        .map(|b| Block {
            id: b.id,
            start: b.start,
            items: b.items,
        })
        .ok_or(ModelError::UnknownBlock(block))
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
    date: NaiveDate,
    edit: LogEdit,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    if date < state.local_now.date() {
        return Err(ModelError::PastDate);
    }
    let touched: Vec<BlockId> = match edit {
        LogEdit::Insert {
            block,
            index,
            items,
        } => {
            let mut b = block_on(state, date, block)?;
            let at = index.min(b.items.len());
            check(state, date, block, at)?;
            b.items = inserted(b.items, at, items);
            crate::schedule::change_block_on_date(state, date, b)?;
            vec![block]
        }
        LogEdit::Remove { block, index } => {
            check(state, date, block, index)?;
            let mut b = block_on(state, date, block)?;
            if index >= b.items.len() {
                return Err(ModelError::UnknownItem);
            }
            b.items.remove(index);
            crate::schedule::change_block_on_date(state, date, b)?;
            vec![block]
        }
        // Onto itself or the row below it: nothing moves.
        LogEdit::Move { from, to } if from.0 == to.0 && (to.1 == from.1 || to.1 == from.1 + 1) => {
            return Ok(());
        }
        LogEdit::Move { from, to } => {
            check(state, date, from.0, from.1)?;
            check(state, date, to.0, to.1)?;
            let mut source = block_on(state, date, from.0)?;
            if from.1 >= source.items.len() {
                return Err(ModelError::UnknownItem);
            }
            let item = source.items.remove(from.1);
            if from.0 == to.0 {
                let at = if to.1 > from.1 { to.1 - 1 } else { to.1 };
                source.items = inserted(source.items, at, vec![item]);
                crate::schedule::change_block_on_date(state, date, source)?;
                vec![from.0]
            } else {
                let mut target = block_on(state, date, to.0)?;
                target.items = inserted(target.items, to.1, vec![item]);
                crate::schedule::change_block_on_date(state, date, source)?;
                crate::schedule::change_block_on_date(state, date, target)?;
                vec![from.0, to.0]
            }
        }
        LogEdit::SetHardStart { block, index, on } => {
            check(state, date, block, index)?;
            let mut b = block_on(state, date, block)?;
            match b.items.get_mut(index) {
                Some(Item::Track { time_critical, .. } | Item::Category { time_critical, .. }) => {
                    *time_critical = on;
                }
                _ => return Err(ModelError::UnknownItem),
            }
            crate::schedule::change_block_on_date(state, date, b)?;
            vec![block]
        }
        LogEdit::AddBlock { start } => {
            crate::schedule::add_block_on_date(state, date, start)?;
            Vec::new()
        }
        LogEdit::MoveBlock { block, start } => {
            check(state, date, block, 0)?;
            let mut b = block_on(state, date, block)?;
            b.start = start;
            crate::schedule::change_block_on_date(state, date, b)?;
            vec![block]
        }
        LogEdit::RemoveBlock { block } => {
            check(state, date, block, 0)?;
            crate::schedule::remove_block_on_date(state, date, block)?;
            Vec::new()
        }
        LogEdit::Revert { block } => {
            check(state, date, block, 0)?;
            crate::schedule::revert_block_on_date(state, date, block)?;
            vec![block]
        }
    };
    // AR19b: an edit to the block on air takes effect at once, after the
    // row on air.
    if let Some(item) = crate::automation::on_air_item(state)
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
