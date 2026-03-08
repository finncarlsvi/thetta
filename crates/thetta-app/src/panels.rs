use crate::state::AppState;

#[derive(Debug, Clone)]
pub enum PanelAction {
    OpenStepDialog,
    SelectThickness(usize),
    OpenExportDialog,
    ToggleRun,
    DeleteSelected,
    Renest,
}

pub fn show_side_panel(ctx: &egui::Context, app: &mut AppState) -> Vec<PanelAction> {
    let mut actions = Vec::new();

    egui::SidePanel::left("left_panel")
        .resizable(true)
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("Thetta");
            ui.separator();

            egui::CollapsingHeader::new("STEP")
                .default_open(true)
                .show(ui, |ui| {
                    let dialog_busy = app.step_dialog_rx.is_some();
                    if ui
                        .add_enabled(
                            !dialog_busy && !app.step_loading,
                            egui::Button::new("Load STEP"),
                        )
                        .clicked()
                    {
                        actions.push(PanelAction::OpenStepDialog);
                    }

                    if app.step_loading {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Importing STEP...");
                        });
                    }

                    if let Some(err) = &app.step_error {
                        ui.colored_label(egui::Color32::from_rgb(200, 60, 60), err);
                    }
                    if let Some(status) = &app.step_export_status {
                        ui.label(status);
                    }

                    if !app.thickness_groups.is_empty() {
                        ui.label("Thickness groups");
                        for (idx, group) in app.thickness_groups.iter().enumerate() {
                            let label =
                                format!("{:.2} mm ({} solids)", group.value_mm, group.count);
                            if ui
                                .radio_value(&mut app.selected_thickness_idx, idx, label)
                                .clicked()
                            {
                                actions.push(PanelAction::SelectThickness(idx));
                            }
                        }
                    }

                    let can_export = app.current_result.is_some()
                        && app.step_session.is_some()
                        && !app.step_exporting
                        && app.export_dialog_rx.is_none();
                    if ui
                        .add_enabled(can_export, egui::Button::new("Export STEP"))
                        .clicked()
                    {
                        actions.push(PanelAction::OpenExportDialog);
                    }
                    if app.step_exporting {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Exporting...");
                        });
                    }
                });

            ui.separator();
            egui::CollapsingHeader::new("Sheet")
                .default_open(true)
                .show(ui, |ui| {
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
            egui::CollapsingHeader::new("Pieces")
                .default_open(true)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(180.0)
                        .show(ui, |ui| {
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
            egui::CollapsingHeader::new("Nesting")
                .default_open(true)
                .show(ui, |ui| {
                    let label = if app.optimizer_running {
                        "Stop"
                    } else {
                        "Start"
                    };
                    if ui.button(label).clicked() {
                        actions.push(PanelAction::ToggleRun);
                    }
                });

            ui.separator();
            if let Some(bench) = &app.benchmark {
                egui::CollapsingHeader::new("Benchmark")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(format!("Instance: {}", bench.name));
                        ui.label(format!("Strip height: {:.3}", bench.strip_height));
                    });
                ui.separator();
            }

            ui.separator();
            egui::CollapsingHeader::new("Stats")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!("Utilization: {:.2}%", app.stats.utilization_pct));
                    ui.label(format!("Strip Length: {:.3}", app.stats.strip_length));
                    ui.label(format!(
                        "Placed: {}/{} on {} sheet{}",
                        app.stats.pieces_placed,
                        app.stats.pieces_total,
                        app.stats.sheet_count.max(1),
                        if app.stats.sheet_count.max(1) == 1 {
                            ""
                        } else {
                            "s"
                        }
                    ));
                    if app.stats.sheet_count > 1 {
                        ui.label(format!("Sheets: {}", app.stats.sheet_count));
                    }
                    if app.stats.too_large_for_sheet > 0 {
                        ui.colored_label(
                            egui::Color32::from_rgb(220, 120, 80),
                            format!(
                                "{} pieces too large for sheet",
                                app.stats.too_large_for_sheet
                            ),
                        );
                    }
                    ui.label(format!("Elapsed: {:.1}s", app.stats.elapsed.as_secs_f64()));
                    ui.label(format!("Solutions: {}", app.stats.iterations));
                    if app.optimizer_running {
                        ui.separator();
                        ui.label(format!("GLS iter: {}", app.stats.gls_iteration));
                        ui.label(format!("Overlap: {:.0}", app.stats.gls_overlap));
                        ui.label(format!("Best overlap: {:.0}", app.stats.gls_best_overlap));
                        ui.label(format!("Feasible found: {}", app.stats.gls_feasible_count));
                    }
                });
        });

    actions
}
