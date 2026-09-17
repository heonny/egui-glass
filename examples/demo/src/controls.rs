use std::ops::RangeInclusive;

use eframe::egui::{self, Color32, Margin, Vec2};
use egui_glass::{Glass, GlassButton, GlassSlider, GlassStyle};

use crate::App;

impl App {
    pub(super) fn controls_panel(&mut self, ui: &mut egui::Ui) {
        let panel_style = if self.dark { GlassStyle::panel_dark() } else { GlassStyle::panel() };
        egui::Panel::left("controls").exact_size(320.0)
            .frame(egui::Frame::NONE.fill(self.paper()).inner_margin(Margin::same(10)))
            .show(ui, |ui| {
                Glass::new(panel_style).inner_margin(Margin::same(18)).show(ui, |ui| {
                    ui.set_min_height(ui.available_height());
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                    self.controls_header(ui);
                    ui.add_space(8.0);
                    ui.separator();
                    let height = (ui.available_height() - 80.0).max(60.0);
                    egui::ScrollArea::vertical().id_salt("parameters").auto_shrink([false, false]).max_height(height).show(ui, |ui| {
                        self.controls_parameters(ui);
                        if let Some(status) = &self.status {
                            ui.add_space(12.0);
                            ui.label(egui::RichText::new(status).small().weak());
                        }
                    });
                    ui.separator();
                    let button_style = if self.dark { GlassStyle::dark() } else { GlassStyle::regular() };
                    ui.horizontal(|ui| {
                        let size = Vec2::new((ui.available_width() - 8.0) / 2.0, 40.0);
                        if GlassButton::new("Import").style(button_style).min_size(size).show(ui).clicked() { self.import_settings(); }
                        if GlassButton::new("Export").style(button_style).min_size(size).show(ui).clicked() { self.export_settings(); }
                    });
                });
            });
    }

    fn controls_header(&mut self, ui: &mut egui::Ui) {
        let accent = if self.dark { Color32::from_rgb(111, 209, 225) } else { Color32::from_rgb(30, 127, 150) };
        ui.label(egui::RichText::new("MATERIAL STUDIO").size(10.0).strong().color(accent));
        ui.label(egui::RichText::new("egui_glass").size(26.0).strong());
        ui.label(egui::RichText::new("Shape light. Make it yours.").weak());
        ui.add_space(10.0);
        for presets in [
            [("Regular", GlassStyle::regular()), ("Clear", GlassStyle::clear())],
            [("Dark", GlassStyle::dark()), ("Panel", GlassStyle::panel())],
        ] {
            ui.horizontal(|ui| {
                let width = (ui.available_width() - 8.0) / 2.0;
                for (name, preset) in presets {
                    let selected = self.style == preset;
                    let mut button = egui::Button::new(name).selected(selected);
                    if selected { button = button.fill(accent.gamma_multiply(if self.dark { 0.3 } else { 0.14 })); }
                    if ui.add_sized([width, 30.0], button).clicked() {
                        self.style = preset;
                    }
                }
            });
        }
        ui.add_space(6.0);
        ui.checkbox(&mut self.live_mode, "Live backdrop").on_hover_text("Refract the photo and page content. Adds an off-screen render pass.");
    }

    fn controls_parameters(&mut self, ui: &mut egui::Ui) {
        let material = if self.dark { GlassStyle::dark() } else { GlassStyle::regular() };
        let slider = |ui: &mut egui::Ui, value: &mut f32, range: RangeInclusive<f32>, name: &str, unit: &str| {
            GlassSlider::new(value, range).text(name).suffix(unit).style(material).show(ui);
        };
        let s = &mut self.style;
        section(ui, "Material", true, |ui| {
            slider(ui, &mut s.blur, 0.0..=80.0, "Blur", " pt");
            slider(ui, &mut s.brightness, 0.5..=1.6, "Brightness", "×");
            slider(ui, &mut s.saturation, 0.0..=2.0, "Saturation", "×");
            ui.horizontal(|ui| {
                ui.label("Tint");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.color_edit_button_srgba(&mut s.tint); });
            });
        });
        section(ui, "Shape", true, |ui| {
            slider(ui, &mut s.corner_radius, 0.0..=80.0, "Corner radius", " pt");
            slider(ui, &mut s.corner_smoothing, 0.0..=1.0, "Smoothing", "");
        });
        section(ui, "Refraction", false, |ui| {
            slider(ui, &mut s.refraction, 0.0..=60.0, "Refraction", " pt");
            slider(ui, &mut s.edge_width, 1.0..=120.0, "Edge width", " pt");
            slider(ui, &mut s.chromatic, 0.0..=1.0, "Chromatic split", "");
        });
        section(ui, "Light & shadow", false, |ui| {
            slider(ui, &mut s.specular, 0.0..=1.0, "Highlight", "");
            slider(ui, &mut s.border, 0.0..=1.0, "Border", "");
            slider(ui, &mut s.shadow, 0.0..=1.0, "Shadow", "");
            slider(ui, &mut s.shadow_radius, 0.0..=60.0, "Shadow radius", " pt");
            slider(ui, &mut s.shadow_offset, -30.0..=30.0, "Shadow offset", " pt");
            slider(ui, &mut s.shadow_spread, -10.0..=30.0, "Shadow spread", " pt");
        });
    }
}

fn section(ui: &mut egui::Ui, title: &str, open: bool, contents: impl FnOnce(&mut egui::Ui)) {
    egui::CollapsingHeader::new(egui::RichText::new(title).strong()).default_open(open).show(ui, |ui| {
        ui.add_space(6.0);
        contents(ui);
        ui.add_space(6.0);
    });
}
