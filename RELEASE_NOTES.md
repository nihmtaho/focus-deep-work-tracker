## Release Infrastructure & CI/CD Setup

This patch release focuses on release process automation and developer experience improvements.

### 🔧 What's New

- **Release Workflow**: Full Git Flow implementation (release branches → PR → tag → automated builds)
- **CI/CD Pipelines**: GitHub Actions workflows for continuous testing and cross-platform binary builds
- **Conventional Commits**: Local `commit-msg` hook enforces consistent commit message format
- **Developer Tools**: Makefile for one-command operations (`make setup`, `make test`, `make release`)

### 📋 Technical Details

**GitHub Actions:**
- `ci.yml`: Runs cargo clippy, cargo test, and validates commit messages on every PR/push
- `release.yml`: Automatically builds release binaries for Linux (x86_64), macOS (x86_64 + arm64), and Windows (x86_64) when a version tag is pushed

**Local Setup:**
```bash
make setup    # Install git hooks (one-time)
make release VERSION=1.2.1   # Create and prepare a release branch
```

**Conventional Commits:** All commits must follow the format:
```
<type>(<scope>): <subject>
```
Supported types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`

### 📦 Install / Upgrade

```bash
# via cargo
cargo install --force focus
```

**Full Changelog:** https://github.com/nihmtaho/focus-deep-work-tracker/compare/v1.2.0...v1.2.1
