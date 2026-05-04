#!/usr/bin/env bash
# build-release.sh — Local release build helper
#
# Usage:
#   ./scripts/build-release.sh [--target <target>] [--all]
#
# Options:
#   --target <triple>  Build a single target (e.g. aarch64-apple-darwin)
#   --all              Build all supported targets (default if no flag given)
#   --help             Show this message
#
# Supported targets (mirrors GitHub Actions release.yml):
#   x86_64-unknown-linux-gnu
#   x86_64-apple-darwin
#   aarch64-apple-darwin
#   x86_64-pc-windows-msvc
#
# Output: dist/focus-<version>-<target>[.exe]

set -euo pipefail

ALL_TARGETS=(
  "x86_64-unknown-linux-gnu"
  "x86_64-apple-darwin"
  "aarch64-apple-darwin"
  "x86_64-pc-windows-msvc"
)

usage() {
  grep '^#' "$0" | grep -v '^#!/' | sed 's/^# \{0,1\}//'
  exit 0
}

# Parse args
TARGETS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      shift
      TARGETS+=("$1")
      ;;
    --all)
      TARGETS=("${ALL_TARGETS[@]}")
      ;;
    --help|-h)
      usage
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
  shift
done

# Default: build all
if [[ ${#TARGETS[@]} -eq 0 ]]; then
  TARGETS=("${ALL_TARGETS[@]}")
fi

# Read version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
TAG="v${VERSION}"

echo "Building focus ${TAG} for ${#TARGETS[@]} target(s)..."
echo ""

mkdir -p dist

BUILT=()
FAILED=()

for TARGET in "${TARGETS[@]}"; do
  echo "--- ${TARGET} ---"

  # Ensure the Rust target is installed
  if ! rustup target list --installed | grep -q "^${TARGET}$"; then
    echo "  Installing target ${TARGET}..."
    rustup target add "${TARGET}"
  fi

  BINARY="focus"
  [[ "$TARGET" == *windows* ]] && BINARY="focus.exe"

  ARCHIVE_NAME="focus-${TAG}-${TARGET}"
  [[ "$TARGET" == *windows* ]] && ARCHIVE_NAME="${ARCHIVE_NAME}.exe"
  DEST="dist/${ARCHIVE_NAME}"

  if cargo build --release --target "${TARGET}" 2>&1; then
    cp "target/${TARGET}/release/${BINARY}" "${DEST}"
    [[ "$TARGET" != *windows* ]] && chmod +x "${DEST}"
    echo "  OK -> ${DEST}"
    BUILT+=("$DEST")
  else
    echo "  FAILED (skipping)"
    FAILED+=("$TARGET")
  fi

  echo ""
done

echo "=============================="
echo "Build summary for ${TAG}"
echo "=============================="

if [[ ${#BUILT[@]} -gt 0 ]]; then
  echo ""
  echo "Built (${#BUILT[@]}):"
  for f in "${BUILT[@]}"; do
    SIZE=$(du -sh "$f" 2>/dev/null | cut -f1)
    echo "  ${f}  (${SIZE})"
  done
fi

if [[ ${#FAILED[@]} -gt 0 ]]; then
  echo ""
  echo "Failed (${#FAILED[@]}):"
  for t in "${FAILED[@]}"; do
    echo "  ${t}"
  done
  echo ""
  echo "Note: cross-compilation (e.g. Windows from macOS) requires additional"
  echo "toolchains or cross (https://github.com/cross-rs/cross)."
  exit 1
fi

echo ""
echo "All artifacts are in dist/"
