mod app;
mod gui;
mod types;
mod utils;
mod data;
mod analytics;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Budget Tracking App",
        native_options,
        Box::new(|_cc| Box::new(app::BudgetApp::new(None))),
    )
}