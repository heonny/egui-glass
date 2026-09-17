# Changelog

## Unreleased

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
