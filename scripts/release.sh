#!/usr/bin/env bash
# release.sh — Git Flow release helper
#
# Usage:
#   ./scripts/release.sh <version>        # e.g. 1.3.0
#   ./scripts/release.sh <version> patch  # patch bump
#   ./scripts/release.sh <version> minor  # minor bump
#   ./scripts/release.sh <version> major  # major bump
#
# What it does:
#   1. Creates release/v<version> branch from develop
#   2. Updates version in Cargo.toml
#   3. Generates changelog entry and prepends to CHANGELOG.md
#   4. Commits the version bump
#
# After running:
#   - Review the generated CHANGELOG.md entry
#   - Push: git push origin release/v<version>
#   - Open PR: release/v<version> → main
#   - After merge: tag main with v<version> → triggers GitHub Actions release

set -euo pipefail

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>  (e.g. 1.3.0)"
  exit 1
fi

# Strip leading 'v' if provided
VERSION="${VERSION#v}"
TAG="v${VERSION}"
BRANCH="release/${TAG}"

echo "🚀  Starting release ${TAG}..."
echo ""

# Ensure we're on develop and it's up to date
CURRENT=$(git branch --show-current)
if [ "$CURRENT" != "develop" ]; then
  echo "⚠️  Switching to develop first..."
  git checkout develop
fi

git pull origin develop

# Check working directory is clean
if [ -n "$(git status --porcelain)" ]; then
  echo "❌  Working directory is not clean. Commit or stash your changes first."
  exit 1
fi

# Create release branch
echo "📦  Creating branch ${BRANCH}..."
git checkout -b "${BRANCH}"

# Update version in Cargo.toml
echo "📝  Updating Cargo.toml version to ${VERSION}..."
sed -i.bak "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml
rm -f Cargo.toml.bak

# Update Cargo.lock
cargo update --workspace --quiet 2>/dev/null || true

# Generate changelog entry using git-cliff (requires: cargo install git-cliff)
if command -v git-cliff &> /dev/null; then
  echo "📋  Generating CHANGELOG entry..."
  ENTRY=$(git cliff --unreleased --tag "${TAG}" --strip all 2>/dev/null || echo "")
  if [ -n "$ENTRY" ]; then
    TEMP=$(mktemp)
    printf '%s\n\n' "$ENTRY" > "$TEMP"
    # Prepend after the first line (header) of CHANGELOG.md
    HEAD=$(head -1 CHANGELOG.md)
    TAIL=$(tail -n +2 CHANGELOG.md)
    printf '%s\n\n%s\n%s' "$HEAD" "$ENTRY" "$TAIL" > CHANGELOG.md
    rm -f "$TEMP"
    echo "   ✅  CHANGELOG.md updated"
  else
    echo "   ⚠️  No unreleased commits found for changelog — update CHANGELOG.md manually"
  fi
else
  echo "   ⚠️  git-cliff not found. Install with: cargo install git-cliff"
  echo "       Skipping CHANGELOG auto-generation — update CHANGELOG.md manually"
fi

# Commit
echo "💾  Committing version bump..."
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore(release): bump version to ${VERSION}"

echo ""
echo "✅  Release branch ${BRANCH} is ready!"
echo ""
echo "Next steps:"
echo "  1. Review CHANGELOG.md and adjust if needed"
echo "  2. git push origin ${BRANCH}"
echo "  3. Open PR: ${BRANCH} → main"
echo "  4. After merge to main, tag it:"
echo "     git checkout main && git pull origin main"
echo "     git tag -a ${TAG} -m 'Release ${VERSION}'"
echo "     git push origin ${TAG}"
echo "  5. GitHub Actions will build and publish the release automatically"
echo "  6. Merge main back to develop:"
echo "     git checkout develop && git merge --no-ff main && git push origin develop"
