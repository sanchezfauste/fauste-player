//! Form factors and the phone's view state (phone layout spec PL1).

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
fn live_is_the_first_screen() {
    assert_eq!(PhoneView::default().screen, PhoneScreen::Live);
}

#[test]
fn a_removed_playlists_player_falls_back_to_the_first() {
    let mut view = PhoneView {
        screen: PhoneScreen::Playlists,
        playlists: Some(PlayerId(3)),
    };
    view.settle(&ids(2));
    assert_eq!(view.playlists, None);
    assert_eq!(view.shown_player(&ids(2)), Some(PlayerId(0)));
    assert_eq!(view.screen, PhoneScreen::Playlists);
}

#[test]
fn a_playlists_player_that_exists_stays() {
    let mut view = PhoneView {
        playlists: Some(PlayerId(1)),
        ..PhoneView::default()
    };
    view.settle(&ids(2));
    assert_eq!(view.shown_player(&ids(2)), Some(PlayerId(1)));
    assert_eq!(view.shown_player(&[]), None);
}

#[test]
fn back_goes_to_live_once() {
    let mut view = PhoneView {
        screen: PhoneScreen::Carts,
        ..PhoneView::default()
    };
    assert!(view.back());
    assert_eq!(view.screen, PhoneScreen::Live);
    assert!(!view.back());
}

#[test]
fn a_long_sideways_drag_is_a_swipe() {
    use fp_app::ui::touch::swipe_direction;
    assert_eq!(swipe_direction(vec2(-120.0, 10.0)), Some(true));
    assert_eq!(swipe_direction(vec2(120.0, -10.0)), Some(false));
    assert_eq!(swipe_direction(vec2(-50.0, 0.0)), None, "too short");
    assert_eq!(swipe_direction(vec2(-120.0, 90.0)), None, "too steep");
}
