use crate::state::AppState;
use egui::emath::RectTransform;
use egui::epaint::Mesh;
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

pub fn show_viewport(
    ui: &mut egui::Ui,
    state: &mut ViewportState,
    app: &AppState,
) -> Option<PieceId> {
    let available = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(available, Sense::click_and_drag());
    let painter = ui.painter_at(rect);

    let sheet_w = (app.sheet_config.width as f32).max(1.0);
    let sheet_h = (app.sheet_config.height as f32).max(1.0);
    let sheet_count = app
        .current_result
        .as_ref()
        .map(|r| r.sheets.len().max(1))
        .unwrap_or(1);
    let gap = 100.0_f32;
    let total_width = sheet_count as f32 * sheet_w + (sheet_count.saturating_sub(1) as f32 * gap);

    // Compute uniform scale: fit full multi-sheet span into screen rect preserving aspect ratio
    let scale_x = rect.width() / total_width.max(1.0);
    let scale_y = rect.height() / sheet_h;
    let base_scale = scale_x.min(scale_y);
    let ppu = base_scale * state.zoom.max(0.05);

    // World-space visible extent, centered on full layout center + pan
    let vis_w = rect.width() / ppu;
    let vis_h = rect.height() / ppu;
    let center = Pos2::new(total_width * 0.5, sheet_h * 0.5) + state.pan;
    let world = Rect::from_center_size(center, Vec2::new(vis_w, vis_h));

    let to_screen = RectTransform::from_to(world, rect);

    if response.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll.abs() > 0.0 {
            let factor = (1.0 + scroll * 0.001).clamp(0.2, 4.0);
            state.zoom = (state.zoom * factor).clamp(0.1, 50.0);
        }
    }

    let dragging =
        ui.input(|i| i.pointer.middle_down() || (i.modifiers.ctrl && i.pointer.primary_down()));
    if dragging && response.dragged() {
        let delta = response.drag_delta();
        let x_scale = world.width() / rect.width().max(1.0);
        let y_scale = world.height() / rect.height().max(1.0);
        state.pan.x -= delta.x * x_scale;
        state.pan.y -= delta.y * y_scale;
    }

    let mut click_hit = None;

    for sheet_idx in 0..sheet_count {
        let x_offset = sheet_idx as f32 * (sheet_w + gap);
        painter.rect_stroke(
            to_screen.transform_rect(Rect::from_min_max(
                Pos2::new(x_offset, 0.0),
                Pos2::new(x_offset + sheet_w, sheet_h),
            )),
            0.0,
            Stroke::new(2.0, Color32::WHITE),
            egui::StrokeKind::Outside,
        );

        if let Some(result) = &app.current_result
            && let Some(sheet) = result.sheets.get(sheet_idx)
        {
            let label = format!(
                "Sheet {} ({:.0}%)",
                sheet_idx + 1,
                sheet.utilization * 100.0
            );
            let label_pos = to_screen.transform_pos(Pos2::new(x_offset, -24.0));
            painter.text(
                label_pos,
                egui::Align2::LEFT_BOTTOM,
                label,
                egui::FontId::proportional(14.0),
                Color32::WHITE,
            );

            let used_x = x_offset + sheet.strip_length.clamp(0.0, app.sheet_config.width) as f32;
            let a = to_screen.transform_pos(Pos2::new(used_x, 0.0));
            let b = to_screen.transform_pos(Pos2::new(used_x, sheet_h));
            painter.line_segment([a, b], Stroke::new(2.0, Color32::from_rgb(255, 180, 0)));

            for placement in &sheet.placements {
                if let Some(piece) = app.pieces.iter().find(|p| p.id == placement.piece_id) {
                    let adjusted = Placement {
                        x: placement.x + x_offset as f64,
                        ..placement.clone()
                    };
                    let poly = placed_polygon(&piece.polygon, &adjusted);
                    let points = poly
                        .outer
                        .iter()
                        .map(|p| to_screen.transform_pos(Pos2::new(p.x as f32, p.y as f32)))
                        .collect::<Vec<_>>();

                    let mut fill = app.piece_color(&piece.source_file);
                    let selected = app.selected_piece == Some(piece.id);
                    if selected {
                        fill = fill.gamma_multiply(0.65);
                    }

                    if let Some(mesh) = triangulate_polygon(&points, fill) {
                        painter.add(Shape::mesh(mesh));
                    }

                    let stroke_width = if selected { 3.0 } else { 1.0 };
                    let mut outline = points.clone();
                    if !outline.is_empty() {
                        outline.push(outline[0]);
                    }
                    painter.add(Shape::line(
                        outline,
                        Stroke::new(stroke_width, Color32::BLACK),
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

fn triangulate_polygon(screen_pts: &[Pos2], color: Color32) -> Option<Mesh> {
    if screen_pts.len() < 3 {
        return None;
    }
    let coords: Vec<f64> = screen_pts
        .iter()
        .flat_map(|p| [p.x as f64, p.y as f64])
        .collect();
    let indices = earcutr::earcut(&coords, &[], 2).ok()?;
    if indices.is_empty() {
        return None;
    }
    let mut mesh = Mesh::default();
    for pt in screen_pts {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: *pt,
            uv: egui::Pos2::ZERO,
            color,
        });
    }
    for idx in indices {
        mesh.indices.push(idx as u32);
    }
    Some(mesh)
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
