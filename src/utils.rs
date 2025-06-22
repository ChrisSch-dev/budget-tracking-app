use crate::types::Theme;
use eframe::egui;

impl super::types::AppState {
    pub fn set_theme(&self, ctx: &egui::Context) {
        match self.theme {
            Theme::Light => ctx.set_visuals(egui::Visuals::light()),
            Theme::Dark => ctx.set_visuals(egui::Visuals::dark()),
        }
    }
}