use crate::state::AppState;
use thetta_geometry::types::{MaterialType, Tool};

#[derive(Debug, Clone)]
pub enum PanelAction {
    OpenStepDialog,
    SelectThickness(usize),
    OpenExportDialog,
    ToggleRun,
    DeleteSelected,
    Renest,
    RegenerateTabs,
    SaveToolTable,
    OpenGcodeDialog,
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
            egui::CollapsingHeader::new("Tabs")
                .default_open(true)
                .show(ui, |ui| {
                    let cfg = &mut app.tab_state.config;
                    let mut config_changed = false;

                    egui::ComboBox::from_label("Material")
                        .selected_text(match cfg.material_type {
                            MaterialType::WoodSoft => "Wood (Soft)",
                            MaterialType::WoodHard => "Wood (Hard)",
                            MaterialType::Plastic => "Plastic",
                            MaterialType::Composite => "Composite",
                        })
                        .show_ui(ui, |ui| {
                            config_changed |= ui
                                .selectable_value(
                                    &mut cfg.material_type,
                                    MaterialType::WoodSoft,
                                    "Wood (Soft)",
                                )
                                .changed();
                            config_changed |= ui
                                .selectable_value(
                                    &mut cfg.material_type,
                                    MaterialType::WoodHard,
                                    "Wood (Hard)",
                                )
                                .changed();
                            config_changed |= ui
                                .selectable_value(
                                    &mut cfg.material_type,
                                    MaterialType::Plastic,
                                    "Plastic",
                                )
                                .changed();
                            config_changed |= ui
                                .selectable_value(
                                    &mut cfg.material_type,
                                    MaterialType::Composite,
                                    "Composite",
                                )
                                .changed();
                        });

                    config_changed |= ui
                        .add(
                            egui::Slider::new(&mut cfg.material_thickness, 1.0..=50.0)
                                .text("Thickness (mm)"),
                        )
                        .changed();
                    config_changed |= ui.checkbox(&mut cfg.auto_size, "Auto size").changed();

                    if cfg.auto_size {
                        ui.label(format!("Tab width: {:.2} mm", cfg.effective_tab_width()));
                        ui.label(format!("Tab height: {:.2} mm", cfg.effective_tab_height()));
                    } else {
                        config_changed |= ui
                            .add(
                                egui::Slider::new(&mut cfg.tab_width, 3.0..=30.0)
                                    .text("Tab width (mm)"),
                            )
                            .changed();
                        config_changed |= ui
                            .add(
                                egui::Slider::new(&mut cfg.tab_height, 1.0..=20.0)
                                    .text("Tab height (mm)"),
                            )
                            .changed();
                    }

                    config_changed |= ui
                        .add(
                            egui::Slider::new(&mut cfg.tab_distance, 10.0..=200.0)
                                .text("Distance (mm)"),
                        )
                        .changed();
                    config_changed |= ui
                        .add(
                            egui::DragValue::new(&mut cfg.min_tabs)
                                .range(1..=50)
                                .prefix("Min tabs "),
                        )
                        .changed();
                    config_changed |= ui
                        .add(
                            egui::DragValue::new(&mut cfg.max_tabs)
                                .range(1..=50)
                                .prefix("Max tabs "),
                        )
                        .changed();
                    config_changed |= ui
                        .add(
                            egui::Slider::new(&mut cfg.corner_exclusion, 0.0..=20.0)
                                .text("Corner exclusion (mm)"),
                        )
                        .changed();
                    config_changed |= ui
                        .add(
                            egui::Slider::new(&mut cfg.size_threshold, 0.0..=100.0)
                                .text("Size threshold (mm)"),
                        )
                        .changed();

                    if cfg.min_tabs > cfg.max_tabs {
                        cfg.max_tabs = cfg.min_tabs;
                        config_changed = true;
                    }

                    if ui.button("Regenerate Tabs").clicked() {
                        actions.push(PanelAction::RegenerateTabs);
                    }
                    if config_changed {
                        actions.push(PanelAction::RegenerateTabs);
                    }

                    let total_tabs: usize = app.tab_state.results.values().map(Vec::len).sum();
                    let tabbed_pieces = app.tab_state.results.len();
                    let total_placements = app
                        .current_result
                        .as_ref()
                        .map(|result| result.sheets.iter().map(|s| s.placements.len()).sum())
                        .unwrap_or(0usize);
                    let skipped = total_placements.saturating_sub(tabbed_pieces);
                    ui.label(format!(
                        "{total_tabs} tabs across {tabbed_pieces} pieces ({skipped} pieces skipped)"
                    ));
                });

