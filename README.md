# egui_glass

Apple *Liquid Glass* style surfaces for [egui](https://github.com/emilk/egui), rendered on the GPU
through `egui-wgpu` paint callbacks. One fragment shader does everything: rounded-rect SDF,
edge refraction (lensing) with optional chromatic aberration, mip-based backdrop blur,
tint / vibrancy, specular rim, inner border and drop shadow. No animation, no dependencies
beyond `egui`, `egui-wgpu`, `wgpu`, `bytemuck`.

![demo](docs/demo.png)

## Usage

```rust
// once, e.g. in App::new (eframe with the wgpu backend)
let rs = cc.wgpu_render_state.as_ref().unwrap();
egui_glass::init(rs, 1 /* msaa samples */);
egui_glass::set_backdrop(&cc.egui_ctx, rs, &wallpaper_color_image);

// every frame
egui_glass::show_backdrop(ui, ui.max_rect());   // draws the wallpaper (aspect-fill), records its mapping
// or place the image yourself; glass outside it shows `page_color`:
// show_backdrop_mapped(ui, image_rect, clip_rect, page_color)

let style = GlassStyle::regular();                      // or ::clear(), ::dark(), or tweak fields
Glass::new(style).show(ui, |ui| { ui.label("card / section / sidebar"); });
GlassButton::new("Continue").style(style).show(ui);
GlassToolbar::new(style).show(ui, |ui| { ui.button("↩"); ui.button("🗑"); });
paint_glass(ui, rect, &style);                          // building block for your own widgets
```

`GlassStyle` fields (all in logical points): `corner_radius` (`f32::INFINITY` = capsule),
`corner_smoothing` (0 = circular arc, 0.6 = the continuous look of iOS corners, 1 = max), `blur`,
`refraction`, `edge_width`, `chromatic`, `tint`, `brightness`, `saturation`, `specular`, `border`,
`shadow`, `shadow_radius`.

Corners use the corner-smoothing model popularised by Figma (a Bezier ease into a shorter circular
arc, spanning `(1 + smoothing) * radius` along each edge), evaluated as a signed-distance field in
the shader. No Apple curve constants are used.

## How it works / limits

egui paints in a single render pass, so a shader cannot read the pixels already drawn behind a
widget in the same frame. By default glass therefore refracts a **backdrop image** you register
with `set_backdrop` (a wallpaper, a photo) and place with `show_backdrop`; anything egui draws on
top of it shows as the page colour through the glass. This is cheap and matches Apple's guidance
that glass floats above content and is never stacked on glass.

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

Each glass surface costs one draw call (a single triangle) and one 256-byte uniform slot; the
backdrop is uploaded once with a CPU-generated mip chain, and blur is a 5-tap sample at a mip level.

## Demo

```bash
cargo run -p egui_glass_demo
```

Left panel: sliders for every style parameter and the presets (Regular, Clear, Dark, Panel). A
dark tint switches the whole page to a macOS-style dark theme. The demo loads the platform UI font
(SF Pro / Apple SD Gothic Neo on macOS, Segoe UI / Malgun Gothic on Windows) with egui's bundled
fonts as fallback. Drop an image file on the
window to change the photo; drag the glass panels around. Sidebar items switch between the photos
in `examples/asset`; portrait photos are laid out as a tall column on the right, landscape ones
across the top. The page scrolls under the floating glass, so the sidebar shows the photo flowing
beneath it; outside the photo the glass shows the page colour.

## Trademark note

"Liquid Glass" is Apple's name for its design language; `egui_glass` is an independent
reimplementation of the *look*, not affiliated with or endorsed by Apple. The term is only used
descriptively in this documentation.

## License

MIT OR Apache-2.0
