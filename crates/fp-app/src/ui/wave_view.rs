//! Which stretch of a track the waveform shows (feedback spec §3.3): one
//! pure mapping between seconds and pixels that drawing, markers, marker
//! handles, the hover tooltip and seeking all share.

use std::collections::HashMap;

use egui::{MouseWheelUnit, Rect, Vec2, pos2};
use fp_model::{EntryId, PlayerId};

/// A player's zoomed waveform: the view, the entry it belongs to (another
/// entry returns to the full view) and when the operator last moved it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveZoom {
    pub view: WaveView,
    pub entry: EntryId,
    /// `Scene::time` of the last zoom or pan.
    pub moved_at: f64,
}

/// Which waveform a view state belongs to (operator feedback 4, Q7.2): a
/// player's, or its CUE window's. Their zoom, menu and marker drag are
/// independent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WaveKey {
    Player(PlayerId),
    Cue(PlayerId),
}

impl WaveKey {
    /// The waveform's egui id: its memoised columns and its pan drag are
    /// keyed on it.
    pub fn id(self) -> egui::Id {
        match self {
            Self::Player(p) => egui::Id::new(("waveform", p)),
            Self::Cue(p) => egui::Id::new(("cue-waveform", p)),
        }
    }
}

/// The zoomed waveforms; one that is not here shows the whole track.
#[derive(Debug, Clone, Default)]
pub struct WaveZooms(HashMap<WaveKey, WaveZoom>);

impl WaveZooms {
    /// The zoom of `key`, when it was made on `entry`: another entry, or
    /// none, shows the whole track.
    pub fn get(&self, key: WaveKey, entry: Option<EntryId>) -> Option<WaveZoom> {
        self.0.get(&key).copied().filter(|z| Some(z.entry) == entry)
    }

    /// Keeps `zoom` for `key`; `None` returns it to the whole track.
    pub fn set(&mut self, key: WaveKey, zoom: Option<WaveZoom>) {
        match zoom {
            Some(z) => {
                self.0.insert(key, z);
            }
            None => {
                self.0.remove(&key);
            }
        }
    }
}

/// The visible stretch of a track, in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveView {
    pub start_secs: f64,
    pub span_secs: f64,
}

/// A usable length: finite and positive, else 0.
fn length(total: f64) -> f64 {
    if total.is_finite() && total > 0.0 {
        total
    } else {
        0.0
    }
}

/// The shortest span the waveform zooms to: one analysis bucket per pixel.
pub fn min_span(bucket_secs: f64, width: f32) -> f64 {
    let bucket = if bucket_secs.is_finite() && bucket_secs > 0.0 {
        bucket_secs
    } else {
        0.0
    };
    bucket * f64::from(width.max(1.0))
}

/// One wheel notch zooms the waveform in to this share of its span.
pub const ZOOM_STEP: f64 = 0.8;
/// One sideways notch pans the waveform by this share of its width.
pub const PAN_STEP: f32 = 0.1;
/// Smooth-scrolling wheels and trackpads report points: this many make a
/// notch.
pub const POINTS_PER_NOTCH: f32 = 50.0;
/// A wheel that reports pages: one page is this many notches.
pub const NOTCHES_PER_PAGE: f32 = 3.0;

/// A wheel event in notches: a line is one, points and pages are
/// converted, so a trackpad zooms in proportion instead of one step per
/// event.
pub fn wheel_notches(unit: MouseWheelUnit, delta: Vec2) -> Vec2 {
    match unit {
        MouseWheelUnit::Line => delta,
        MouseWheelUnit::Point => delta / POINTS_PER_NOTCH,
        MouseWheelUnit::Page => delta * NOTCHES_PER_PAGE,
    }
}

impl WaveView {
    /// The whole track.
    pub fn full(total: f64) -> Self {
        Self {
            start_secs: 0.0,
            span_secs: length(total),
        }
    }

    /// Whether the view shows the whole track.
    pub fn is_full(&self, total: f64) -> bool {
        let total = length(total);
        self.start_secs <= 0.0 && self.span_secs >= total - 1e-9
    }