            ui.separator();
            egui::CollapsingHeader::new("Tool Database")
                .default_open(true)
                .show(ui, |ui| {
                    let mut changed = false;

                    if ui.button("Add Tool").clicked() {
                        app.tool_table.tools.push(Tool {
                            number: app.tool_table.next_number(),
                            name: "New Tool".to_string(),
                            diameter: 6.0,
                            flute_count: 2,
                            feed_rate: 1500.0,
                            plunge_rate: 500.0,
                            spindle_speed: 18000,
                        });
                        changed = true;
                    }

                    let mut remove_index = None;
                    for (idx, tool) in app.tool_table.tools.iter_mut().enumerate() {
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label(format!("T{}", tool.number));
                            changed |= ui
                                .add(egui::DragValue::new(&mut tool.number).range(1..=999))
                                .changed();
                            changed |= ui.text_edit_singleline(&mut tool.name).changed();
                            if ui.button("Delete").clicked() {
                                remove_index = Some(idx);
                            }
                        });
                        ui.horizontal(|ui| {
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut tool.diameter)
                                        .speed(0.1)
                                        .prefix("Dia "),
                                )
                                .changed();
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut tool.flute_count)
                                        .range(1..=16)
                                        .prefix("Flutes "),
                                )
                                .changed();
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut tool.feed_rate)
                                        .speed(10.0)
                                        .prefix("Feed "),
                                )
                                .changed();
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut tool.plunge_rate)
                                        .speed(10.0)
                                        .prefix("Plunge "),
                                )
                                .changed();
                            changed |= ui
                                .add(
                                    egui::DragValue::new(&mut tool.spindle_speed)
                                        .range(1..=50000)
                                        .prefix("RPM "),
                                )
                                .changed();
                        });
                    }

                    if let Some(idx) = remove_index {
                        app.tool_table.tools.remove(idx);
                        changed = true;
                    }

                    if let Err(err) = app.tool_table.validate() {
                        ui.colored_label(egui::Color32::from_rgb(200, 60, 60), err);
                    }

                    if changed {
                        actions.push(PanelAction::SaveToolTable);
                    }
                });

            ui.separator();
            egui::CollapsingHeader::new("G-code")
                .default_open(true)
                .show(ui, |ui| {
                    egui::ComboBox::from_label("Tool")
                        .selected_text(
                            app.gcode_config
                                .tool_number
                                .and_then(|n| app.tool_table.by_number(n))
                                .map(|t| format!("T{} {}", t.number, t.name))
                                .unwrap_or_else(|| "Select tool".to_string()),
                        )
                        .show_ui(ui, |ui| {
                            for tool in &app.tool_table.tools {
                                ui.selectable_value(
                                    &mut app.gcode_config.tool_number,
                                    Some(tool.number),
                                    format!("T{} {} ({:.1}mm)", tool.number, tool.name, tool.diameter),
                                );
                            }
                        });

                    ui.add(
                        egui::Slider::new(&mut app.gcode_config.safe_z, 1.0..=20.0).text("Safe Z (mm)"),
                    );
                    ui.add(
                        egui::Slider::new(&mut app.gcode_config.cut_depth, 1.0..=50.0)
                            .text("Cut depth (mm)"),
                    );
                    ui.add(
                        egui::Slider::new(&mut app.profile_config.lead_in_radius, 0.5..=10.0)
                            .text("Lead radius (mm)"),
                    );

                    if let Some(status) = &app.gcode_status {
                        ui.label(status);
                    }
                    if let Some(err) = &app.gcode_error {
                        ui.colored_label(egui::Color32::from_rgb(200, 60, 60), err);
                    }

                    let precondition_error = app.gcode_export_precondition_error();
                    let can_export = precondition_error.is_none() && !app.gcode_exporting;
                    if let Some(err) = precondition_error {
                        ui.label(err);
                    }
                    if app.gcode_exporting {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Generating G-code...");
                        });
                    }
                    if ui
                        .add_enabled(can_export, egui::Button::new("Generate G-code"))
                        .clicked()
                    {
                        actions.push(PanelAction::OpenGcodeDialog);
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
