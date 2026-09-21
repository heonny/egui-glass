# Reference

Complete description of `egui_glass`: API, style parameters and their defaults, how the renderer
works, its limits, the demo, packaging and publishing. For the short version see the
[guide](guide.md).

## Contents

1. [Setup](#setup)
2. [Backdrops](#backdrops)
3. [Components](#components)
4. [GlassStyle](#glassstyle)
5. [Live backdrop](#live-backdrop)
6. [How the shader works](#how-the-shader-works)
7. [Limits and gotchas](#limits-and-gotchas)
8. [Demo app](#demo-app)
9. [macOS app bundle](#macos-app-bundle)
10. [Development and releases](#development-and-releases)
11. [Managed setup (development version)](#managed-setup-development-version)

## Setup

| Function | When | What it does |
|---|---|---|
| `init(render_state: &RenderState, msaa_samples: u32)` | once | Compiles the glass pipeline and installs shared GPU resources into egui-wgpu's callback resources. `msaa_samples` must match `NativeOptions::multisampling` (pass 1 when that is 0). |

Requirements: Rust 1.92+, `eframe`/`egui-wgpu` 0.35, wgpu 29. Backends: Metal, Vulkan, DX12
(selected by the host's wgpu configuration). These are backend options, not a claim of runtime
verification on every device. The glow backend is not supported because the effect is a wgpu paint
callback.

Dependencies of the library: `egui`, `egui-wgpu`, `wgpu`, `bytemuck`; `serde` behind the `serde`
feature.

## Backdrops

Glass refracts a texture, not the framebuffer. There is one static backdrop per egui context plus
an optional per-frame live backdrop.

| Function | Notes |
|---|---|
| `set_backdrop(ctx, render_state, &ColorImage) -> TextureId` | Uploads the image as an sRGB texture and builds its mip pyramid on the GPU. Replaces the previous backdrop. Returns an egui texture id you can also draw with `ui.image`. Requires `init` first. |
| `show_backdrop(ui, rect)` | Draws the image aspect-filled and centred in `rect` and records that mapping. Outside the image glass shows `ui.visuals().panel_fill`. |
| `show_backdrop_mapped(ui, image_rect, clip, outside)` | You place the image (it may be larger than `clip`). Glass that samples beyond `image_rect` fades to `outside` over the blur radius. A transparent `outside` means "clamp to the edge" (used by the live backdrop). |
| `backdrop_rect(ctx) -> Option<Rect>` | The rect recorded by the last `show_backdrop*`. |
| `backdrop_size(ctx) -> Option<Vec2>` | Pixel size of the registered image. |
| `register_native_texture(render_state, &TextureView) -> TextureId` | Registers a wgpu texture for `ui.image` in the app renderer **and** in the live renderer, so it also shows through live glass. |
| `free_native_texture_id(render_state, &TextureId)` | Frees one registered above. |

The mapping is recorded in egui memory each frame; glass drawn in the same frame samples with it.

## Components

### `Glass`

Container: allocates the content, paints glass behind it.

```rust
Glass::default()          // GlassStyle::regular()
Glass::panel()            // GlassStyle::panel()
Glass::panel_dark()
Glass::new(style)
    .inner_margin(Margin)  // default Margin::same(16)
    .preserve_theme(true) // unreleased; default false
    .show(ui, |ui| …) -> InnerResponse<R>
```

Controls inside get flat visuals (transparent idle background, soft translucent highlight, capsule
corners, text colour matched to the glass) so glass is never stacked on glass.
Set `.preserve_theme(true)` to inherit the parent UI's control visuals instead.
Glass material and padding are unchanged; match the tint to the inherited text colours.

### `GlassButton`

```rust
GlassButton::new(text)     // capsule, GlassStyle::regular()
    .style(style)
    .icon()                // square padding: a round button for a glyph
    .min_size(Vec2)        // default 44 x 44
    .text_color(Color32)   // default: strong text colour, white on dark glass
    .accessible_name("Back") // screen-reader name; defaults to the visible text
    .animate(false)        // unreleased; default true
    .show(ui) -> Response
```

Hover and press use `style.hovered()` / `style.pressed()` (brighter / dimmer variants).
In the development version, brightness, specular strength and tint transition using the
local `Style::animation_time`, capped at 120 ms. `.animate(false)` or a host animation
time of zero switches instantly. Geometry, input and focus are not animated.
Keyboard focus adds an outline in the theme's selection stroke colour. Tab / Shift+Tab and
Enter / Space use egui's standard focus and activation handling. The button registers its role,
name, enabled state, and actions with egui's accessibility tree; the host must enable its native
AccessKit integration (for eframe, the `accesskit` feature). Containers and toolbars preserve
their child widgets' accessibility information.

### `GlassToolbar`

A capsule bar for flat buttons (`ui.button`, `ui.selectable_label`, …):

```rust
GlassToolbar::default().spacing(6.0).show(ui, |ui| { let _ = ui.button("↩"); }) -> InnerResponse<R>
```

The unreleased `.preserve_theme(true)` option preserves the parent UI's control visuals,
as on `Glass`. Toolbar spacing and padding still apply.

### `GlassSlider` (since 0.1.4)

The development version supports `.animate(false)` to disable hover/press material
transitions, with the same timing policy as `GlassButton`. The thumb retains its white
tint and subdued highlight; the value and thumb position always update immediately.

Horizontal linear slider for `&mut f32`, with an editable value above the track:

```rust
GlassSlider::new(&mut value, 0.0..=100.0)
    .text("Volume")        // visible label and accessible name
    .suffix("%")           // unit on the editable value
    .width(240.0)          // default: available width; minimum 64 points
    .step_by(1.0)          // optional step for the slider and numeric editor
    .style(style)          // optical material; white capsule thumb is retained
    .show(ui)              // Response; changed() includes numeric edits
```

Also implements `egui::Widget` for `ui.add(...)`. Uses finite ranges, including descending and
single-value ranges, and clamps values to the range. The editable value shows up to three
decimal places. The track uses a blue accent and a theme-aware neutral remainder, with a
white 20 × 16 point capsule thumb and a soft shadow, based on the supplied macOS Settings reference.
The interaction area remains 40 points tall for comfortable pointer input.
The thumb retains its white tint and subdued rim when the optical material changes. A focused thumb has an
outline; disabled sliders use a flat inactive thumb. Each enabled visible slider adds one glass
draw call. Use this outside `LiveBackdrop::run`'s content closure, like other glass widgets.

### `paint_glass(ui, rect, &style)`

Paints only the surface; build your own widgets with it (allocate a rect, paint glass, paint
content).

## GlassStyle

All lengths are logical points. `GlassStyle::default()` is `regular()`.

| Field | Meaning | regular | clear | panel | dark | panel_dark |
|---|---|---|---|---|---|---|
| `corner_radius` | corner radius; `f32::INFINITY` = capsule | 22 | 22 | 28 | 22 | 28 |
| `corner_smoothing` | 0 circular … 1 max; 0.6 looks like iOS continuous corners | 0.6 | 0.6 | 0.6 | 0.6 | 0.6 |
| `blur` | backdrop blur radius | 10 | 2 | 36 | 10 | 36 |
| `refraction` | max displacement of the backdrop at the edge | 14 | 22 | 4 | 14 | 4 |
| `edge_width` | width of the lens band measured inwards from the edge | 8 | 34 | 8 | 8 | 8 |
| `chromatic` | RGB split inside the lens band | 0.3 | 0.35 | 0 | 0.3 | 0 |
| `tint` | colour mixed over the backdrop; alpha is the amount | white 52 | white 18 | white 80 | black 90 | white 24 |
| `brightness` | backdrop multiplier | 1.06 | 1.06 | 0.985 | 0.9 | 0.72 |
| `saturation` | backdrop saturation | 1.2 | 1.2 | 1.05 | 1.2 | 0.9 |
| `specular` | bevel highlight strength (corner-weighted) | 0.5 | 0.5 | 0.08 | 0.5 | 0.1 |
| `border` | inner hairline + dark edge contour strength | 0.3 | 0.3 | 0.15 | 0.5 | 0.15 |
| `shadow` | shadow strength | 0.045 | 0.045 | 0.06 | 0.045 | 0 |
| `shadow_radius` | shadow blur radius | 18 | 18 | 10 | 18 | 10 |
| `shadow_offset` | shadow pushed down by | 3 | 3 | 2 | 3 | 2 |
| `shadow_spread` | shadow shape grown by (before blur) | 0 | 0 | 0 | 0 | 0 |

Builders: `with_corner_radius`, `capsule`, `with_tint`, `hovered`, `pressed`; `is_dark()` says
whether text should be light.

What the presets are for: `regular` — buttons, toolbars, cards (lightly frosted, thin bevel, a
faint wide halo); `clear` — small controls on rich imagery; `panel` — sidebars and sheets (a plain
frosted sheet with a whisper of lensing, no highlights, a tight contact shadow); `dark` /
`panel_dark` — dark themes (dimmed content, light text).

Corners use the corner-smoothing model popularised by Figma: a Bezier ease into a shorter circular
arc spanning `(1 + smoothing) * radius` along each edge. No Apple curve constants are used.

With the `serde` feature the struct is `Serialize + Deserialize` and `#[serde(default)]`, so
partial JSON loads.

## Live backdrop

`LiveBackdrop` makes glass refract whatever is drawn beneath it, text included, with no frame of
lag.

```rust
let live = LiveBackdrop::new(render_state, Some(font_definitions));   // after init, before set_backdrop
live.set_fonts(font_definitions);                                       // if the app changes fonts
live.run(ui, clear_color, |ui| page(ui));                               // every frame
```

The development version also provides
`run_with_quality(ui, clear_color, LiveBackdropQuality, closure)`:

| Quality | Off-screen width and height | Approximate pixel count |
|---|---|---|
| `Full` (default for `run`) | 1× | 100% |
| `Balanced` | 0.75× | 56.25% |
| `Performance` | 0.5× | 25% |

Dimensions round up to whole pixels. Quality is selected per call, with no shared mutable
quality setting between clones. Resizing or changing quality reallocates the target only
when its dimensions change. Logical projection and glass geometry stay unchanged, and
mip selection accounts for backdrop resolution so the requested blur remains in points.
This reduces rasterization and mipmap cost, not the second layout or main glass draw.
Lower settings soften small refracted details. See [performance](performance.md).

How: `run` shows `page` in `ui` as usual, then lays it out a second time in a **twin egui
context** (memory cloned from the main one, input events cleared, a `Ui` with the same id and
rect so scroll state matches), tessellates it and, inside a paint-callback `prepare`, renders it
with a second `egui_wgpu::Renderer` into an sRGB texture, builds the mip pyramid and binds it as
this frame's backdrop. Glass drawn after `run` (floating areas, later widgets) samples it; glass
inside `page` would be rendered into the backdrop too, so keep it outside.

Cost: one extra layout and draw of the page per frame plus the pyramid blits. The twin pass
clears input events, but the closure still executes again (and egui may request more passes).
Keep non-UI side effects outside it; do not rely on it running once per frame. Native textures
are known only to the app renderer; `register_native_texture`
mirrors them (with id remapping) into the twin renderer. If a frame does not call `run`, glass
falls back to the static backdrop automatically.

## How the shader works

One fragment shader per glass surface, drawn as a single triangle over the callback rect
(`glass.wgsl`):

1. **Shape**: signed distance to a box with smoothed corners, evaluated as a polyline (Bezier +
   arc from the corner model), folded into one quadrant. Anti-aliased mask from the distance.
2. **Lens**: within `edge_width` of the edge the backdrop is sampled displaced outward along the
   analytic rounded-box normal (the polyline gradient would be faceted), with a circular
   profile and an RGB split for `chromatic`.
3. **Blur**: the backdrop is an sRGB texture with a 13-tap (Jimenez) mip pyramid; the shader
   samples a fractional level with a 9-tap disc, which reads like a Gaussian without banding.
4. **Material**: saturation and brightness, then the tint mix.
5. **Bevel**: a thin specular rim (`pow(t,10)`) weighted to curved parts of the edge, a hairline
   inner border, and a faint dark contour at the very edge scaled by `border` — on a white page
   this contour is what separates glass from the page.
6. **Shadow**: outside the shape only, from the shape pushed down by `shadow_offset` and grown by
   `shadow_spread`, blurred over `shadow_radius` with a quadratic tail.

Output is premultiplied; colours are handled in gamma space like egui, with a linear conversion
when the surface format is sRGB. Each surface costs one draw call and one 256-byte uniform slot
(slots are reused per frame and copied when the buffer grows).

## Limits and gotchas

- Static backdrop mode does not refract egui-drawn content; use `LiveBackdrop` for that.
- `egui::Area` on its first (sizing) frame assumes a 600x400 size and, with the default
  `constrain`, clamps and stores a wrong position; use `.constrain(false)` for small floating
  glass.
- `ctx.set_visuals` only touches the current (system) theme; force a theme with `set_theme` and
  `set_visuals_of` (the demo does).
- egui routes the mouse wheel only to the scroll area whose layer is topmost under the pointer;
  over floating glass forward it yourself (demo `scene()`).
- Very large blur values on tiny surfaces read as a flat colour; keep `blur` under the surface's
  half size for small controls.
- MSAA: pass the same sample count to `init` as `NativeOptions::multisampling`, otherwise pipeline
  validation fails (0 and 1 both mean one sample).
- Create one `LiveBackdrop` per renderer, before registering textures. Creating another replaces
  the off-screen renderer and its texture registrations. Keep the twin's fonts synchronized.
- `set_backdrop` invalidates the previous backdrop's texture id. Stop drawing with that id after
  replacing the image; do not manually free the currently active backdrop.
- Browser/mobile builds and multi-viewport rendering are not covered by CI. Test your intended
  host configuration before adopting the library.
- There is no automatic opaque/reduced-transparency mode; provide an app-level fallback if
  needed. Check text and focus-indicator contrast on your actual backdrop. Native accessibility
  tree checks do not replace end-to-end testing with the screen readers your users rely on.

## Demo app

The development version adds **Copy Rust**, **Reset**, and a **Modified** indicator.
Copy Rust produces a complete `egui_glass::GlassStyle` expression using exact premultiplied
tint bytes and round-trip float literals. Reset restores the last selected preset or
successfully imported material, leaving Live backdrop unchanged. Selecting a preset or
importing a material establishes a new baseline; Export and Copy Rust do not change it.

`cargo run -p egui_glass_demo` (`examples/demo`). Left glass panel: grouped glass sliders for every
numeric style field, presets, a *Live backdrop* toggle, and pinned Export / Import actions for
`{ style, live_mode, live_quality }` as JSON through native
file dialogs. Sidebar items switch between the bundled photos (`examples/asset`); drop an image
onto the window to load your own; drag the floating glass around and try the Volume slider card.
A dark tint switches the whole
page to a macOS-style dark theme. Platform UI fonts are loaded when present (SF Pro / Apple SD
Gothic Neo on macOS, Segoe UI / Malgun Gothic on Windows).

Environment knobs, mainly for screenshots: `LG_PHOTO=<index>`, `LG_SCROLL=<px>`,
`LG_PRESET=dark|clear`, `LG_LIVE=0`, `LG_POS=x,y` (window position, e.g. to land on a 2x display).

`cargo run -p egui_glass_demo --example minimal` runs the README example.

## macOS app bundle

| Command | What it does |
|---|---|
| `make` / `make bundle` | release build, `Egui Glass.app` in `target/release/bundle` |
| `make install` | same, then replaces `/Applications/Egui Glass.app` |
| `make run` / `make test` / `make lint` | cargo run / test / clippy |

`scripts/bundle-macos.sh` uses the icon from `assets/branding` (`EguiGlass.icns`, or generates one
from a 1024 px PNG passed as an argument), copies the sample photos into
`Contents/Resources/asset`, writes `Info.plist`, ad-hoc signs the bundle and, with `--install`,
copies it to `/Applications` and strips any quarantine flag. Needs `sips`, `iconutil`,
`codesign` (all in macOS).

On the first launch from Finder macOS shows *"Apple could not verify … is free of malware"*
because the app is not notarized: *System Settings > Privacy & Security > Open Anyway*, once. To
ship without the prompt, sign with a Developer ID and notarize:

```bash
CODESIGN_IDENTITY="Developer ID Application: <name> (<team id>)" \
NOTARY_PROFILE="<keychain profile from: xcrun notarytool store-credentials>" \
make install
```

Uninstall by deleting the app; it stores nothing else.

## Development and releases

```bash
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features    # kept warning-free
```

CI (`.github/workflows/ci.yml`) runs default/serde library tests and clippy on Linux, macOS and
Windows, checks Rust 1.92, builds API docs and the package, and builds the demo on macOS and
Windows. These automated checks do not exercise GPU rendering.

See [CONTRIBUTING.md](../CONTRIBUTING.md) for the release checklist. `release.yml` checks the tag
against the crate version, runs library tests, checks docs, and performs a publish dry run before
publishing to crates.io with the `CARGO_REGISTRY_TOKEN` secret.

See `CLAUDE.md` for the code layout and the design rules the look follows.

## Managed setup (development version)

`GlassContext::new(ctx, render_state, msaa_samples) -> Result<GlassContext, GlassError>`
replaces the manual `init` / `LiveBackdrop::new` sequence. Store the result in app state.
The sample count must match the host renderer (0 means 1). The existing functions remain
available; choose one setup path for each renderer/context pair.

- `set_fonts(fonts)` configures both visible and offscreen contexts, including later updates.
- `set_backdrop(&image)` validates dimensions, pixel count and the device texture limit.
  Invalid images leave the existing backdrop intact. Successful replacement frees its old ID.
- `live_backdrop()` exposes `run` and `run_with_quality` with their existing closure contract.
- `register_native_texture(&view)` returns a `GlassTexture`. Use `handle.id()` in widgets.
  Clones share ownership; the final drop frees the ID in both renderers. Keep the handle
  until the frame finishes rendering, and drop it outside renderer lock guards. Do not
  manually free this ID through the low-level API.

`GlassError` distinguishes duplicate setup, unsupported sample counts, malformed images
and oversized images. Allocation/device errors still use wgpu's error handling. Custom
fonts installed elsewhere are not automatically captured: use the manager's setter.
The twin renderer is initialized eagerly, but does no offscreen rendering until `run`.
Shared pipelines and the active backdrop live with the renderer, not the manager handle.
