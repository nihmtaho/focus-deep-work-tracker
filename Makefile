.PHONY: setup test lint release build-release release-patch release-minor release-major

## setup: Install git hooks (run once after cloning)
setup:
	git config core.hooksPath .githooks
	@echo "✅  Git hooks installed from .githooks/"

## test: Run all tests
test:
	cargo test

## lint: Run clippy
lint:
	cargo clippy -- -D warnings

## release VERSION=x.y.z: Create release branch (requires VERSION)
release:
	@test -n "$(VERSION)" || (echo "❌  Usage: make release VERSION=1.3.0"; exit 1)
	@bash scripts/release.sh $(VERSION)

## build-release: Build release binaries for all targets into dist/ (use TARGET=<triple> for one)
build-release:
	@bash scripts/build-release.sh $(if $(TARGET),--target $(TARGET),--all)

## help: Show available targets
help:
	@grep -E '^## ' Makefile | sed 's/^## //'
