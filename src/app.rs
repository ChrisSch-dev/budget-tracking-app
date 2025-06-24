use crate::types::*;
use crate::gui;

pub struct BudgetApp {
    pub state: AppState,
}

impl BudgetApp {
    pub fn new(file_path: Option<std::path::PathBuf>) -> Self {
        let state = AppState::load_or_default(file_path);
        Self { state }
    }
}

impl eframe::App for BudgetApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        gui::draw_main_window(self, ctx, frame);
    }
}