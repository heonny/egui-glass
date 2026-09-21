<p align="center">
  <img src="https://raw.githubusercontent.com/heonny/egui-glass/main/assets/branding/readme-logo.png" alt="egui_glass" width="640">
</p>

[![CI](https://github.com/heonny/egui-glass/actions/workflows/ci.yml/badge.svg)](https://github.com/heonny/egui-glass/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/egui_glass.svg)](https://crates.io/crates/egui_glass)
[![API docs](https://docs.rs/egui_glass/badge.svg)](https://docs.rs/egui_glass)

Glass surfaces for [egui](https://github.com/emilk/egui), inspired by Apple's *Liquid Glass*:
edge refraction, frosted blur, tint, and a thin bevel highlight, rendered on the GPU through
`egui-wgpu`. Includes containers, capsule buttons, glass sliders, toolbars, light/dark presets, and an optional
live backdrop for refracting egui content. The development version adds short hover/press
material transitions for buttons and sliders; released 0.1.5 switches states instantly.

![demo](https://raw.githubusercontent.com/heonny/egui-glass/main/docs/demo.png)

## Quick start

Requires Rust 1.92+, egui/eframe 0.35, and wgpu 29. Use the **wgpu** renderer; glow is unsupported.
The `wayland` and `x11` features below enable Linux windowing. A wgpu-compatible graphics
adapter is required. Start with `cargo new glass-app`, replace its dependencies with the
following, put the Rust example in `src/main.rs`, and run `cargo run`. No assets or local
checkout of this repository are needed.

```toml
[dependencies]
eframe = { version = "0.35", default-features = false, features = ["accesskit", "default_fonts", "wgpu", "wayland", "x11"] }
egui = "0.35"
egui_glass = "0.1.5"
```

```rust
use eframe::egui;
use egui_glass::{Glass, GlassToolbar};

struct App;

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let rs = cc.wgpu_render_state.as_ref().expect("eframe must use the wgpu renderer");
        egui_glass::init(rs, 1);
        // A gradient makes refraction visible; a uniform colour looks almost flat.
        let mut wallpaper = egui::ColorImage::filled([256, 256], egui::Color32::WHITE);
        for y in 0..256 {
            for x in 0..256 {
                wallpaper[(x, y)] = egui::Color32::from_rgb(70 + x as u8 / 2, 130, 180 + y as u8 / 4);
            }
        }
        egui_glass::set_backdrop(&cc.egui_ctx, rs, &wallpaper);   // what the glass refracts
        Self
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
            egui_glass::show_backdrop(ui, ui.max_rect());
            egui::Area::new(egui::Id::new("card"))
                .default_pos(egui::pos2(40.0, 40.0)).movable(true).constrain(false)
                .show(ui.ctx(), |ui| {
                Glass::default().show(ui, |ui| {
                    ui.heading("Hello glass");
                    ui.label("Drag this card to see the background change.");
                    let _ = ui.button("Continue"); // ordinary controls inside glass
                });
            });
            egui::Area::new(egui::Id::new("bar"))
                .default_pos(egui::pos2(40.0, 200.0)).movable(true).constrain(false)
                .show(ui.ctx(), |ui| {
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

`init` runs once per renderer; `set_backdrop` uploads once or when the image changes;
`show_backdrop` runs before glass each frame. The `1` passed to `init` matches the default
MSAA setting (`NativeOptions::multisampling = 0` means one sample). If you enable MSAA,
pass the same sample count to both. Keep the renderer alive for the lifetime of the UI.

## Choose what the glass sees

| Backdrop | What appears through glass | Cost and use |
|---|---|---|
| Static image, as above | Only the registered image at that screen position | Upload once; useful for sidebars, cards and dialogs over a gradient or wallpaper |
| `LiveBackdrop` | Page content rendered by its closure, including text | Extra layout and GPU rendering every frame; useful for floating controls over changing content |

For example, if a log line is painted over a static wallpaper, glass over that line still
shows **the wallpaper**, not a blurred log line. Increasing `blur` does not change this.
Static glass is not OS window transparency and cannot sample the desktop or other apps.

For a live backdrop, store a `LiveBackdrop` in your app. Create it after `init`, before
`set_backdrop` or native texture registration. This is the replacement for the static
backdrop drawing step; keep glass outside the page closure:

```rust
// In the creation closure, after init (rs is the wgpu render state):
let live = egui_glass::LiveBackdrop::new(rs, None); // default fonts

// In App::ui, using the stored live value:
live.run(ui, egui::Color32::WHITE, |page| {
    page.label("This text is part of the refracted page.");
});
egui::Area::new(egui::Id::new("overlay"))
    .fixed_pos(egui::pos2(20.0, 20.0)).constrain(false)
    .show(ui.ctx(), |ui| {
        egui_glass::Glass::panel().show(ui, |ui| { ui.label("Floating glass"); });
    });
```

The page closure runs for visible and off-screen layout, potentially again if egui requests
another pass: keep file writes, network calls, state transitions and other
side effects outside it. Create one live backdrop per renderer. For custom fonts, pass
`Some(font_definitions)` and keep them synchronized with the main context. Images in the
page must use textures registered with `egui_glass::register_native_texture` (`set_backdrop`
does this for its image); ordinary egui texture uploads are not automatically mirrored.

The development version adds per-frame live backdrop quality (not yet released):

```rust
live.run_with_quality(ui, egui::Color32::WHITE, egui_glass::LiveBackdropQuality::Balanced, |page| {
    page.label("This text is part of the refracted page.");
});
```

`Full` uses native resolution, `Balanced` uses 0.75× width and height, and `Performance`
uses 0.5×. The existing `run` method stays at `Full`. Lower settings reduce off-screen
pixel and mipmap work while keeping the visible page and glass geometry at native resolution.
Small refracted text becomes softer; the extra UI layout cost remains. See the
[benchmark instructions](https://github.com/heonny/egui-glass/blob/main/docs/performance.md) for measurement scope and local results.

## Add glass to an existing app

Use the initialization and dependencies above. Start with navigation surfaces or transient
overlays; keep dense text and code on opaque surfaces when contrast matters. Glass does not
replace your application's event handling or persistence.

### Sidebar and opaque content

This code goes in `App::ui`. Draw one backdrop over the whole root UI **before** allocating
panels so every surface uses the same screen mapping. Transparent outer panel frames let
that backdrop show through. `Glass` supplies its own padding (16 points by default); avoid
adding a second padded frame unless you want both margins.

```rust
egui_glass::show_backdrop(ui, ui.max_rect());
egui::Panel::left("navigation")
    .default_size(220.0)
    .frame(egui::Frame::NONE)
    .show(ui, |ui| {
        egui_glass::Glass::panel().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
            ui.set_min_size(ui.available_size()); // fill the panel, even with little content
            ui.heading("Projects");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for name in ["Frontend", "API", "Worker"] {
                    let _ = ui.button(name);
                }
            });
        });
    });
egui::CentralPanel::default()
    .frame(egui::Frame::new().fill(egui::Color32::WHITE))
    .show(ui, |ui| { ui.label("Opaque log or editor content"); });
```

### Modal with scrolling and dismissal

Keep your existing modal open flag and action handling. Use `Frame::NONE` on the modal
and put glass inside it; the modal still supplies input blocking and click-away/Escape
dismissal. The inner `ScrollArea` scrolls the form, while its footer stays visible.

```rust
// In App::ui, after drawing the page. `open` is your &mut bool modal state.
if *open {
    let response = egui::Modal::new(egui::Id::new("settings"))
        .frame(egui::Frame::NONE)
        .show(ui.ctx(), |ui| {
            egui_glass::Glass::panel().inner_margin(egui::Margin::same(20)).show(ui, |ui| {
                ui.set_width(320.0);
                ui.heading("Settings");
                let body_height = (ui.ctx().content_rect().height() - 160.0).max(80.0);
                egui::ScrollArea::vertical().max_height(body_height).show(ui, |ui| {
                    for n in 1..=20 { ui.label(format!("Setting {n}")); }
                });
                ui.button("Close").clicked()
            }).inner
        });
    if response.should_close() || response.inner { *open = false; }
}
```

### Preserve your existing control theme

`Glass::show` and `GlassToolbar::show` apply flat visuals to child controls: transparent
idle fills, capsule corners, changed foreground and selection colours, and no widget
borders. This is local to their child UI. On the development version (not yet released),
use `.preserve_theme(true)` to retain your app's existing buttons, validation colours and
focus styling:

```rust
egui_glass::Glass::panel().preserve_theme(true).show(ui, |ui| {
    let _ = ui.button("Uses the app's theme");
});
```

`GlassToolbar` supports the same option. The default is `false`, keeping flat child visuals.
This option preserves control visuals; glass material, padding and toolbar spacing still
come from the glass container. In the released **0.1.5** version, snapshot the style
**before** entering the container instead:

```rust
let app_style = ui.style().clone();
egui_glass::Glass::panel().show(ui, |ui| {
    ui.set_style(app_style);
    let _ = ui.button("Uses the app's theme");
});
```

Choose a glass tint that suits the restored text colours; restoring a light theme on dark
glass also restores its dark text. Use ordinary egui controls inside glass containers.
`GlassButton` paints another glass surface, so use it on the backdrop rather than nesting
it inside `Glass` or `GlassToolbar`.

## Materials and controls

| API / preset | Intended use |
|---|---|
| `Glass::default()` / `GlassStyle::regular()` | Small frosted cards, restrained bevel and halo |
| `Glass::panel()` / `GlassStyle::panel()` | Large sheets and sidebars with quiet edges |
| `GlassStyle::clear()` | Stronger lensing over rich imagery, little frost |
| `GlassStyle::dark()` / `panel_dark()` | Dark counterparts; pair with a dark backdrop |
| `GlassButton::new("Continue").show(ui)` | Standalone capsule action |
| `GlassToolbar::default().show(ui, closure)` | Toolbar with ordinary egui controls inside |
| `GlassSlider::new(&mut value, 0.0..=100.0).show(ui)` | Standalone slider with glass thumb and editable value |
| `paint_glass(ui, rect, &style)` | Custom surface; paint it before its foreground content |

Tune the material without changing your widget layout:

```rust
let style = egui_glass::GlassStyle {
    blur: 20.0,
    refraction: 4.0,
    chromatic: 0.0,
    corner_radius: 12.0,
    tint: egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180),
    ..egui_glass::GlassStyle::panel()
};
egui_glass::Glass::new(style).show(ui, |ui| { ui.label("A quiet sheet"); });
```

Higher tint alpha mixes in more tint colour; a high white tint improves text contrast but
makes the backdrop less apparent. Blur and refraction are measured in logical points.
For dark UIs use a dark preset and a matching host theme/backdrop. For reduced transparency,
choose an ordinary opaque `egui::Frame` in your app instead of drawing glass.

## Troubleshooting

On the development version, `GlassButton` and `GlassSlider` animate their hover/press
material changes using the local egui `Style::animation_time`, capped at 120 ms.
Use `.animate(false)` on either widget, or set the host animation time to `0.0`, for instant
transitions. Clicks, focus, slider values and thumb positions still update immediately.

| Symptom | Check |
|---|---|
| No `wgpu_render_state` / no glass | Enable eframe's `wgpu` feature and select `Renderer::Wgpu`; glow is unsupported |
| Panic at `set_backdrop` or GPU sample-count error | Call `init` first and match the renderer's MSAA count |
| Glass looks flat or white | Register a non-uniform backdrop, draw it before glass, and reduce tint alpha if needed |
| Text behind glass does not appear | Static glass sees only its image; use `LiveBackdrop` for page content |
| Background vanishes around panels | Opaque panel frames cover it; use `Frame::NONE` for those outer frames |
| Buttons or selection colours changed | Glass applies flat child visuals; use the style-preservation recipe above |
| Dialog padding doubled | Remove the modal's outer frame padding; give padding to `Glass` once |
| Floating area moves on its first frame | Use `Area::constrain(false)`; your app then owns viewport placement/clamping |
| Page does not scroll beneath a floating overlay | egui sends wheel input to the top layer; keep scrolling content in that layer or explicitly route wheel input in your app |
| Images missing through live glass | Register their native textures with the library before drawing the live page |
| Headless tests have no GPU renderer | Exercise content using an opaque frame; verify glass rendering separately with wgpu |

## Before adopting

This is a **0.1 library**: suitable for experimenting with glass in native egui apps, with a
small API that may evolve before 1.0. Evaluate rendering and interaction on your target devices.

- Static glass samples the registered image; it does not automatically capture widgets behind it.
  `LiveBackdrop` renders page content again off screen, adding layout and GPU work each frame.
- Library tests run in CI on Linux, macOS, and Windows. Demo builds are checked on macOS and
  Windows. These are build/test checks, not a GPU rendering certification; browser and mobile
  targets are not covered by CI.
- `GlassButton` supports Tab navigation, Enter/Space activation, a visible focus outline, and
  screen-reader button names. Enable the host's AccessKit integration (as above), and use
  `.accessible_name("Back")` for icon-only buttons. An opaque/reduced-transparency fallback
  remains an application responsibility; check contrast against your actual backdrop.
- There are no default crate features. Enable `features = ["serde"]` to serialize `GlassStyle`.

## Docs

- [Guide](https://github.com/heonny/egui-glass/blob/main/docs/guide.md) — put glass into an existing app: backdrop, components, presets, dark
  theme, glass over live content.
- [Reference](https://github.com/heonny/egui-glass/blob/main/docs/reference.md) — every function and `GlassStyle` field, how the shader works,
  limits, the demo app, building the macOS app, publishing.
- [docs.rs](https://docs.rs/egui_glass) — API docs.
- [Contributing](https://github.com/heonny/egui-glass/blob/main/CONTRIBUTING.md) — development, bug reports, and release checks.
- [Changelog](https://github.com/heonny/egui-glass/blob/main/CHANGELOG.md) — changes for the next release.
- [Verification](https://github.com/heonny/egui-glass/blob/main/docs/verification.md) — automated GPU checks and remaining native testing.
- [Performance](https://github.com/heonny/egui-glass/blob/main/docs/performance.md) — quality options, benchmark commands and measurement limits.

## Demo

```bash
cargo run -p egui_glass_demo          # sliders for every parameter, presets, export/import
cargo run -p egui_glass_demo --example minimal
make install                           # macOS: Egui Glass.app into /Applications
```

The development demo includes **Copy Rust** to copy the current material as a complete
`egui_glass::GlassStyle` expression. **Modified** marks changes relative to the selected
preset, and **Reset** restores that preset. After Import, the imported material becomes
the reset baseline. Reset affects the material only; the Live backdrop toggle is unchanged.

## License

MIT. "Liquid Glass" is Apple's name for its design language; this is an independent
reimplementation of the look and is not affiliated with Apple.
See [asset provenance](https://github.com/heonny/egui-glass/blob/main/docs/assets.md) for the demo
images and branding; these assets are not included in the library crate.

### Managed integration (development version)

The unreleased `GlassContext` API combines initialization and resource management:

```rust
let glass = egui_glass::GlassContext::new(ctx, rs, 1)?;
glass.set_fonts(font_definitions);
glass.set_backdrop(&wallpaper)?;
// Store `glass` in application state.
```

Use this in place of `init` and `LiveBackdrop::new`. Access live rendering through
`glass.live_backdrop()`. Duplicate initialization and invalid images return `GlassError`;
an invalid upload preserves the previous backdrop. Native textures registered through
`glass.register_native_texture(&view)` return cloneable owning handles: retain them through
rendering, then let the final handle release the registration in both renderers.
See the [managed API reference](https://github.com/heonny/egui-glass/blob/main/docs/reference.md#managed-setup-development-version)
for lifetime and MSAA requirements. The published 0.1.5 integration examples above remain valid.
