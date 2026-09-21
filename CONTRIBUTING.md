# Contributing

Bug reports, documentation fixes, and focused pull requests are welcome. For a new public API
or a substantial visual change, open an issue describing the use case first.

## Development

Install Rust 1.92 or newer and clone the repository. Run commands from the repository root:

```bash
cargo run -p egui_glass_demo
cargo run -p egui_glass_demo --example minimal
cargo test -p egui_glass --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc -p egui_glass --all-features --no-deps --locked
```

The demo needs a graphical session and a wgpu-compatible GPU. Linux builds also need the
platform development libraries required by eframe/winit and the demo's file-dialog backend.
CI currently checks the library on Linux, macOS, and Windows and builds the demo on macOS
and Windows. The minimum Rust version is checked separately for the library.

On a machine with a native wgpu adapter, also run:

```bash
cargo test -p egui_glass --all-features --locked --lib context_gpu -- --ignored --test-threads=1
cargo run -p egui_glass_demo --example live_benchmark --release --locked
```

The GPU tests check managed initialization, font synchronization and texture ownership.
They are ignored in the default suite. The benchmark checks rendered pixels before timing;
see [verification scope](docs/verification.md) and [measurement limits](docs/performance.md).

## Repository layout

- `crates/egui_glass`: public API, widgets, GPU renderer, and WGSL shaders.
- `examples/demo`: interactive demo and the minimal integration example.
- `examples/demo/examples/live_benchmark.rs`: headless rendering checks and frame timings.
- `docs/guide.md`: the short integration path.
- `docs/reference.md`: API tables, defaults, renderer details, and limitations.
- `assets/branding` and `examples/asset`: repository/demo assets; see [provenance](docs/assets.md).

Keep the library dependency-light. Application fonts, dialogs, and layouts belong in the demo.
Match the surrounding code style and avoid unrelated formatting. Include a regression test for
behavior changes, update API docs and the reference for public changes, and add an entry under
Unreleased in the changelog. Use commit titles such as `fix: preserve backdrop mapping`.

For shader or visual changes, also run the demo and attach before/after screenshots over a
busy image and a plain background. Check light/dark presets, static/live backdrops, resize,
scrolling, and display scaling. CPU tests and compilation do not validate GPU appearance.

## Reporting a bug

Use [GitHub issues](https://github.com/heonny/egui-glass/issues). Include:

- A minimal reproduction and the expected/actual behavior.
- Crate, Rust, egui, and wgpu versions.
- OS, GPU, graphics backend, display scale, and MSAA setting.
- Static or live backdrop mode, screenshots, and relevant validation errors.

Remove credentials and private application data from logs and screenshots.

## Release checklist (maintainers)

1. Review the diff, update the changelog and version, and refresh `Cargo.lock` if needed.
2. Run the checks above, including default features separately from the serde-enabled workspace.
   Build docs with `RUSTDOCFLAGS="-D warnings"` and check the minimum Rust version with
   `cargo +1.92.0 check -p egui_glass --all-features --locked`.
3. Visually check the demo on the target GPUs. Keep [asset credits](docs/assets.md) up to date
   when changing demo images; library packaging does not include those assets.
4. Keep `crates/egui_glass/LICENSE` identical to the root `LICENSE`. Run
   `cargo package -p egui_glass --list` and `cargo publish -p egui_glass --dry-run --locked`
   from a clean tree. Confirm the package contains the license, README, and both WGSL shaders.
5. Commit the release, wait for CI to pass on that commit, then tag it `vX.Y.Z` and push the tag.
   The release workflow verifies the tag/version, tests the library, builds docs, performs a
   publish dry run, and publishes using the repository's `CARGO_REGISTRY_TOKEN` secret.

Publishing and pushing a release tag are maintainer actions. Do not put registry tokens in files.
