use crate::state::AppState;
use egui::emath::RectTransform;
use egui::{Color32, Pos2, Rect, Sense, Shape, Stroke, Vec2};
use thetta_geometry::polygon_ops;
use thetta_geometry::types::{PieceId, Placement, Point2D, Polygon};

#[derive(Debug, Clone)]
pub struct ViewportState {
    pub zoom: f32,
    pub pan: Vec2,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: Vec2::ZERO,
        }
    }
}

pub fn show_viewport(ui: &mut egui::Ui, state: &mut ViewportState, app: &AppState) -> Option<PieceId> {
    let available = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(available, Sense::click_and_drag());
    let painter = ui.painter_at(rect);

    let mut world = Rect::from_min_max(
        Pos2::new(0.0, 0.0),
        Pos2::new(app.sheet_config.width as f32, app.sheet_config.height as f32),
    );

    if world.width() < 1.0 || world.height() < 1.0 {
        world = Rect::from_min_max(Pos2::ZERO, Pos2::new(100.0, 100.0));
    }

    let center = world.center();
    let scaled_size = world.size() / state.zoom.max(0.05);
    world = Rect::from_center_size(center + state.pan, scaled_size);

    let to_screen = RectTransform::from_to(world, rect);

    if response.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll.abs() > 0.0 {
            let factor = (1.0 + scroll * 0.001).clamp(0.2, 4.0);
            state.zoom = (state.zoom * factor).clamp(0.1, 50.0);
        }
    }

    let dragging = ui.input(|i| i.pointer.middle_down() || (i.modifiers.ctrl && i.pointer.primary_down()));
    if dragging && response.dragged() {
        let delta = response.drag_delta();
        let x_scale = world.width() / rect.width().max(1.0);
        let y_scale = world.height() / rect.height().max(1.0);
        state.pan.x -= delta.x * x_scale;
        state.pan.y -= delta.y * y_scale;
    }

    painter.rect_stroke(
        to_screen.transform_rect(Rect::from_min_max(
            Pos2::new(0.0, 0.0),
            Pos2::new(app.sheet_config.width as f32, app.sheet_config.height as f32),
        )),
        0.0,
        Stroke::new(2.0, Color32::WHITE),
        egui::StrokeKind::Outside,
    );

    let used_width = app
        .current_result
        .as_ref()
        .and_then(|r| r.sheets.first())
        .map(|s| s.strip_length)
        .unwrap_or(app.stats.strip_length)
        .clamp(0.0, app.sheet_config.width);
    if used_width > 0.0 {
        let x = used_width as f32;
        let a = to_screen.transform_pos(Pos2::new(x, 0.0));
        let b = to_screen.transform_pos(Pos2::new(x, app.sheet_config.height as f32));
        painter.line_segment([a, b], Stroke::new(2.0, Color32::from_rgb(255, 180, 0)));
    }

    let mut click_hit = None;

    if let Some(result) = &app.current_result {
        if let Some(sheet) = result.sheets.first() {
            for placement in &sheet.placements {
                if let Some(piece) = app.pieces.iter().find(|p| p.id == placement.piece_id) {
                    let poly = placed_polygon(&piece.polygon, placement);
                    let points = poly
                        .outer
                        .iter()
                        .map(|p| to_screen.transform_pos(Pos2::new(p.x as f32, p.y as f32)))
                        .collect::<Vec<_>>();

                    let mut fill = app.piece_color(&piece.source_file);
                    if app.selected_piece == Some(piece.id) {
                        fill = fill.gamma_multiply(0.65);
                    }

                    painter.add(Shape::convex_polygon(
                        points.clone(),
                        fill,
                        Stroke::new(
                            if app.selected_piece == Some(piece.id) { 3.0 } else { 1.0 },
                            Color32::BLACK,
                        ),
                    ));

                    if response.clicked()
                        && let Some(pointer) = response.interact_pointer_pos()
                        && point_in_polygon_screen(pointer, &points)
                    {
                        click_hit = Some(piece.id);
                    }
                }
            }
        }
    }

    click_hit
}

fn placed_polygon(poly: &Polygon, placement: &Placement) -> Polygon {
    let rotated = polygon_ops::rotate(poly, placement.rotation, Point2D { x: 0.0, y: 0.0 });
    polygon_ops::translate(&rotated, placement.x, placement.y)
}

fn point_in_polygon_screen(p: Pos2, vertices: &[Pos2]) -> bool {
    let mut inside = false;
    for i in 0..vertices.len() {
        let j = (i + 1) % vertices.len();
        let a = vertices[i];
        let b = vertices[j];
        let hit = (a.y > p.y) != (b.y > p.y)
            && p.x < (b.x - a.x) * (p.y - a.y) / ((b.y - a.y).abs().max(1e-6)) + a.x;
        if hit {
            inside = !inside;
        }
    }
    inside
}
