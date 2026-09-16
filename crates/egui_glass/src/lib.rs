//! Apple Liquid Glass style widgets for egui, rendered on the GPU through
//! egui-wgpu paint callbacks.
//!
//! ```ignore
//! # fn setup(cc: &eframe::CreationContext<'_>, wallpaper: egui::ColorImage) {
//! let rs = cc.wgpu_render_state.as_ref().unwrap();
//! egui_glass::init(rs, 1);
//! egui_glass::set_backdrop(&cc.egui_ctx, rs, &wallpaper);
//! # }
//! # fn ui(ui: &mut egui::Ui) {
//! egui_glass::show_backdrop(ui, ui.max_rect());
//! egui_glass::Glass::new(egui_glass::GlassStyle::regular()).show(ui, |ui| {
//!     ui.label("Hello");
//! });
//! # }
//! ```
//!
//! Glass refracts a *backdrop image* you register once (or whenever it changes),
//! not the live framebuffer: egui paints in a single pass, so the pixels behind a
//! widget are not available to a shader in the same frame.

mod backdrop;
mod renderer;
mod style;
mod widgets;

pub use backdrop::{backdrop_rect, backdrop_size, set_backdrop, show_backdrop, show_backdrop_mapped};
pub use renderer::init;
pub use style::GlassStyle;
pub use widgets::{paint_glass, Glass, GlassButton, GlassToolbar};
