//! Icons with no Phosphor equivalent, drawn as shapes (spec §8.3).

use egui::epaint::PathShape;
use egui::{Color32, Pos2, Rect, Shape, Stroke, pos2};

/// Fade stop: a filled shape, flat on the left and ramping down to the right.
pub fn fade_stop(rect: Rect, color: Color32) -> Vec<Shape> {
    let r = rect.shrink(rect.width() * 0.15);
    let at = |x: f32, y: f32| pos2(r.left() + x * r.width(), r.top() + y * r.height());
    let points = vec![at(0.0, 1.0), at(0.0, 0.0), at(0.4, 0.0), at(1.0, 1.0)];
    vec![Shape::convex_polygon(points, color, Stroke::NONE)]
}

/// Stop after current: a reversed return arrow leading into a stop square.
pub fn stop_after(rect: Rect, color: Color32) -> Vec<Shape> {
    let r = rect.shrink(rect.width() * 0.1);
    let at = |x: f32, y: f32| pos2(r.left() + x * r.width(), r.top() + y * r.height());
    let stroke = Stroke::new((r.width() * 0.1).max(1.0), color);
    let square = Rect::from_min_max(at(0.5, 0.4), at(1.0, 0.9));
    let arrow_y = 0.65;
    let path: Vec<Pos2> = vec![at(0.05, 0.1), at(0.05, arrow_y), at(0.4, arrow_y)];
    let head = vec![
        at(0.42, arrow_y),
        at(0.3, arrow_y - 0.12),
        at(0.3, arrow_y + 0.12),
    ];
    vec![
        Shape::Path(PathShape::line(path, stroke)),
        Shape::convex_polygon(head, color, Stroke::NONE),
        Shape::rect_filled(square, 0.0, color),
    ]
}
