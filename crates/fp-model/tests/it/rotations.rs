#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU1, AU3, AU4: rotations and their steps.

use fp_model::rotation::{Rotation, RotationStep, Rotations, StepSource, ensure_default};
use fp_model::{AppState, CategoryId, Command, Config, ModelError, RotationId, TrackKind, apply};

fn state() -> AppState {
    AppState::new(Config::default(), "P")
}

fn step(source: StepSource, min: u8, max: u8) -> RotationStep {
    RotationStep { source, min, max }
}

fn create(
    s: &mut AppState,
    name: &str,
    steps: Vec<RotationStep>,
) -> Result<RotationId, ModelError> {
    apply(
        s,
        Command::CreateRotation {
            name: name.into(),
            steps,
        },
    )?;
    Ok(s.rotations.iter().map(|r| r.id).max().unwrap())
}

fn music(min: u8, max: u8) -> RotationStep {
    step(StepSource::Kind(TrackKind::Music), min, max)
}

#[test]
fn create_validates_the_name_and_steps() {
    let mut s = state();
    assert_eq!(
        create(&mut s, "  ", vec![music(1, 1)]),
        Err(ModelError::EmptyRotationName)
    );
    assert_eq!(
        create(&mut s, "Day", vec![music(3, 2)]),
        Err(ModelError::InvalidRotationStep)
    );
    assert_eq!(
        create(&mut s, "Day", vec![music(0, 1)]),
        Err(ModelError::InvalidRotationStep)
    );
    assert_eq!(
        create(&mut s, "Day", vec![music(1, 100)]),
        Err(ModelError::InvalidRotationStep)
    );
    assert_eq!(s.rotations.iter().count(), 0);
}

#[test]
fn a_rotation_needs_at_least_one_step() {
    // Final review: a default rotation without steps would leave the
    // filler nothing to fall back on (AU34).
    let mut s = state();
    assert_eq!(
        create(&mut s, "Day", vec![]),
        Err(ModelError::EmptyRotation)
    );
    let day = create(&mut s, "Day", vec![music(1, 1)]).unwrap();
    assert_eq!(
        apply(
            &mut s,
            Command::UpdateRotation(Rotation {
                id: day,
                name: "Day".into(),
                steps: vec![],
            }),
        ),
        Err(ModelError::EmptyRotation)
    );
    assert_eq!(s.rotations.get(day).unwrap().steps.len(), 1);
}

#[test]
fn create_update_delete_round_trip() {
    let mut s = state();
    let day = create(&mut s, "Day", vec![music(2, 3)]).unwrap();
    let night = create(&mut s, " Night ", vec![music(1, 1)]).unwrap();
    assert_eq!(s.rotations.get(night).unwrap().name, "Night");
    let gold = CategoryId(999);
    apply(
        &mut s,
        Command::UpdateRotation(Rotation {
            id: day,
            name: "Daytime".into(),
            steps: vec![music(2, 3), step(StepSource::Category(gold), 1, 1)],
        }),
    )
    .unwrap();
    let updated = s.rotations.get(day).unwrap();
    assert_eq!(updated.name, "Daytime");
    assert_eq!(updated.steps.len(), 2);
    assert_eq!(
        apply(
            &mut s,
            Command::UpdateRotation(Rotation {
                id: RotationId(12_345),
                name: "X".into(),
                steps: vec![music(1, 1)],
            }),
        ),
        Err(ModelError::UnknownRotation(RotationId(12_345)))
    );
    apply(&mut s, Command::DeleteRotation(night)).unwrap();
    assert!(s.rotations.get(night).is_none());
}

#[test]
fn the_last_rotation_cannot_be_deleted() {
    let mut s = state();
    let only = create(&mut s, "Only", vec![music(1, 1)]).unwrap();
    assert_eq!(
        apply(&mut s, Command::DeleteRotation(only)),
        Err(ModelError::LastRotation)
    );
}

#[test]
fn deleting_the_default_promotes_the_first() {
    let mut s = state();
    let a = create(&mut s, "A", vec![music(1, 1)]).unwrap();
    let b = create(&mut s, "B", vec![music(1, 1)]).unwrap();
    let c = create(&mut s, "C", vec![music(1, 1)]).unwrap();
    apply(&mut s, Command::SetDefaultRotation(c)).unwrap();
    assert_eq!(s.rotations.default_id(), Some(c));
    apply(&mut s, Command::DeleteRotation(c)).unwrap();
    assert_eq!(s.rotations.default_id(), Some(a));
    assert_eq!(
        apply(&mut s, Command::SetDefaultRotation(c)),
        Err(ModelError::UnknownRotation(c))
    );
    let _ = b;
}

#[test]
fn the_first_rotation_created_is_the_default() {
    let mut s = state();
    let a = create(&mut s, "A", vec![music(1, 1)]).unwrap();
    assert_eq!(s.rotations.default_id(), Some(a));
}

#[test]
fn ensure_default_creates_music_once() {
    let mut s = state();
    ensure_default(&mut s, "Music");
    ensure_default(&mut s, "Music");
    let all: Vec<&Rotation> = s.rotations.iter().collect();
    assert_eq!(all.len(), 1);
    let r = all[0];
    assert_eq!(r.name, "Music");
    assert_eq!(r.steps, vec![music(1, 1)]);
    assert_eq!(s.rotations.default_id(), Some(r.id));
}

#[test]
fn from_parts_repairs_a_dangling_default() {
    let r = Rotation {
        id: RotationId(5),
        name: "R".into(),
        steps: vec![music(1, 1)],
    };
    let rotations = Rotations::from_parts(vec![r], Some(RotationId(77)));
    assert_eq!(rotations.default_id(), Some(RotationId(5)));
    assert_eq!(
        Rotations::from_parts(Vec::new(), Some(RotationId(1))).default_id(),
        None
    );
}
