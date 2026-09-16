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
egui_glass::show_backdrop(ui, ui.max_rect());   // draws the wallpaper, records its mapping

let style = GlassStyle::regular();                      // or ::clear(), ::dark(), or tweak fields
Glass::new(style).show(ui, |ui| { ui.label("card / section / sidebar"); });
GlassButton::new("Continue").style(style).show(ui);
GlassToolbar::new(style).show(ui, |ui| { ui.button("↩"); ui.button("🗑"); });
paint_glass(ui, rect, &style);                          // building block for your own widgets
```

`GlassStyle` fields (all in logical points): `corner_radius` (`f32::INFINITY` = capsule), `blur`,
`refraction`, `edge_width`, `chromatic`, `tint`, `brightness`, `saturation`, `specular`, `border`,
`shadow`, `shadow_radius`.

## How it works / limits

egui paints in a single render pass, so a shader cannot read the pixels already drawn behind a
widget in the same frame. Glass therefore refracts a **backdrop image** you register with
`set_backdrop` (a wallpaper, a photo, a composited screenshot of your content) and place with
`show_backdrop`. Anything egui draws on top of that backdrop is not refracted. This matches
Apple's guidance that glass floats above content and is never stacked on glass.

Each glass surface costs one draw call (a single triangle) and one 256-byte uniform slot; the
backdrop is uploaded once with a CPU-generated mip chain, and blur is a 5-tap sample at a mip level.

## Demo

```bash
cargo run -p egui_glass_demo
```

Left panel: sliders for every style parameter and the three presets. Drop an image file on the
window to change the photo; drag the glass panels around. Photos in `examples/asset` are cycled
with *Next photo*.

## License

MIT OR Apache-2.0
