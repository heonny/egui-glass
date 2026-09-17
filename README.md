<p align="center">
  <img src="https://raw.githubusercontent.com/heonny/egui-glass/main/assets/branding/readme-logo.png" alt="egui_glass" width="640">
</p>

Apple *Liquid Glass* style surfaces for [egui](https://github.com/emilk/egui), drawn on the GPU
through `egui-wgpu`: refraction at the edge, frosted blur, tint, a thin bevel highlight. No
animation. Tuned side by side with iOS 26 screenshots, so the defaults are the look.

![demo](https://raw.githubusercontent.com/heonny/egui-glass/main/docs/demo.jpg)

## Quick start

Requires Rust 1.92+ and `eframe` with the **wgpu** renderer.

```toml
[dependencies]
eframe = { version = "0.35", default-features = false, features = ["default_fonts", "wgpu"] }
egui = "0.35"
egui_glass = "0.1"
```

```rust
use eframe::egui;
use egui_glass::{Glass, GlassButton, GlassToolbar};

struct App;

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let rs = cc.wgpu_render_state.as_ref().expect("eframe must use the wgpu renderer");
        egui_glass::init(rs, 1);
        let wallpaper = egui::ColorImage::filled([256, 256], egui::Color32::from_rgb(70, 130, 180));
        egui_glass::set_backdrop(&cc.egui_ctx, rs, &wallpaper);   // what the glass refracts
        Self
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
            egui_glass::show_backdrop(ui, ui.max_rect());
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
```

That is the whole integration: `init` once, give the glass something to refract, draw glass.

## Docs

- [Guide](docs/guide.md) — put glass into an existing app: backdrop, components, presets, dark
  theme, glass over live content.
- [Reference](docs/reference.md) — every function and `GlassStyle` field, how the shader works,
  limits, the demo app, building the macOS app, publishing.
- [docs.rs](https://docs.rs/egui_glass) — API docs.

## Demo

```bash
cargo run -p egui_glass_demo          # sliders for every parameter, presets, export/import
cargo run -p egui_glass_demo --example minimal
make install                           # macOS: Egui Glass.app into /Applications
```

## License

MIT. "Liquid Glass" is Apple's name for its design language; this is an independent
reimplementation of the look and is not affiliated with Apple.
