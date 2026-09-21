# Live backdrop quality and performance

The development version provides `LiveBackdrop::run_with_quality`. Existing `run` calls
use `LiveBackdropQuality::Full`, so applications opt in to lower backdrop resolution.

```rust
live.run_with_quality(ui, clear_color, egui_glass::LiveBackdropQuality::Performance, |page| {
    page.label("Refracted content");
});
```

| Setting | Width/height | Backdrop pixels and mip storage, approximately |
|---|---|---|
| Full | 1× | 100% |
| Balanced | 0.75× | 56.25% |
| Performance | 0.5× | 25% |

The main page, glass shapes, input coordinates, refraction displacement and requested blur
remain in their original coordinate system. Lower quality softens refracted small text
and fine detail. It reduces off-screen rasterization and mipmap work, but does not remove
the second layout, texture updates, or glass fragment shading. It is not a cache or frame cap.
The demo includes a Quality selector; old settings JSON defaults to Full.

## Reproduce

```sh
cargo run -p egui_glass_demo --example live_benchmark --release
```

Requires a usable native wgpu adapter; failure to initialize or validate is an error, not
a skipped success. The example uses the real egui-wgpu renderer without a window. It first
reads rendered pixels back from the GPU to check four colour landmarks through clear glass
while changing quality, returning to static mode, resizing, and using odd dimensions and
HiDPI. It then warms up each case for 20 frames and measures 100 frames.

The scene contains 300 text labels and 1 or 16 glass surfaces, each 160×90 logical points.
1280×720 at 1 point per pixel and 2560×1440 at 2 pixels per point use the same logical layout.
Static mode samples a 64×64 colour backdrop; live mode also includes the page's text.
Static and live are therefore different visual workloads, not interchangeable results.

- `layout_median_ms`: main UI, twin UI when live, and tessellation CPU time.
- `frame_median_ms` / `frame_p95_ms`: wall time from UI start through GPU completion,
  including texture updates, command encoding, driver submission and a blocking device poll.
- These are **not GPU timestamp measurements**, FPS, or presentation/vsync latency.
- Runs are sequential. GPU clocks, thermal state and other applications can affect results.
  Repeat on the intended hardware and scene before choosing a default.

## Local observation — 2026-09-21

Apple M1 Pro, Metal, macOS, release profile, egui/eframe 0.35, wgpu 29, one sample per pixel.
One run; [raw results](performance-2026-09-21.csv). Values below are frame median milliseconds.

| Physical size | Glass surfaces | Static | Full | Balanced | Performance |
|---|---:|---:|---:|---:|---:|
| 1280×720 | 1 | 1.591 | 2.345 | 2.199 | 2.302 |
| 1280×720 | 16 | 5.860 | 6.695 | 6.627 | 6.691 |
| 2560×1440 | 1 | 3.048 | 5.170 | 5.133 | 3.660 |
| 2560×1440 | 16 | 10.455 | 10.506 | 8.966 | 7.265 |

Half-resolution reduced the measured high-resolution frame median by about 29–31% in
this run. At 1280×720 the difference was small. This is not a cross-device guarantee;
the resolution option is most useful when off-screen GPU work is a meaningful part of
frame cost. Full remains the default.

## Verification limits

GPU pixel readback passed for 801×603 at 1.25× DPI, 1280×720 at 1× DPI, and 2560×1440 at
2× DPI, across Full → Performance → Static → Balanced → Full. Shader and GPU resource
validation passed on this Metal adapter. Unit tests cover zero/tiny target calculations
and legacy settings import. The zero-screen early return was code-reviewed; minimized
native-window behavior was not exercised. Visual blur quality, custom paint callbacks,
other GPU backends and the native demo selector still require platform testing.

A later compatibility run after managed-context integration passed the same pixel checks,
but timings varied: at 2560×1440 with 16 glass surfaces, Full measured 8.434 ms and
Performance 9.075 ms. The initial single-run reduction is workload/run-specific; reduced
resolution is not a guaranteed speedup. Repeat measurements on the target app and device.
