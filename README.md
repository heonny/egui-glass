<p align="center">
  <img src="assets/branding/readme-logo.png" alt="egui_glass" width="640">
</p>

Apple *Liquid Glass* style surfaces for [egui](https://github.com/emilk/egui), rendered on the GPU
through `egui-wgpu` paint callbacks. One fragment shader does everything: continuous-corner
signed-distance shape, edge refraction (lensing) with optional chromatic aberration, mip-based
backdrop blur, tint / vibrancy, specular rim, inner border and drop shadow. No animation.

![demo](docs/demo.png)

## Requirements

| | |
|---|---|
| Rust | 1.92 or newer (MSRV of `eframe` / `wgpu`) |
| Backend | `eframe` with the **wgpu** renderer (`egui-wgpu`); the glow backend is not supported |
| GPU | anything wgpu drives: Metal, Vulkan, DX12 (the demo was developed on macOS) |

Runtime dependencies of the library: `egui`, `egui-wgpu`, `wgpu`, `bytemuck`, and `serde`
(optional, feature `serde`). The demo additionally uses `eframe`, `image`, `serde_json`, `rfd`
and `log`.

## Run the demo

```bash
git clone https://github.com/heonny/egui-glass.git
cd egui-glass
cargo run -p egui_glass_demo
```

The first build compiles wgpu and takes a few minutes. Left panel: sliders for every style
parameter, the presets (Regular, Clear, Dark, Panel), a *Live backdrop* toggle, and Export /
Import buttons that save the settings as JSON. A dark tint switches the page to a macOS-style dark
theme. Sidebar items switch between the bundled photos (`examples/asset`); drop an image file onto
the window to load your own. Drag the glass panels around.

Environment knobs for screenshots: `LG_PHOTO=<index>`, `LG_SCROLL=<px>`, `LG_PRESET=dark|clear`,
`LG_LIVE=0` (start with the live backdrop off).

### macOS app

```bash
make            # release build + Egui Glass.app in target/release/bundle
make install    # same, then copy the app into /Applications
```

The script (`scripts/bundle-macos.sh`) uses the app icon from `assets/branding`, copies the sample
photos into the bundle and ad-hoc signs it for local use. On first launch from Finder, macOS shows
"Apple could not verify ... is free of malware": open *System Settings > Privacy & Security* and
click *Open Anyway* once. For a build that launches without the prompt, sign with a Developer ID
and notarize by setting `CODESIGN_IDENTITY` (and optionally `NOTARY_PROFILE`) before `make install`. `make run`, `make test`
and `make lint` wrap the cargo commands.

## Use the library

```toml
[dependencies]
egui_glass = { git = "https://github.com/heonny/egui-glass.git" }   # not on crates.io yet
```

```rust
use egui_glass::{Glass, GlassButton, GlassStyle, GlassToolbar};

// once, e.g. in App::new (eframe with the wgpu backend)
let rs = cc.wgpu_render_state.as_ref().unwrap();
egui_glass::init(rs, 1 /* msaa samples */);
egui_glass::set_backdrop(&cc.egui_ctx, rs, &wallpaper_color_image);

// every frame
egui_glass::show_backdrop(ui, ui.max_rect());          // draws the wallpaper (aspect-fill), records its mapping
// or place the image yourself; glass outside it shows `page_color`:
// egui_glass::show_backdrop_mapped(ui, image_rect, clip_rect, page_color)

let style = GlassStyle::regular();                      // ::clear(), ::dark(), ::panel(), ::panel_dark()
Glass::new(style).show(ui, |ui| { ui.label("card / section / sidebar"); });
GlassButton::new("Continue").style(style).show(ui);
GlassToolbar::new(style).show(ui, |ui| { ui.button("↩"); ui.button("🗑"); });
egui_glass::paint_glass(ui, rect, &style);              // building block for your own widgets
```

`GlassStyle` fields (all in logical points): `corner_radius` (`f32::INFINITY` = capsule),
`corner_smoothing` (0 = circular arc, 0.6 = the continuous look of iOS corners, 1 = max), `blur`,
`refraction`, `edge_width`, `chromatic`, `tint`, `brightness`, `saturation`, `specular`, `border`,
`shadow`, `shadow_radius`. Use `panel()` for large surfaces (sidebars, sheets): heavy blur, almost
no lensing. Text on glass picks a light colour automatically when the tint is dark.

Corners use the corner-smoothing model popularised by Figma (a Bezier ease into a shorter circular
arc, spanning `(1 + smoothing) * radius` along each edge), evaluated as a signed-distance field in
the shader. No Apple curve constants are used.

## How it works / limits

egui paints in a single render pass, so a shader cannot read the pixels already drawn behind a
widget in the same frame. By default glass therefore refracts a **backdrop image** you register
with `set_backdrop` (a wallpaper, a photo) and place with `show_backdrop`; anything egui draws on
top of it shows as the page colour through the glass. This is cheap and matches Apple's guidance
that glass floats above content and is never stacked on glass.

Each glass surface costs one draw call (a single triangle) and one 256-byte uniform slot; the
backdrop is uploaded once with a linear-light mip chain, and blur is a 5-tap sample at a mip level.

### Live backdrop (optional)

`LiveBackdrop` lifts that limit: the content you pass to `LiveBackdrop::run` is laid out a second
time in a twin egui context (same memory, no input events) and rendered off screen without the
glass, then used as this frame's backdrop. Glass then refracts everything beneath it, text
included, with no frame of lag. The cost is one extra layout and draw of that content per frame.

```rust
// once, after init and before set_backdrop / register_native_texture
let live = LiveBackdrop::new(rs, Some(font_definitions));
// every frame: content inside, glass outside
live.run(ui, page_color, |ui| page(ui));
Glass::new(style).show(ui, |ui| { /* floats over the page */ });
```

Images drawn inside the content must be registered with `egui_glass::register_native_texture`
(or `set_backdrop`) so the off-screen pass can draw them too.

## Development

```bash
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features
```

See `CLAUDE.md` for the project layout and the design rules the code follows.

## Trademark note

"Liquid Glass" is Apple's name for its design language; `egui_glass` is an independent
reimplementation of the *look*, not affiliated with or endorsed by Apple. The term is only used
descriptively in this documentation.

## License

MIT. See [LICENSE](LICENSE).
