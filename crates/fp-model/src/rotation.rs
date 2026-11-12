//! Rotations (automation spec AU1–AU4): named, ordered lists of steps the
//! music filler plays in turn. A step names where its tracks come from and
//! how many it plays each time it comes up, drawn in `min..=max`.

use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::error::ModelError;
use crate::ids::{CategoryId, RotationId};
use crate::state::AppState;
use crate::track::TrackKind;

/// Where a step's tracks come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepSource {
    /// A category with all its subcategories.
    Category(CategoryId),
    /// Every track of a kind (the default rotation's "all music").
    Kind(TrackKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RotationStep {
    pub source: StepSource,
    pub min: u8,
    pub max: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rotation {
    pub id: RotationId,
    pub name: String,
    pub steps: Vec<RotationStep>,
}

/// Every rotation, in display order, and which one is the default (AU4).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Rotations {
    list: Vec<Rotation>,
    default: Option<RotationId>,
}

/// The fewest tracks a step may ask for: 0 lets the step sit out a turn
/// ("between 0 and 1"). It checks data, it is not an operator setting.
pub const STEP_MIN_RANGE: RangeInclusive<u8> = 0..=99;

/// The most tracks a step may ask for: at least 1, so that every step can
/// play.
pub const STEP_MAX_RANGE: RangeInclusive<u8> = 1..=99;

impl Rotations {
    pub fn get(&self, id: RotationId) -> Option<&Rotation> {
        self.list.iter().find(|r| r.id == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rotation> {
        self.list.iter()
    }

    pub fn default_id(&self) -> Option<RotationId> {
        self.default
    }

    pub fn default_rotation(&self) -> Option<&Rotation> {
        self.default.and_then(|id| self.get(id))
    }

    /// Rotations as loaded: a default that names no rotation becomes the
    /// first one (none when there is none).
    pub fn from_parts(list: Vec<Rotation>, default: Option<RotationId>) -> Self {
        let mut out = Self { list, default };
        out.settle_default();
        out
    }

    pub fn into_parts(self) -> (Vec<Rotation>, Option<RotationId>) {
        (self.list, self.default)
    }

    /// The largest raw id in use (0 when empty).
    pub fn max_raw_id(&self) -> u64 {
        self.list.iter().map(|r| r.id.0).max().unwrap_or(0)
    }

    fn settle_default(&mut self) {
        if self.default_rotation().is_none() {
            self.default = self.list.first().map(|r| r.id);
        }
    }
}

/// AU1: a step's counts lie in `STEP_MIN_RANGE` and `STEP_MAX_RANGE`, and
/// `min <= max`.
pub fn check_step(step: &RotationStep) -> Result<(), ModelError> {
    let ok = STEP_MIN_RANGE.contains(&step.min)
        && STEP_MAX_RANGE.contains(&step.max)
        && step.min <= step.max;
    if ok {
        Ok(())
    } else {
        Err(ModelError::InvalidRotationStep)
    }
}

fn checked_name(name: &str) -> Result<String, ModelError> {
    let name = name.trim();
    if name.is_empty() {
        Err(ModelError::EmptyRotationName)
    } else {
        Ok(name.to_owned())
    }
}

fn check_steps(steps: &[RotationStep]) -> Result<(), ModelError> {
    if steps.is_empty() {
        return Err(ModelError::EmptyRotation);
    }
    steps.iter().try_for_each(check_step)
}

/// AU4: with no rotation at all, adds `name` with one step over all music
/// as the default.
pub fn ensure_default(state: &mut AppState, name: &str) {
    if state.rotations.list.is_empty() {
        let id = state.ids.rotation();
        state.rotations.list.push(Rotation {
            id,
            name: name.to_owned(),
            steps: vec![RotationStep {
                source: StepSource::Kind(TrackKind::Music),
                min: 1,
                max: 1,
            }],
        });
        state.rotations.default = Some(id);
    }
}

pub(crate) fn create(
    state: &mut AppState,
    name: &str,
    steps: Vec<RotationStep>,
) -> Result<(), ModelError> {
    let name = checked_name(name)?;
    check_steps(&steps)?;
    let id = state.ids.rotation();
    state.rotations.list.push(Rotation { id, name, steps });
    state.rotations.settle_default();
    Ok(())
}

pub(crate) fn update(state: &mut AppState, rotation: Rotation) -> Result<(), ModelError> {
    let name = checked_name(&rotation.name)?;
    check_steps(&rotation.steps)?;
    let slot = state
        .rotations
        .list
        .iter_mut()
        .find(|r| r.id == rotation.id)
        .ok_or(ModelError::UnknownRotation(rotation.id))?;
    slot.name = name;
    slot.steps = rotation.steps;
    Ok(())
}

pub(crate) fn delete(state: &mut AppState, id: RotationId) -> Result<(), ModelError> {
    if state.rotations.get(id).is_none() {
        return Err(ModelError::UnknownRotation(id));
    }
    if state.rotations.list.len() == 1 {
        return Err(ModelError::LastRotation);
    }
    state.rotations.list.retain(|r| r.id != id);
    state.rotations.settle_default();
    Ok(())
}

pub(crate) fn set_default(state: &mut AppState, id: RotationId) -> Result<(), ModelError> {
    if state.rotations.get(id).is_none() {
        return Err(ModelError::UnknownRotation(id));
    }
    state.rotations.default = Some(id);
    Ok(())
}
