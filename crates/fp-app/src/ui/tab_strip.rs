//! The layout of the playlist tabs (feedback 2 spec O35): they share the
//! strip while each keeps its minimum width, then they scroll.

/// The narrowest a tab gets before the strip scrolls.
pub const MIN_TAB_WIDTH: f32 = 72.0;
/// Width of each scroll arrow.
pub const ARROW_WIDTH: f32 = 22.0;

/// How `count` tabs are laid out in a strip `available` pixels wide.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub tab_width: f32,
    /// The tabs do not fit: arrows show and the tabs scroll.
    pub overflow: bool,
    /// Width of the clipped view the tabs sit in (the strip, less the arrows).
    pub view_width: f32,
    /// Total width of the tabs.
    pub content_width: f32,
}

pub fn layout(available: f32, count: usize) -> Layout {
    let available = if available.is_finite() {
        available.max(0.0)
    } else {
        0.0
    };
    if count > 0 {
        let share = available / count as f32;
        if share >= MIN_TAB_WIDTH {
            return Layout {
                tab_width: share,
                overflow: false,
                view_width: available,
                content_width: available,
            };
        }
    } else {
        return Layout {
            tab_width: available,
            overflow: false,
            view_width: available,
            content_width: 0.0,
        };
    }
    Layout {
        tab_width: MIN_TAB_WIDTH,
        overflow: true,
        view_width: (available - 2.0 * ARROW_WIDTH).max(0.0),
        content_width: count as f32 * MIN_TAB_WIDTH,
    }
}

/// `offset` limited to what the content can scroll.
pub fn clamp_offset(offset: f32, content: f32, view: f32) -> f32 {
    let offset = if offset.is_finite() { offset } else { 0.0 };
    offset.clamp(0.0, (content - view).max(0.0))
}

/// The least change of `offset` that shows tab `index` whole.
pub fn reveal(offset: f32, index: usize, layout: &Layout) -> f32 {
    let left = index as f32 * layout.tab_width;
    let right = left + layout.tab_width;
    let wanted = if left < offset {
        left
    } else if right > offset + layout.view_width {
        right - layout.view_width
    } else {
        offset
    };
    clamp_offset(wanted, layout.content_width, layout.view_width)
}

/// One arrow press: a tab to the left (`-1`) or right (`1`).
pub fn step(offset: f32, direction: i32, layout: &Layout) -> f32 {
    clamp_offset(
        offset + direction as f32 * layout.tab_width,
        layout.content_width,
        layout.view_width,
    )
}
