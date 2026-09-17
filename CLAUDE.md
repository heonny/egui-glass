# egui_glass

Apple Liquid Glass style surfaces for egui, drawn on the GPU through `egui-wgpu` paint callbacks.
Cargo workspace: the library in `crates/egui_glass`, the demo app in `examples/demo`, sample
photos in `examples/asset`.

## Commands

```bash
cargo run -p egui_glass_demo                              # demo (eframe + wgpu)
cargo test --workspace --all-features                     # unit tests (corner math, serde)
cargo clippy --workspace --all-targets --all-features     # must stay warning-free
make bundle | make install                                 # macOS .app via scripts/bundle-macos.sh
```

Screenshot knobs for the demo: `LG_PHOTO=<index>` (initial photo), `LG_SCROLL=<px>` (initial
scroll offset), `LG_PRESET=dark|clear`, `LG_LIVE=0` (start with the live backdrop off),
`LG_POS=x,y` (window position, e.g. to land on a 2x display for crisp captures). Launch,
wait for the first frame, then capture the window; a capture taken before the first present is
blank, so retry rather than assume a bug.

## Layout

- `crates/egui_glass/src/glass.wgsl` — the one fragment shader: smoothed-corner SDF (polyline of
  Bezier + arc, folded into one quadrant), edge lens refraction with chromatic split, 5-tap blur at
  a mip level, tint/vibrancy, specular rim, border, shadow. Premultiplied output.
- `renderer.rs` — pipeline, dynamic-offset uniform slots (256 B each, reused per pass, copied on
  growth), `corner_params()` (CPU side of the corner model), static vs live backdrop binding.
- `backdrop.rs` — static backdrop upload (level 0), screen mapping recorded in egui memory,
  `register_native_texture` mirrored into the live renderer.
- `mipgen.rs` + `mipgen.wgsl` — sRGB mip pyramid built on the GPU with the 13-tap Jimenez
  downsample; sampled through a non-sRGB view so the glass shader sees gamma bytes. Used by both
  the static and the live backdrop.
- `live.rs` — `LiveBackdrop`: twin egui context rendered off screen inside a paint-callback
  `prepare`, then bound as this frame's backdrop.
- `style.rs` — `GlassStyle` and presets; `widgets.rs` — `Glass`, `GlassButton`, `GlassToolbar`,
  `paint_glass`, flat visuals for controls sitting on glass.
- `assets/branding/` — app icon (`EguiGlass.icns`, `app-icon.png`) and README logo; keep the
  crab / copper-and-cyan identity, do not regenerate these programmatically.
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
  shader. Backdrop textures are sRGB so mips filter in linear light; the shader samples a
  non-sRGB view and works in gamma space like egui.
- Blur quality is what separates "glass" from "grey box": keep the 13-tap pyramid + disc taps;
  plain box mips looked blotchy on large panels.
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
- Verify visually after shader or layout changes: capture the demo on a 2x display and inspect
  zoomed crops of a control over a busy photo; a CPU replica of the SDF was the fastest way to
  find the sign bug.
- Tuned look (compared at 2x against iOS 26 home/lock screens on the same Monet wallpaper, then
  reviewed by a designer): Apple glass is a *flat plate with a thin bevel*, not a puffy droplet.
  Small controls: white tint 52/255, blur 10, brightness 1.06, refraction 14 in an 8 px edge
  band, chroma 0.3, specular 0.5, border 0.3; shadow is a faint wide halo, not a drop shadow
  (0.045, radius 18, offset 3 — iOS Photos toolbar on white), panels 0.04 / 24 / 4, dark 0; rim = `pow(t,10)`, corner-weighted (`4 n.x² n.y²`), no top-down sheen. On a white page
  glass is separated only by a 2-3 px faint dark contour (`0.2 * pow(t,8)`, stronger on the
  unlit side) and, for panels, a brightness of 0.985 (a hair darker than the page). Panels: blur 36, tint 80/255, refraction 8 in 10 px, specular 0.3, border 0.25, radius 28. The white line must never be what separates glass from the background; the
  tint/blur contrast and the contour do that. Lens/lighting
  normal is the analytic rounded-box normal (the SDF gradient is faceted and drew spokes).
  Change these only with side-by-side captures.

## Release

The library is published to crates.io from `crates/egui_glass` (no path dependencies; the
README is pulled in from the workspace root; image links in it must be absolute raw GitHub URLs,
relative paths 404 on crates.io because the crate lives in a subdirectory). `cargo publish -p egui_glass --dry-run` must pass.
Bump `version` in `crates/egui_glass/Cargo.toml`, commit, then `git tag vX.Y.Z && git push origin
vX.Y.Z`: `.github/workflows/release.yml` checks the tag against the version and publishes with the
`CARGO_REGISTRY_TOKEN` secret. CI (`ci.yml`) runs library tests/clippy on Linux, macOS and Windows
and builds the demo on macOS and Windows.

## Docs

Three tiers, keep them in this shape: root `README.md` = logo, one paragraph, quick start,
links; `docs/guide.md` = the short path to using it in an app; `docs/reference.md` = everything
(API tables, every `GlassStyle` field with preset defaults, shader, limits, demo, packaging,
releases). When a default or an API changes, update the reference table and, if it touches the
short path, the guide.

## Conventions

- Commits: `<type>: <title>` (feat, fix, refactor, docs, test, chore), imperative, no emojis, no
  generation markers. Body lists what changed and why.
- No personal data, machine paths or credentials in the repo or docs.
