mod gui;
mod data;
mod analytics;
mod utils;
mod types;

pub use types::*;
pub use data::*;
pub use analytics::*;
pub use gui::*;
pub use utils::*;

use std::path::PathBuf;
use chrono::Local;
use eframe::epi;

pub struct BudgetApp {
    pub state: AppState,
}

impl BudgetApp {
    pub fn new(file_path: Option<PathBuf>) -> Self {
        let state = AppState::load_or_default(file_path);
        Self { state }
    }
}

impl epi::App for BudgetApp {
    fn name(&self) -> &str {
        "Advanced Finance Tracker"
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        gui::draw_main_window(self, ctx, frame);
    }
}