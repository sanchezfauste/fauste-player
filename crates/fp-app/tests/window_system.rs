//! Which window system the window opens on, on Linux (file drops reach the
//! application only through X11 with the windowing library in use).

use fp_app::window_system::{Choice, choose};
use fp_model::{Config, WindowSystem};

#[test]
fn auto_is_the_default() {
    assert_eq!(Config::default().ui.window_system, WindowSystem::Auto);
}

#[test]
fn auto_takes_x11_when_it_is_there_so_files_can_be_dropped() {
    assert_eq!(choose(WindowSystem::Auto, true, true), Some(Choice::X11));
    assert_eq!(choose(WindowSystem::Auto, true, false), Some(Choice::X11));
    // A Wayland session without XWayland: the library's own choice.
    assert_eq!(choose(WindowSystem::Auto, false, true), None);
}

#[test]
fn a_choice_that_is_not_available_falls_back_to_the_library() {
    assert_eq!(
        choose(WindowSystem::Wayland, true, true),
        Some(Choice::Wayland)
    );
    assert_eq!(choose(WindowSystem::Wayland, true, false), None);
    assert_eq!(choose(WindowSystem::X11, true, true), Some(Choice::X11));
    assert_eq!(choose(WindowSystem::X11, false, true), None);
}
