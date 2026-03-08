mod native_dialog;
mod panels;
mod state;
mod viewport;

use panels::{PanelAction, show_side_panel};
use state::AppState;
use viewport::{ViewportState, show_viewport};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("Thetta"),
        ..Default::default()
    };

    eframe::run_native(
        "Thetta",
        options,
        Box::new(|_cc| {
            Ok(Box::new(ThettaApp {
                state: AppState::default(),
                viewport_state: ViewportState::default(),
                error_text: None,
            }))
        }),
    )
}

struct ThettaApp {
    state: AppState,
    viewport_state: ViewportState,
    error_text: Option<String>,
}

impl eframe::App for ThettaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.poll_updates();
        self.state.poll_step_import();
        self.state.poll_step_export();

        // Poll file dialogs (non-blocking)
        if let Some(path) = self.state.poll_step_dialog() {
            self.error_text = None;
            self.state.start_step_import(path);
        }
        if let Some((out_dir, file_stem)) = self.state.poll_export_dialog() {
            self.state.start_step_export(out_dir, file_stem);
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
            self.state.delete_selected_piece();
        }

        let actions = show_side_panel(ctx, &mut self.state);
        for action in actions {
            match action {
                PanelAction::OpenStepDialog => {
                    self.state.step_dialog_rx = Some(native_dialog::request_step_file());
                }
                PanelAction::SelectThickness(idx) => {
                    self.state.select_thickness(idx);
                }
                PanelAction::OpenExportDialog => {
                    let default_name = self
                        .state
                        .step_source_path
                        .as_ref()
                        .and_then(|p| p.file_stem())
                        .and_then(|s| s.to_str())
                        .map(|s| format!("{s}_nested.step"))
                        .unwrap_or_else(|| "nested.step".to_string());
                    self.state.export_dialog_rx =
                        Some(native_dialog::request_export_target(default_name));
                }
                PanelAction::ToggleRun => {
                    if self.state.optimizer_running {
                        self.state.stop_optimizer();
                    } else {
                        self.state.start_optimizer();
                    }
                }
                PanelAction::DeleteSelected => {
                    self.state.delete_selected_piece();
                }
                PanelAction::Renest => {
                    self.state.start_optimizer();
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(err) = &self.error_text {
                ui.colored_label(egui::Color32::from_rgb(200, 60, 60), err);
            }
            if let Some(hit) = show_viewport(ui, &mut self.viewport_state, &self.state) {
                self.state.selected_piece = Some(hit);
            }
        });

        ctx.request_repaint();
    }
}
