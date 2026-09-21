# Changelog

## Unreleased

These changes are available in the development checkout; the published crate remains 0.1.5.

### Integration

- Add `GlassContext` for one-call setup, shared font configuration, validated backdrop
  uploads and cloneable `GlassTexture` handles that release both texture registrations.
- Reject duplicate initialization and unsupported MSAA counts without replacing resources.
- Migrate the demo to managed setup; preserve the standalone integration API.
- Fix live backdrop font updates being lost when copying visible-context memory.

### Rendering

- Add `LiveBackdropQuality` and `run_with_quality` for full, three-quarter, or half-resolution
  live backdrops. Existing `run` stays full quality; projection and blur units are preserved.
- Add a demo quality selector and backward-compatible JSON settings for it.
- Add a headless GPU benchmark with pixel-readback checks for quality switching, resize,
  odd dimensions, HiDPI and returning to static backdrops.

### Demo

- Add Copy Rust for a complete `GlassStyle` expression, preserving float precision and tint values.
- Keep the selected preset visible while editing and show a Modified indicator.
- Add Reset to restore the selected preset or last imported material; backdrop mode is unchanged.

### Components

- Animate button and slider hover/press materials using the host animation time, capped
  at 120 ms. Add `.animate(false)` for instant transitions; a zero host animation time
  also disables motion. Input, focus and slider position remain immediate.
- Add `Glass::preserve_theme` and `GlassToolbar::preserve_theme` to keep the host UI's control
  visuals without manually restoring its style. Flat glass visuals remain the default.

## 0.1.5 - 2026-09-17

### Documentation

- Make the crates.io README a self-contained integration guide with a complete, asset-free app.
- Add sidebar, scrolling-modal, and host-theme preservation recipes for existing egui apps.
- Explain static versus live backdrops, initialization order, MSAA, material selection, and troubleshooting.
- Replace nested glass in the quick start with ordinary child controls and clarify painting order.
- No library API or rendering changes.

## 0.1.4 - 2026-09-17

### Components and demo

- Add `GlassSlider`: a glass thumb, filled track, editable value, keyboard controls, and accessibility metadata.
- Rebuild the demo settings as a frosted glass panel with presets, collapsible parameter groups,
  and persistent Import / Export actions. Use glass sliders for all numeric material controls.
- Add a floating slider playground to the demo canvas.
- Match the slider proportions to macOS Settings and refresh the demo screenshot.

### Accessibility

- Expose `GlassButton` roles, labels, disabled state, and actions to assistive technology.
- Add `GlassButton::accessible_name` for icon-only actions and a visible keyboard-focus outline.
- Enable native AccessKit support in the demo and name its icon controls.
- Add regression coverage for focus navigation, keyboard activation, and accessibility actions.

## 0.1.3 - 2026-09-17

### Documentation

- Clarify compatibility, feature flags, platform verification, and adoption limits.
- Add contributor guidance, release checks, and demo asset provenance notes.
- Compile the crate-level integration example as a doctest.
- Document public widget builders and backdrop lifecycle requirements.

### Packaging and checks

- Include the MIT license text in the published library package.
- Check default and serde-enabled builds, minimum Rust version, API docs, and packaging in CI.
- Run library tests, documentation checks, and a publish dry run before release publishing.

For earlier changes, see the [commit history](https://github.com/heonny/egui-glass/commits/main/).
