mod app;
use app::BudgetApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Advanced Finance Tracker",
        native_options,
        Box::new(|_cc| Box::new(BudgetApp::new(None))),
    )
}