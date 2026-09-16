# egui_glass

Apple Liquid Glass style surfaces for egui, drawn on the GPU through `egui-wgpu` paint callbacks.
Cargo workspace: the library in `crates/egui_glass`, the demo app in `examples/demo`, sample
photos in `examples/asset`.

## Commands

```bash
cargo run -p egui_glass_demo                              # demo (eframe + wgpu)
cargo test --workspace --all-features                     # unit tests (corner math, mips, serde)
cargo clippy --workspace --all-targets --all-features     # must stay warning-free
make bundle | make install                                 # macOS .app via scripts/bundle-macos.sh
```

Screenshot knobs for the demo: `LG_PHOTO=<index>` (initial photo), `LG_SCROLL=<px>` (initial
scroll offset), `LG_PRESET=dark|clear`, `LG_LIVE=0` (start with the live backdrop off). Launch,
wait for the first frame, then capture the window; a capture taken before the first present is
blank, so retry rather than assume a bug.

## Layout

- `crates/egui_glass/src/glass.wgsl` — the one fragment shader: smoothed-corner SDF (polyline of
  Bezier + arc, folded into one quadrant), edge lens refraction with chromatic split, 5-tap blur at
  a mip level, tint/vibrancy, specular rim, border, shadow. Premultiplied output.
- `renderer.rs` — pipeline, dynamic-offset uniform slots (256 B each, reused per pass, copied on
  growth), `corner_params()` (CPU side of the corner model), static vs live backdrop binding.
- `backdrop.rs` — static backdrop upload with a linear-light CPU mip chain, screen mapping
  recorded in egui memory, `register_native_texture` mirrored into the live renderer.
- `live.rs` + `mipgen.wgsl` — `LiveBackdrop`: twin egui context rendered off screen inside a
  paint-callback `prepare`, GPU mip blits, then bound as this frame's backdrop.
- `style.rs` — `GlassStyle` and presets; `widgets.rs` — `Glass`, `GlassButton`, `GlassToolbar`,
  `paint_glass`, flat visuals for controls sitting on glass.
- `examples/demo/src/main.rs` — the demo page, layouts, theme, settings export/import;
  `fonts.rs` — platform UI fonts with egui's as fallback.

## Design rules

- Glass refracts only the registered backdrop (or the live off-screen render); what it shows must
  match what is on screen next to it. Never bake content into the backdrop that is not visible.
- Large surfaces (sidebars, sheets) use `GlassStyle::panel()` / `panel_dark()`: heavy blur, almost
  no lensing. Strong refraction and specular are for small controls only. Shadows stay subtle.
- Never stack glass on glass: controls inside a `Glass` are flat (transparent idle, soft highlight).
- Corners are the Figma-style corner-smoothing model. Do not add Apple's reverse-engineered curve
  constants. "Liquid Glass" is Apple's trademark: use it only descriptively, never in identifiers.
- Colours: `Color32` is premultiplied — unpremultiply before sending straight colours to the
  shader. Textures are gamma bytes; average mips in linear light.
- Keep the library dependency-light (`egui`, `egui-wgpu`, `wgpu`, `bytemuck`, optional `serde`).
  App-level concerns (fonts, file dialogs, layouts) belong in the demo.

## Gotchas already hit

- egui `Area`: on its first (sizing) frame egui assumes a 600x400 size and, with the default
  `constrain`, clamps and stores a wrong position. Use `.constrain(false)` for small floating glass.
- `ctx.set_visuals` only touches the current (system) theme. Force a theme with `set_theme` and
  `set_visuals_of`.
- eframe persists egui memory (area positions) by default; the demo opts out.
- egui routes the mouse wheel only to the scroll area whose layer is topmost under the pointer;
  over floating glass the demo forwards it by hand.
- Polyline SDF: skip segments with `|b - a|^2 < 1e-4`. Degenerate segments have a noise-sign cross
  product and produced dotted artifacts along the fold axes.
- Native textures registered in the app's renderer are unknown to the live (twin) renderer; use
  `register_native_texture` / `set_backdrop` so they are mirrored with id remapping.
- Verify visually after shader or layout changes: capture the demo and inspect the pixels; a CPU
  replica of the SDF was the fastest way to find the sign bug.

## Conventions

- Commits: `<type>: <title>` (feat, fix, refactor, docs, test, chore), imperative, no emojis, no
  generation markers. Body lists what changed and why.
- No personal data, machine paths or credentials in the repo or docs.
