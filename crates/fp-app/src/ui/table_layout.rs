//! The widths of the track table's columns (feedback 2 spec O16 and O24):
//! pure functions from the stored fractions to pixels, and from a drag of a
//! column edge to the widths of all the columns, recomputed every frame.

use egui::{Pos2, Rect};
use fp_model::{ColumnWidths, TableColumn};

/// Width of one digit of the `#` column, and what its icon and padding take.
const DIGIT: f32 = 8.0;
const NUMBER_PADDING: f32 = 26.0;

/// The narrowest a column may be, in points. The `#` column grows with the
/// number of digits of the playlist length.
pub fn column_min(column: TableColumn, number_digits: usize) -> f32 {
    match column {
        TableColumn::Number => number_digits as f32 * DIGIT + NUMBER_PADDING,
        TableColumn::Title | TableColumn::Artist | TableColumn::Album | TableColumn::FileName => {
            60.0
        }
        TableColumn::Genre => 50.0,
        // "2019-05-14" fits.
        TableColumn::Date => 84.0,
        // "00:00:00" fits.
        TableColumn::Duration | TableColumn::Intro => 52.0,
    }
}

/// How a column shares the room that the fixed ones leave (the default
/// layout): the text columns by these weights, the others at their minimum.
fn weight(column: TableColumn) -> f32 {
    match column {
        TableColumn::Title => 3.0,
        TableColumn::Artist | TableColumn::Album | TableColumn::FileName => 2.0,
        TableColumn::Genre => 1.5,
        TableColumn::Number | TableColumn::Date | TableColumn::Duration | TableColumn::Intro => 0.0,
    }
}

/// Widths close to `wanted` that respect `mins` and add up to `total`:
/// columns that would be too narrow take their minimum and the others share
/// the rest in proportion to what they wanted. When even the minimums do not
/// fit, they are all scaled down.
pub fn fit(wanted: &[f32], mins: &[f32], total: f32) -> Vec<f32> {
    let n = wanted.len().min(mins.len());
    let total = if total.is_finite() {
        total.max(0.0)
    } else {
        0.0
    };
    let wanted: Vec<f32> = wanted
        .iter()
        .take(n)
        .map(|w| if w.is_finite() { w.max(0.0) } else { 0.0 })
        .collect();
    let mins: Vec<f32> = mins.iter().take(n).map(|m| m.max(0.0)).collect();
    let min_sum: f32 = mins.iter().sum();
    if n == 0 {
        return Vec::new();
    }
    if min_sum >= total {
        let scale = if min_sum > 0.0 { total / min_sum } else { 0.0 };
        return mins.iter().map(|m| m * scale).collect();
    }
    let mut pinned = vec![false; n];
    loop {
        let pinned_sum: f32 = mins
            .iter()
            .zip(&pinned)
            .filter(|(_, p)| **p)
            .map(|(m, _)| m)
            .sum();
        let free_wanted: f32 = wanted
            .iter()
            .zip(&pinned)
            .filter(|(_, p)| !**p)
            .map(|(w, _)| w)
            .sum();
        let free_total = total - pinned_sum;
        let free_count = pinned.iter().filter(|p| !**p).count();
        let share = |w: f32| {
            if free_wanted > 0.0 {
                w * free_total / free_wanted
            } else {
                free_total / free_count as f32
            }
        };
        let mut changed = false;
        for ((pin, w), m) in pinned.iter_mut().zip(&wanted).zip(&mins) {
            if !*pin && share(*w) < *m {
                *pin = true;
                changed = true;
            }
        }
        if !changed {
            return wanted
                .iter()
                .zip(&mins)
                .zip(&pinned)
                .map(|((w, m), pin)| if *pin { *m } else { share(*w) })
                .collect();
        }
    }
}

/// The default layout of `columns` in a table `width` wide: the narrow
/// columns (`#`, Date, Duration, Intro) at their minimum and the text
/// columns sharing the rest, Title 3, Artist 2, Album 2, File name 2 and
/// Genre 1.5. With `#`, Title, Artist and Duration that is the layout the
/// table always had: the minimums for `#` and Duration and 60:40 of the rest.
fn default_px(columns: &[TableColumn], width: f32, digits: usize) -> Vec<f32> {
    let mins: Vec<f32> = columns.iter().map(|c| column_min(*c, digits)).collect();
    let fixed: f32 = columns
        .iter()
        .zip(&mins)
        .filter(|(c, _)| weight(**c) == 0.0)
        .map(|(_, m)| m)
        .sum();
    let weights: f32 = columns.iter().map(|c| weight(*c)).sum();
    let rest = (width - fixed).max(0.0);
    let wanted: Vec<f32> = columns
        .iter()
        .zip(&mins)
        .map(|(c, m)| {
            if weights > 0.0 && weight(*c) > 0.0 {
                rest * weight(*c) / weights
            } else {
                *m
            }
        })
        .collect();
    fit(&wanted, &mins, width)
}

