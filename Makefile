# Shortcuts for common tasks. See README.md / CLAUDE.md.
#
#   make            build the release demo and the macOS app -> target/release/bundle
#   make install    same, then copy the app into /Applications
#   make run        run the demo in development
#   make test       unit tests
#   make lint       clippy, warning-free

.PHONY: bundle install run test lint

bundle:
	./scripts/bundle-macos.sh

install:
	./scripts/bundle-macos.sh --install

run:
	cargo run -p egui_glass_demo

test:
	cargo test --workspace --all-features

lint:
	cargo clippy --workspace --all-targets --all-features