    fn end_secs(&self) -> f64 {
        self.start_secs + self.span_secs
    }

    /// The x of `secs` in `rect` (outside it when `secs` is outside the view).
    pub fn x_of(&self, secs: f64, rect: Rect) -> f32 {
        if self.span_secs <= 0.0 {
            return rect.left();
        }
        let f = (secs - self.start_secs) / self.span_secs;
        rect.left() + (f * f64::from(rect.width())) as f32
    }

    /// The time at `x`, clamped to the view.
    pub fn secs_at(&self, x: f32, rect: Rect) -> f64 {
        let f = f64::from(((x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0));
        self.start_secs + f * self.span_secs
    }

    /// Keeps the view inside `[0, total]`; a span of the whole track or
    /// more is the full view.
    fn clamped(start: f64, span: f64, total: f64) -> Self {
        let total = length(total);
        if span >= total || !span.is_finite() {
            return Self::full(total);
        }
        Self {
            start_secs: start.clamp(0.0, total - span),
            span_secs: span,
        }
    }

    /// Zooms by `factor` (below 1 zooms in) keeping the time under `x`
    /// where it is, no closer than `min_span`.
    pub fn zoom_at(&self, x: f32, rect: Rect, factor: f64, total: f64, min_span: f64) -> Self {
        let total = length(total);
        if total <= 0.0 || !factor.is_finite() || factor <= 0.0 {
            return Self::full(total);
        }
        let anchor = self.secs_at(x, rect);
        let span = (self.span_secs * factor).clamp(min_span.min(total), total);
        let frac = f64::from(((x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0));
        Self::clamped(anchor - frac * span, span, total)
    }

    /// Moves the view by `dx` pixels: positive shows earlier audio, as when
    /// the waveform is dragged to the right.
    pub fn pan(&self, dx: f32, rect: Rect, total: f64) -> Self {
        let secs_per_px = self.span_secs / f64::from(rect.width().max(1.0));
        Self::clamped(
            self.start_secs - f64::from(dx) * secs_per_px,
            self.span_secs,
            total,
        )
    }

    /// One wheel event of `notches`: Shift or a mostly sideways wheel pans,
    /// otherwise it zooms around `x`, no closer than `min_span`.
    pub fn wheel(
        &self,
        notches: Vec2,
        shift: bool,
        x: f32,
        rect: Rect,
        total: f64,
        min_span: f64,
    ) -> Self {
        let sideways = notches.x.abs() > notches.y.abs();
        if shift || sideways {
            let step = if sideways { notches.x } else { notches.y };
            self.pan(step * rect.width() * PAN_STEP, rect, total)
        } else {
            self.zoom_at(
                x,
                rect,
                ZOOM_STEP.powf(f64::from(notches.y)),
                total,
                min_span,
            )
        }
    }

    /// Unchanged while `position` is visible; otherwise moved so it sits a
    /// tenth of the way in.
    pub fn follow(&self, position: f64, total: f64) -> Self {
        if !position.is_finite() || (self.start_secs..=self.end_secs()).contains(&position) {
            return *self;
        }
        Self::clamped(position - self.span_secs * 0.1, self.span_secs, total)
    }

    /// The visible parts of the trimmed head (before `cue_in`) and tail
    /// (after `cue_out`), as rectangles of `rect`.
    pub fn trimmed(
        &self,
        rect: Rect,
        cue_in: Option<f64>,
        cue_out: Option<f64>,
        total: f64,
    ) -> [Option<Rect>; 2] {
        let total = length(total);
        let head = cue_in
            .filter(|c| *c > self.start_secs && *c > 0.0)
            .map(|c| {
                let x = self.x_of(c.min(self.end_secs()), rect);
                Rect::from_min_max(rect.left_top(), pos2(x, rect.bottom()))
            });
        let tail = cue_out
            .filter(|c| *c < self.end_secs() && *c < total)
            .map(|c| {
                let x = self.x_of(c.max(self.start_secs), rect);
                Rect::from_min_max(pos2(x, rect.top()), rect.right_bottom())
            });
        [head, tail]
    }
}
