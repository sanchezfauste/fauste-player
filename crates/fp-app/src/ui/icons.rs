//! Icons with no Phosphor equivalent, drawn as shapes (spec §8.3).

use egui::{Color32, Rect, Shape, Stroke, pos2};

/// Fade stop: a filled shape, flat on the left and ramping down to the right.
pub fn fade_stop(rect: Rect, color: Color32) -> Vec<Shape> {
    let r = rect.shrink(rect.width() * 0.15);
    let at = |x: f32, y: f32| pos2(r.left() + x * r.width(), r.top() + y * r.height());
    let points = vec![at(0.0, 1.0), at(0.0, 0.0), at(0.4, 0.0), at(1.0, 1.0)];
    vec![Shape::convex_polygon(points, color, Stroke::NONE)]
}

/// Stop after current: a play triangle followed by a stop square ("play,
/// then stop"), built from the two standard transport glyphs.
pub fn stop_after(rect: Rect, color: Color32) -> Vec<Shape> {
    let r = rect.shrink(rect.height() * 0.1);
    // The square's side: 80 % of the height, unless the width is the limit
    // (triangle 0.8 side + gap 0.25 side + square 1 side).
    let side = (r.height() * 0.8).min(r.width() / 2.05);
    let triangle_width = side * 0.8;
    let gap = side * 0.25;
    let left = r.center().x - (triangle_width + gap + side) / 2.0;
    let top = r.center().y - side / 2.0;
    let triangle = vec![
        pos2(left, top),
        pos2(left + triangle_width, top + side / 2.0),
        pos2(left, top + side),
    ];
    let square = Rect::from_min_size(
        pos2(left + triangle_width + gap, top),
        egui::vec2(side, side),
    );
    vec![
        Shape::convex_polygon(triangle, color, Stroke::NONE),
        Shape::rect_filled(square, 0.0, color),
    ]
}

/// A bar followed by `count` triangles pointing left, each `tri` × side
/// wide, centred in `rect`: the "back" transport glyphs.
fn bar_then_left_triangles(rect: Rect, color: Color32, count: usize, tri: f32) -> Vec<Shape> {
    const BAR: f32 = 0.2;
    let r = rect.shrink(rect.height() * 0.1);
    let units = BAR + tri * count as f32;
    let side = (r.height() * 0.8).min(r.width() / units);
    let left = r.center().x - side * units / 2.0;
    let top = r.center().y - side / 2.0;
    let mut shapes = vec![Shape::rect_filled(
        Rect::from_min_size(pos2(left, top), egui::vec2(side * BAR, side)),
        0.0,
        color,
    )];
    for i in 0..count {
        let x = left + side * (BAR + tri * i as f32);
        shapes.push(Shape::convex_polygon(
            vec![
                pos2(x + side * tri, top),
                pos2(x, top + side / 2.0),
                pos2(x + side * tri, top + side),
            ],
            color,
            Stroke::NONE,
        ));
    }
    shapes
}

/// Restart: a bar and one triangle pointing left (back to the start).
pub fn restart(rect: Rect, color: Color32) -> Vec<Shape> {
    bar_then_left_triangles(rect, color, 1, 0.8)
}

/// Previous: a bar and two triangles pointing left, as on a CD player.
pub fn previous(rect: Rect, color: Color32) -> Vec<Shape> {
    bar_then_left_triangles(rect, color, 2, 0.6)
}
