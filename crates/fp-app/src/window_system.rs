//! The window system the window opens on, on Linux. The windowing library
//! delivers files dropped from a file manager only through X11, so `Auto`
//! prefers X11 (natively or through XWayland) when it is there.

use fp_model::WindowSystem;

/// The window system to ask the windowing library for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    X11,
    Wayland,
}

/// What `setting` asks for, given which window systems are there; `None`
/// leaves the choice to the windowing library.
pub fn choose(setting: WindowSystem, x11: bool, wayland: bool) -> Option<Choice> {
    match setting {
        WindowSystem::Auto | WindowSystem::X11 => x11.then_some(Choice::X11),
        WindowSystem::Wayland => wayland.then_some(Choice::Wayland),
    }
}

/// [`choose`] with what the environment offers (`DISPLAY` for X11,
/// `WAYLAND_DISPLAY` for Wayland).
pub fn from_env(setting: WindowSystem) -> Option<Choice> {
    let set = |name: &str| std::env::var_os(name).is_some_and(|v| !v.is_empty());
    choose(setting, set("DISPLAY"), set("WAYLAND_DISPLAY"))
}
