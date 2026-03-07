use crate::state::AppState;

#[derive(Debug, Clone)]
pub enum PanelAction {
    LoadPath(String),
    ToggleRun,
    DeleteSelected,
    Renest,
}

pub fn show_side_panel(
    ctx: &egui::Context,
    app: &mut AppState,
    path_input: &mut String,
) -> Vec<PanelAction> {
    let mut actions = Vec::new();

    egui::SidePanel::left("left_panel")
        .resizable(true)
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("Thetta");
            ui.separator();

            ui.collapsing("Files", |ui| {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(path_input);
                    if ui.button("Load").clicked() {
                        actions.push(PanelAction::LoadPath(path_input.clone()));
                    }
                });

                for lf in &app.loaded_files {
                    ui.label(format!("{} ({})", lf.path.display(), lf.piece_count));
                }
            });

            ui.separator();
            ui.collapsing("Sheet", |ui| {
                let mut changed = false;
                changed |= ui
                    .add(egui::DragValue::new(&mut app.sheet_config.width).prefix("W "))
                    .changed();
                changed |= ui
                    .add(egui::DragValue::new(&mut app.sheet_config.height).prefix("H "))
                    .changed();
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut app.sheet_config.separation)
                            .prefix("Sep ")
                            .speed(0.1),
                    )
                    .changed();

                if changed {
                    actions.push(PanelAction::Renest);
                }
            });

            ui.separator();
            ui.collapsing("Pieces", |ui| {
                egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                    for piece in &app.pieces {
                        let selected = app.selected_piece == Some(piece.id);
                        if ui
                            .selectable_label(selected, format!("Piece {}", piece.id.0))
                            .clicked()
                        {
                            app.selected_piece = Some(piece.id);
                        }
                    }
                });

                if ui.button("Delete Selected").clicked() {
                    actions.push(PanelAction::DeleteSelected);
                }
            });

            ui.separator();
            ui.collapsing("Nesting", |ui| {
                ui.add(
                    egui::DragValue::new(&mut app.optimizer_config.time_limit_secs)
                        .speed(1.0)
                        .prefix("Time "),
                );

                let label = if app.optimizer_running { "Stop" } else { "Start" };
                if ui.button(label).clicked() {
                    actions.push(PanelAction::ToggleRun);
                }
            });

            ui.separator();
            if let Some(bench) = &app.benchmark {
                ui.collapsing("Benchmark", |ui| {
                    ui.label(format!("Instance: {}", bench.name));
                    ui.label(format!("Strip height: {:.3}", bench.strip_height));
                });
                ui.separator();
            }

            ui.separator();
            ui.collapsing("Stats", |ui| {
                ui.label(format!("Utilization: {:.2}%", app.stats.utilization_pct));
                ui.label(format!("Strip Length: {:.3}", app.stats.strip_length));
                ui.label(format!(
                    "Placed: {}/{}",
                    app.stats.pieces_placed, app.stats.pieces_total
                ));
                ui.label(format!("Elapsed: {:.1}s", app.stats.elapsed.as_secs_f64()));
                ui.label(format!("Updates: {}", app.stats.iterations));
            });
        });

    actions
}
