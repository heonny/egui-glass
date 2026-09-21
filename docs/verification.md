# GPU and accessibility verification

The 2026-09-21 live quality changes were additionally checked with a headless Metal renderer:
quality switching, returning to static mode, resizing, odd dimensions and HiDPI passed GPU
pixel readback. See [performance results and scope](performance.md). These checks do not
replace native-window interaction or visual-quality testing.

The native checks below were performed on 2026-09-17, using macOS on Apple M1 Pro,
the wgpu demo renderer and egui/eframe 0.35. They cover the 0.1.4 accessibility changes,
not the later development UI changes. They do not certify every GPU, OS or screen reader.

The 2026-09-21 development suite passed 32 tests, including documentation examples, plus
three explicitly enabled GPU tests. Clippy and API documentation passed with warnings denied.
Theme preservation, material transitions, style reset/export and legacy settings import have
automated coverage. Native-window visual checks for these changes remain outstanding.

## Automated checks

`crates/egui_glass/tests/accessibility.rs` exercises the real egui context and AccessKit output:

- Button role, visible-text name, and focus/click actions.
- Explicit accessible name for an icon-only button.
- Disabled state and suppression of keyboard/accessibility activation.
- Tab / Shift+Tab navigation past disabled buttons.
- Enter / Space activation of a focused button.
- An actual painted focus outline in egui's output shapes.

Before the fix, tests reproduced missing button roles/names and a missing focus outline.
Keyboard activation already worked through egui and remains covered as a regression check.

## Native checks performed

| Check | Observation |
|---|---|
| Regular material, light theme, live backdrop | Image, frosted panel, and glass controls rendered |
| Dark preset with live and static backdrops | Dark surfaces rendered; mode switching remained responsive |
| Clear preset and scrolling onto a plain background | Glass continued to render as the image left the viewport |
| Return to live mode and resize using window zoom | Backdrop and glass updated to the new window dimensions |
| Native macOS accessibility tree | Export, Import, Back, Edit, and toolbar actions exposed as named buttons |
| Tab / Shift+Tab | Focus moved between glass controls; Export showed a visible outline |
| Enter on Export | Native Save dialog opened |

## Glass slider and settings panel

The 0.1.4 demo uses the library's glass panel and sliders for its settings. The header
and Import / Export actions stay visible while grouped parameters scroll. The canvas includes
an independent Volume slider. Native checks covered light/dark presets, numeric editing,
dragging Volume from 65% to 34%, and scrolling to the last shadow control.

`crates/egui_glass/tests/slider.rs` covers labels and bounds, keyboard changes, pointer input,
disabled interaction, clamping, fixed/reversed ranges, and direct numeric edits.

The slider appearance follows the supplied macOS Settings screenshot and Apple SwiftUI example:
blue progress, a thin neutral remaining track, and a white capsule thumb with a soft shadow. Dimensions
are an approximation of the reference image, not a platform-native SwiftUI control.

## Optional platform checks

- Optional: listen to VoiceOver announcements and exercise its navigation end to end. The checks above
  inspect the native accessibility tree and activation, not synthesized speech.
- Test Windows/Narrator and Linux/Orca, including their GPU backends.
- Check additional GPUs, display scales, and MSAA configurations; measure frame time in a real
  host application before setting a performance budget.
- Choose an app-level reduced-transparency fallback and check contrast over supported imagery.

Repeat the native checks after changes to shaders, callback ordering, layout, or focus painting.

## Managed integration checks

Run `cargo test -p egui_glass --all-features --lib context_gpu -- --ignored --test-threads=1`
on a machine with a native wgpu adapter. These tests are ignored by default so non-GPU CI
can still run the workspace suite. They check duplicate setup, preservation on invalid
uploads, mirrored registration/replacement, final-owner release and font synchronization
across frames and runtime updates. The managed setup rustdoc example is compile-tested.
