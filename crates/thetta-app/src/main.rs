use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("Thetta"),
        ..Default::default()
    };
    eframe::run_native(
        "Thetta",
        options,
        Box::new(|_cc| Ok(Box::new(ThettaApp::default()))),
    )
}

#[derive(Default)]
struct ThettaApp {}

impl eframe::App for ThettaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Thetta");
            ui.label("Load a STEP file to begin.");
        });
    }
}
