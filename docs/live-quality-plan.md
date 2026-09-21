# Live backdrop quality implementation plan

Scope: approved fourth improvement, implemented inline. Assurance tier B for the additive
public rendering API. Existing APIs remain compatible; changes are recorded under Unreleased.

## Requirements and verification

| Requirement | Implementation | Verification |
|---|---|---|
| Existing `run` keeps full resolution | `live.rs` delegates with Full | default/unit tests and Full GPU benchmark |
| Opt-in 1×, 0.75×, 0.5× rendering | `LiveBackdropQuality`, `run_with_quality` | dimension tests, demo selector, GPU runs |
| Preserve logical layout and refraction coordinates | full-size projection, reduced viewport | odd sizes, HiDPI, GPU landmark readback |
| Preserve requested blur units | scale only mip LOD footprint | uniform/shader review, GPU validation |
| Handle resize, switching, and minimized targets | reuse by actual texture dimensions; skip zero screen | unit and GPU smoke scenarios |
| Reproducible performance evidence | headless example using real renderer | Full/Balanced/Performance timings with scope and hardware recorded |

## Execution

1. Add failing size/default tests; implement a discrete quality enum and per-run option.
2. Reduce the offscreen target while retaining the original projection; carry sampling
   density into the glass shader. Keep static backdrop behavior unchanged.
3. Add a demo quality selector and backward-compatible optional settings field.
4. Add a headless GPU benchmark/verification example, measure all qualities, document
   timing limitations and quality tradeoffs. `pollster` is a dev-only dependency already
   present in Cargo.lock, used solely to drive wgpu initialization futures.
5. Run workspace tests, Clippy and API docs; separately review the complete diff against
   the four review criteria, especially lifecycle, zero/odd sizes, scaling and defaults.

No automatic quality selection, caching, frame skipping or changes to the two-layout
closure contract. Full remains the default. Lower resolution can soften small text.

## Completion evidence

- Workspace tests: 30 passed, including 2 quality tests and legacy/new settings coverage.
- Clippy: all workspace targets/features passed with warnings denied.
- API docs: missing docs and warnings denied, passed.
- Release GPU benchmark: Metal / M1 Pro pixel readback passed; results and limitations
  are in `performance.md` and `performance-2026-09-21.csv`.
- Separate diff review: checked defaults, uniform layout, live/static scale reset,
  dimension rounding, unchanged static shader behavior, texture reuse and dependency scope.
  Native minimized windows and custom callback visuals remain explicitly unverified.
- Harness correction: use `ViewportInfo::native_pixels_per_point`, not a queued zoom
  change, when initializing HiDPI test input. `Context::viewport_rect` rounds to 1/32 point;
  compare its documented rounded value rather than raising arbitrary float tolerances.
