# Managed glass context

Approved fifth improvement, implemented inline. Tier B: additive public API and GPU
resource ownership. Legacy functions remain supported; changes are recorded under Unreleased.

| Requirement | Implementation | Verification |
|---|---|---|
| One setup call, deterministic registration order | `GlassContext::new`, internal live constructor | headless GPU construction and repeated-construction rejection |
| Do not overwrite existing renderer resources | check/initialize under one renderer write lock | duplicate managed/legacy initialization tests |
| Fonts stay synchronized when set through the manager | `GlassContext::set_fonts` | twin/main font definitions in test passes |
| Invalid images fail before replacing a working backdrop | checked dimensions and pixel count | unit cases and GPU texture-id preservation |
| Native textures are mirrored and freed once | cloneable `GlassTexture`, final-owner Drop | main/twin registration and final-release checks |
| Existing apps continue to work | retain standalone APIs | existing tests and headless live benchmark |

The manager retains the render state and a twin renderer even for static-only use; an
offscreen frame is rendered only when requested. `live_backdrop()` exposes the existing
live API, including quality selection and its repeated-closure contract. The manager
does not infer MSAA from eframe, automatically capture custom fonts installed elsewhere,
or override an existing setup. Keep texture handles through rendering; do not free their
IDs with the low-level functions. Shared pipelines/backdrops retain renderer lifetime.

Execution: failing image validation tests → managed API and texture ownership → migrate
demo setup → GPU lifecycle verification → workspace tests, Clippy, docs and diff review.

## Verification result

- Workspace all-feature suite: 32 passed, including 3 compiling documentation examples.
- Three explicitly enabled native-GPU tests passed: duplicate/invalid setup and backdrop
  replacement, final-owner release in both renderers, visible/twin fonts across frames,
  updates, clones and the legacy constructor.
- The font test first failed on the twin pass: copying main memory erased pending fonts.
  Queue font changes outside egui memory and install them after the copy. Regression passes.
- All-target/all-feature Clippy with warnings denied and rustdoc with missing docs/warnings
  denied passed. `git diff --check` passed.
- Separate review checked SOLID, error paths, compatibility, lock order, ownership and
  dependency direction. Documented final-drop locking and frame lifetime requirements.
  No runtime dependency added; pollster is test-only and already present in the workspace.
- Scope deferral: the existing demo main module exceeds the 300-line soft limit; splitting
  unrelated scene code is deferred to avoid expanding this integration change (under 500).
- Native-window visual QA remains outstanding; headless checks do not verify presentation.
- Legacy headless benchmark completed on Apple M1 Pro / Metal: pixel readback passed for
  quality changes, resize, odd sizes and HiDPI. Timings vary between runs; this check
  establishes rendering compatibility, not a new performance claim.
