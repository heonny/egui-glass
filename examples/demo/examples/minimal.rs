//! The README example: the library alone, no demo scaffolding.
//! Run with `cargo run -p egui_glass_demo --example minimal`.

use eframe::egui;
use egui_glass::{Glass, GlassButton, GlassToolbar};

struct App;

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let rs = cc.wgpu_render_state.as_ref().expect("eframe must use the wgpu renderer");
        egui_glass::init(rs, 1);                                   // msaa samples of NativeOptions
        let wallpaper = egui::ColorImage::filled([256, 256], egui::Color32::from_rgb(70, 130, 180));
        egui_glass::set_backdrop(&cc.egui_ctx, rs, &wallpaper);    // any ColorImage: photo, gradient, ...
        Self
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
            egui_glass::show_backdrop(ui, ui.max_rect());          // draw the backdrop, record where it is
            egui::Area::new(egui::Id::new("card")).movable(true).constrain(false).show(ui.ctx(), |ui| {
                Glass::default().show(ui, |ui| {
                    ui.heading("Hello glass");
                    GlassButton::new("Continue").show(ui);
                });
            });
            egui::Area::new(egui::Id::new("bar")).movable(true).constrain(false).show(ui.ctx(), |ui| {
                GlassToolbar::default().show(ui, |ui| {
                    let _ = ui.button("Undo");
                    let _ = ui.button("Share");
                });
            });
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions { renderer: eframe::Renderer::Wgpu, ..Default::default() };
    eframe::run_native("glass", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
