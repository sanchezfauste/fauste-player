//! Form factors and the phone's view state (mobile spec §6).

use egui::vec2;
use fp_app::ui::touch::{FormFactor, PhoneScreen, PhoneView, form_factor};
use fp_model::PlayerId;

fn ids(n: u64) -> Vec<PlayerId> {
    (0..n).map(PlayerId).collect()
}

#[test]
fn a_narrow_screen_is_a_phone() {
    assert_eq!(
        form_factor(vec2(412.0, 892.0)),
        FormFactor::Phone { landscape: false }
    );
}

#[test]
fn a_wide_phone_is_in_landscape() {
    assert_eq!(
        form_factor(vec2(892.0, 412.0)),
        FormFactor::Phone { landscape: true }
    );
}

#[test]
fn a_large_screen_is_a_tablet() {
    assert_eq!(form_factor(vec2(800.0, 1280.0)), FormFactor::Tablet);
    assert_eq!(form_factor(vec2(1280.0, 600.0)), FormFactor::Tablet);
}

#[test]
fn a_removed_open_player_falls_back_to_all() {
    let mut view = PhoneView {
        open: Some(PlayerId(3)),
        ..PhoneView::default()
    };
    view.settle(&ids(2));
    assert_eq!(view.open, None);
    assert_eq!(view.screen, PhoneScreen::Players);
}

#[test]
fn an_open_player_that_exists_stays_open() {
    let mut view = PhoneView {
        open: Some(PlayerId(1)),
        ..PhoneView::default()
    };
    view.settle(&ids(2));
    assert_eq!(view.open, Some(PlayerId(1)));
}

#[test]
fn pairs_cover_odd_counts() {
    assert_eq!(PhoneView::pairs(0), 0);
    assert_eq!(PhoneView::pairs(1), 1);
    assert_eq!(PhoneView::pairs(3), 2);
    assert_eq!(PhoneView::pairs(4), 2);
    let view = PhoneView {
        pair: 1,
        ..PhoneView::default()
    };
    assert_eq!(view.pair_players(&ids(3)), vec![PlayerId(2)]);
}

#[test]
fn the_pair_never_points_past_the_last_player() {
    let mut view = PhoneView {
        pair: 5,
        ..PhoneView::default()
    };
    view.settle(&ids(3));
    assert_eq!(view.pair, 1);
    view.settle(&[]);
    assert_eq!(view.pair, 0);
    assert!(view.pair_players(&[]).is_empty());
}

#[test]
fn swiping_stops_at_both_ends() {
    let mut view = PhoneView::default();
    view.swipe(false, 4);
    assert_eq!(view.pair, 0);
    view.swipe(true, 4);
    assert_eq!(view.pair, 1);
    view.swipe(true, 4);
    assert_eq!(view.pair, 1);
    assert_eq!(view.pair_players(&ids(4)), vec![PlayerId(2), PlayerId(3)]);
}

#[test]
fn a_long_sideways_drag_is_a_swipe() {
    use fp_app::ui::touch::swipe_direction;
    assert_eq!(swipe_direction(vec2(-120.0, 10.0)), Some(true));
    assert_eq!(swipe_direction(vec2(120.0, -10.0)), Some(false));
    assert_eq!(swipe_direction(vec2(-50.0, 0.0)), None, "too short");
    assert_eq!(swipe_direction(vec2(-120.0, 90.0)), None, "too steep");
}
