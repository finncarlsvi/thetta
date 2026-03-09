/// Minimal test: does rfd file dialog work from within eframe?
/// Run with: cargo run -p thetta-app --example test_dialog

fn main() -> eframe::Result {
    eframe::run_native(
        "Dialog Test",
        eframe::NativeOptions::default(),
        Box::new(|_cc| {
            Ok(Box::new(DialogTest {
                rx: None,
                result: "Click the button".to_string(),
            }))
        }),
    )
}

struct DialogTest {
    rx: Option<std::sync::mpsc::Receiver<Option<std::path::PathBuf>>>,
    result: String,
}

impl eframe::App for DialogTest {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll for dialog result
        if let Some(rx) = &self.rx {
            match rx.try_recv() {
                Ok(Some(path)) => {
                    self.result = format!("Selected: {}", path.display());
                    self.rx = None;
                }
                Ok(None) => {
                    self.result = "Cancelled".to_string();
                    self.rx = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.result = "Dialog thread died".to_string();
                    self.rx = None;
                }
                _ => {}
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File Dialog Test");
            ui.label(&self.result);

            let busy = self.rx.is_some();
            if ui
                .add_enabled(!busy, egui::Button::new("Open File Dialog"))
                .clicked()
            {
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let result = rfd::FileDialog::new()
                        .set_title("Pick any file")
                        .pick_file();
                    let _ = tx.send(result);
                });
                self.rx = Some(rx);
                self.result = "Dialog open...".to_string();
            }

            if busy {
                ui.spinner();
            }
        });

        ctx.request_repaint();
    }
}