/// Pixel widths of `columns` for a table `width` wide. The stored fractions
/// of the columns that have one are used, scaled to the room left by the
/// columns without one (a column shown after the widths were stored), which
/// take their default width; with no stored fractions every column does.
/// The minimums win, and the widths always add up to `width`.
pub fn column_px(
    columns: &[TableColumn],
    stored: &ColumnWidths,
    width: f32,
    number_digits: usize,
) -> Vec<f32> {
    let width = if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    };
    let default = default_px(columns, width, number_digits);
    let stored = stored.clone().normalized();
    let have: Vec<Option<f32>> = columns.iter().map(|c| stored.fraction(*c)).collect();
    let stored_sum: f32 = have.iter().flatten().sum();
    if stored_sum <= 0.0 {
        return default;
    }
    let reserved: f32 = have
        .iter()
        .zip(&default)
        .filter(|(f, _)| f.is_none())
        .map(|(_, d)| d)
        .sum();
    let room = (width - reserved).max(0.0);
    let wanted: Vec<f32> = have
        .iter()
        .zip(&default)
        .map(|(f, d)| f.map_or(*d, |f| f / stored_sum * room))
        .collect();
    let mins: Vec<f32> = columns
        .iter()
        .map(|c| column_min(*c, number_digits))
        .collect();
    fit(&wanted, &mins, width)
}

/// The widths while the right edge of column `edge` is dragged so that the
/// column is `new_width` wide (O16). The columns on its left keep their
/// width; the ones on its right share what is left in proportion to their
/// width when the drag began, so every one of them moves on every frame. No
/// column goes under its minimum, and the total never changes. The last
/// column has no edge to its right: nothing moves.
pub fn resize_px(start: &[f32], mins: &[f32], edge: usize, new_width: f32) -> Vec<f32> {
    let n = start.len().min(mins.len());
    if edge + 1 >= n || !new_width.is_finite() {
        return start.to_vec();
    }
    let total: f32 = start.iter().sum();
    let left: f32 = start.iter().take(edge).sum();
    let right_min: f32 = mins.iter().take(n).skip(edge + 1).sum();
    let lowest = mins.get(edge).copied().unwrap_or(0.0);
    let highest = (total - left - right_min).max(lowest);
    let width = new_width.clamp(lowest, highest);
    let mut out: Vec<f32> = start.iter().take(edge).copied().collect();
    out.push(width);
    let rest = total - left - width;
    let wanted: Vec<f32> = start.iter().take(n).skip(edge + 1).copied().collect();
    let right_mins: Vec<f32> = mins.iter().take(n).skip(edge + 1).copied().collect();
    out.extend(fit(&wanted, &right_mins, rest));
    out
}

/// The widths to store for `px`: each column's share of the total.
pub fn fractions_of(columns: &[TableColumn], px: &[f32]) -> ColumnWidths {
    ColumnWidths::keyed(columns.iter().copied().zip(px.iter().copied()))
}

/// The row boundary a dragged entry would be dropped at (operator feedback 4,
/// Q4): the one nearest to the pointer, from `0` (before the first row) to
/// `len` (after the last). It comes from the pointer and the table's
/// geometry alone, never from the widget under the pointer. `body` is the
/// visible part of the table body (without the header and the scroll bar)
/// and `scroll_y` how far it is scrolled. `None` when the pointer is not
/// inside `body`.
pub fn drop_index(
    body: Rect,
    scroll_y: f32,
    row_height: f32,
    len: usize,
    pointer: Pos2,
) -> Option<usize> {
    if !body.contains(pointer) || !(row_height.is_finite() && row_height > 0.0) {
        return None;
    }
    let y = pointer.y - body.top() + if scroll_y.is_finite() { scroll_y } else { 0.0 };
    let boundary = (y / row_height).round().max(0.0);
    Some((boundary as usize).min(len))
}

/// The screen y of boundary `index` (the top edge of row `index`), when it is
/// inside the visible `body`; the bar is drawn there even if the rows next
/// to it were not built this frame.
pub fn boundary_y(body: Rect, scroll_y: f32, row_height: f32, index: usize) -> Option<f32> {
    let y = body.top() + index as f32 * row_height - scroll_y;
    (y.is_finite() && y >= body.top() - 0.5 && y <= body.bottom() + 0.5).then_some(y)
}

/// `x` is within `grab` points of an edge between two columns of a table
/// whose left side is at `left` (the column resize grab zones: a drop there
/// has no target, Q4.2). The right edge of the last column is not one.
pub fn on_column_edge(px: &[f32], left: f32, grab: f32, x: f32) -> bool {
    let mut edge = left;
    px.iter().take(px.len().saturating_sub(1)).any(|w| {
        edge += w;
        (x - edge).abs() <= grab
    })
}

/// How far from the top or bottom edge of a table body a dragged entry
/// makes the table scroll by itself (operator feedback 4), in points. A UI
/// metric: a body shorter than three of these keeps a third of its height
/// for each zone, so that its middle never scrolls.
pub const DRAG_SCROLL_EDGE: f32 = 36.0;

/// How fast a table scrolls by itself with the pointer on its edge, in
/// points per second (about 32 rows of 28 points).
pub const DRAG_SCROLL_MAX_SPEED: f32 = 900.0;

/// The speed a table body scrolls at while an entry is dragged with the
/// pointer at `pointer`, in points per second: negative up, positive down,
/// zero outside the edge zones or outside `body`. It ramps up linearly from
/// the inner side of a zone to [`DRAG_SCROLL_MAX_SPEED`] at the edge.
pub fn drag_scroll_speed(body: Rect, pointer: Pos2) -> f32 {
    if !body.contains(pointer) || !body.is_finite() {
        return 0.0;
    }
    let zone = DRAG_SCROLL_EDGE.min(body.height() / 3.0);
    if zone.is_nan() || zone <= 0.0 {
        return 0.0;
    }
    let ramp = |distance: f32| ((zone - distance) / zone).clamp(0.0, 1.0) * DRAG_SCROLL_MAX_SPEED;
    let (up, down) = (pointer.y - body.top(), body.bottom() - pointer.y);
    if up < zone {
        -ramp(up)
    } else if down < zone {
        ramp(down)
    } else {
        0.0
    }
}
