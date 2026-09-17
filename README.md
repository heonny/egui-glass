<p align="center">
  <img src="https://raw.githubusercontent.com/heonny/egui-glass/main/assets/branding/readme-logo.png" alt="egui_glass" width="640">
</p>

[![CI](https://github.com/heonny/egui-glass/actions/workflows/ci.yml/badge.svg)](https://github.com/heonny/egui-glass/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/egui_glass.svg)](https://crates.io/crates/egui_glass)
[![API docs](https://docs.rs/egui_glass/badge.svg)](https://docs.rs/egui_glass)

Glass surfaces for [egui](https://github.com/emilk/egui), inspired by Apple's *Liquid Glass*:
edge refraction, frosted blur, tint, and a thin bevel highlight, rendered on the GPU through
`egui-wgpu`. Includes containers, capsule buttons, toolbars, light/dark presets, and an optional
live backdrop for refracting egui content. No animation is provided.

![demo](https://raw.githubusercontent.com/heonny/egui-glass/main/docs/demo.png)

## Quick start

Requires Rust 1.92+, egui/eframe 0.35, and wgpu 29. Use the **wgpu** renderer; glow is unsupported.
The `wayland` and `x11` features below enable Linux windowing.

```toml
[dependencies]
eframe = { version = "0.35", default-features = false, features = ["default_fonts", "wgpu", "wayland", "x11"] }
egui = "0.35"
egui_glass = "0.1.3"
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

## Before adopting

This is a **0.1 library**: suitable for experimenting with glass in native egui apps, with a
small API that may evolve before 1.0. Evaluate rendering and interaction on your target devices.

- Static glass samples the registered image; it does not automatically capture widgets behind it.
  `LiveBackdrop` renders page content again off screen, adding layout and GPU work each frame.
- Library tests run in CI on Linux, macOS, and Windows. Demo builds are checked on macOS and
  Windows. These are build/test checks, not a GPU rendering certification; browser and mobile
  targets are not covered by CI.
- Glass controls do not provide an opaque accessibility fallback. Check contrast, keyboard
  interaction, and assistive technology in your app; use standard egui controls when appropriate.
- There are no default crate features. Enable `features = ["serde"]` to serialize `GlassStyle`.

## Docs

- [Guide](https://github.com/heonny/egui-glass/blob/main/docs/guide.md) — put glass into an existing app: backdrop, components, presets, dark
  theme, glass over live content.
- [Reference](https://github.com/heonny/egui-glass/blob/main/docs/reference.md) — every function and `GlassStyle` field, how the shader works,
  limits, the demo app, building the macOS app, publishing.
- [docs.rs](https://docs.rs/egui_glass) — API docs.
- [Contributing](https://github.com/heonny/egui-glass/blob/main/CONTRIBUTING.md) — development, bug reports, and release checks.
- [Changelog](https://github.com/heonny/egui-glass/blob/main/CHANGELOG.md) — changes for the next release.

## Demo

```bash
cargo run -p egui_glass_demo          # sliders for every parameter, presets, export/import
cargo run -p egui_glass_demo --example minimal
make install                           # macOS: Egui Glass.app into /Applications
```

## License

MIT. "Liquid Glass" is Apple's name for its design language; this is an independent
reimplementation of the look and is not affiliated with Apple.
See [asset provenance](https://github.com/heonny/egui-glass/blob/main/docs/assets.md) for the demo
images and branding; these assets are not included in the library crate.
