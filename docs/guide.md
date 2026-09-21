# Guide: putting glass into your app

The [README](https://github.com/heonny/egui-glass#quick-start), also displayed on
[crates.io](https://crates.io/crates/egui_glass), is the self-contained starting point: it includes
a complete app, sidebar and scrolling-modal recipes, host-theme preservation, and troubleshooting.
This guide adds topic-specific details. See the [reference](reference.md) for the full API.

## 1. Requirements

- `eframe` 0.35 with the **wgpu** feature (`renderer: eframe::Renderer::Wgpu`). The glow backend
  is not supported.
- Rust 1.92 or newer.

```toml
egui_glass = "0.1.5"                                      # add features = ["serde"] to save styles
```

## 2. Register once

In `App::new` (or wherever you have the `CreationContext`):

```rust
let rs = cc.wgpu_render_state.as_ref().unwrap();
egui_glass::init(rs, 1);                 // 1 = msaa samples; match NativeOptions::multisampling (0 counts as 1)
```

## 3. Give the glass something to refract

egui paints in one pass, so glass cannot read what is already on screen. It refracts a
**backdrop** you provide. Two ways:

**A. A backdrop image** (a wallpaper, a photo, a gradient). Upload once, draw every frame:

```rust
egui_glass::set_backdrop(&cc.egui_ctx, rs, &color_image);      // once, or whenever it changes

// every frame, before the glass:
egui_glass::show_backdrop(ui, ui.max_rect());                  // aspect-fill into a rect
// or place it yourself; `page_color` is what glass shows outside the image:
egui_glass::show_backdrop_mapped(ui, image_rect, clip_rect, page_color);
```

Anything egui draws on top of the backdrop (text, widgets) is absent from the refracted texture:
glass samples the registered image at that location. Outside the image it samples the page
colour. This is the cheaper path.

**B. Live backdrop**: glass refracts everything underneath, text included. Your page content is
laid out and drawn a second time off screen (see the reference for cost and caveats):

```rust
// once, after init and before set_backdrop / register_native_texture
let live = egui_glass::LiveBackdrop::new(rs, Some(font_definitions));

// every frame: page inside the closure, glass outside it
live.run(ui, page_color, |ui| my_page(ui));
Glass::panel().show(ui, |ui| { /* floats over the page */ });
```

Images drawn inside the page must be registered with `egui_glass::register_native_texture`
(`set_backdrop` does this for you) so the off-screen pass can draw them.
The closure runs for both visible and off-screen layout; keep file writes, network requests,
and simulation updates outside it. Create one live backdrop per renderer before registering
textures, and keep its fonts synchronized with the main context.

## 4. Draw glass

```rust
use egui_glass::{Glass, GlassButton, GlassStyle, GlassToolbar};

Glass::default().show(ui, |ui| { ui.label("card / section"); });      // small surface
Glass::panel().show(ui, |ui| { ui.label("sidebar / sheet"); });       // large frosted sheet
GlassButton::new("Continue").show(ui);                                 // capsule button
GlassButton::new("‹").icon().show(ui);                                 // round icon button
GlassToolbar::default().show(ui, |ui| { let _ = ui.button("Undo"); }); // capsule bar of flat buttons
egui_glass::paint_glass(ui, rect, &GlassStyle::regular());             // just the surface
```

Glass usually floats: put it in an `egui::Area` (with `.constrain(false)`, see the reference for
why). Paint a custom glass surface before its own foreground content, but after the backdrop.
Use ordinary egui controls inside `Glass`; they receive flat visuals automatically.
`GlassButton` and other glass surfaces still paint their own glass, so do not nest them inside
a glass container. To retain your existing control theme, use the README's
[style-preservation recipe](https://github.com/heonny/egui-glass#preserve-your-existing-control-theme).

### Glass sliders (since 0.1.4)

`GlassSlider` provides a glass thumb, a filled track, and an editable number. It uses egui's
standard drag, arrow-key, and accessibility handling. Like the other glass widgets, it needs
`init` and a registered backdrop. This control is available starting with version 0.1.4.

```rust
egui_glass::GlassSlider::new(&mut volume, 0.0..=100.0)
    .text("Volume")
    .suffix("%")
    .step_by(1.0)
    .width(240.0)
    .show(ui);
```

Click the number to type a precise value, or drag it for small adjustments. The slider fills
the available width by default. `.style(...)` adjusts the optical material while preserving
the white capsule thumb; the remaining track follows the host's light/dark theme.

## 5. Pick a preset, tweak if needed

| Preset | Use for | Look |
|---|---|---|
| `GlassStyle::regular()` (default) | buttons, toolbars, cards | lightly frosted, thin bevel, faint halo |
| `GlassStyle::clear()` | small controls over rich imagery | almost no frost, strong lensing |
| `GlassStyle::panel()` | sidebars, sheets, large containers | plain frosted sheet, no highlights |
| `GlassStyle::dark()` / `panel_dark()` | dark themes | same, dimmed |

Tweak with the builders or fields:

```rust
let style = GlassStyle::regular().with_corner_radius(12.0).with_tint(Color32::from_rgba_unmultiplied(255, 255, 255, 90));
let style = GlassStyle { blur: 20.0, ..GlassStyle::panel() };
Glass::new(style).inner_margin(egui::Margin::same(20)).show(ui, |ui| { /* … */ });
```

Text on glass switches to white automatically when the tint is dark (`style.is_dark()`).

### Keyboard and screen readers

`GlassButton` participates in egui's Tab / Shift+Tab focus navigation and activates with Enter
or Space. Focused buttons draw an outline using the theme's selection stroke colour.
Disabled UIs suppress activation and expose the disabled state to assistive technology.

For native screen readers, enable the `accesskit` feature in your eframe dependency. Button
names default to the visible text; provide a descriptive name for icon-only actions:

```rust
GlassButton::new("‹").icon().accessible_name("Back").show(ui);
```

Keep meaningful visible text for text buttons. Test contrast over your actual imagery, and
provide an opaque alternative when your application needs reduced transparency.

## 6. Dark theme

Use `panel_dark()` / `dark()` and give the glass a dark page colour (`show_backdrop_mapped`'s
`page_color`, or the live backdrop's clear colour). The demo flips the whole egui theme when the
tint is dark; see `examples/demo/src/main.rs` (`apply_visuals`).

## 7. Save and load a look

With the `serde` feature `GlassStyle` is `Serialize + Deserialize` (missing fields take the
defaults), so a style can live in your settings file. The demo's Export / Import buttons do this
with JSON.

## Live quality in the development version

In the development version, use `live.run_with_quality(ui, clear,
egui_glass::LiveBackdropQuality::Balanced, closure)` to render the backdrop at 0.75×
resolution, or `Performance` for 0.5×. The existing `run` keeps full resolution.
The visible page is unaffected, but refracted small text becomes softer. Extra layout
work is unchanged. The demo's Quality selector lets you compare all three settings;
old imported JSON settings default to full quality. See [performance](performance.md).

## Checklist when something looks off

- Glass is plain white / page-coloured: nothing registered underneath — call `set_backdrop` and
  draw it with `show_backdrop*`, or use `LiveBackdrop`.
- Photo missing through live glass: register its texture with `register_native_texture`.
- A floating panel lands in the wrong place on the first frame: `Area::constrain(false)`.
- Wheel does not scroll under a floating panel: egui routes the wheel to the topmost layer; see
  the demo's forwarding in `scene()`.

## Simpler setup in the development version

The demo now stores a `GlassContext` instead of separately calling `init`, constructing
`LiveBackdrop`, and passing the renderer to every image upload. Create it with
`GlassContext::new(ctx, rs, samples)?`, call `glass.set_fonts(fonts)` once for both contexts,
and upload images with `glass.set_backdrop(&image)?`. Use `glass.live_backdrop()` for live
rendering. Existing standalone setup remains supported; do not mix setup paths on the
same renderer. See [managed setup](reference.md#managed-setup-development-version) for errors
and owned native-texture handles.
