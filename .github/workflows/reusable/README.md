# Reusable Workflows

This directory contains reusable GitHub Actions workflows that can be called from other workflows to avoid duplication and ensure consistency.

## Available Workflows

### Core CI Workflows
- **`detect-changes.yml`** - Detects what type of changes occurred (code vs docs vs version bump)
- **`quick-ci.yml`** - Fast CI checks (lint, format, clippy, codeQL, quick tests, optional codecov)
- **`full-ci.yml`** - Comprehensive CI testing across all platforms (Linux, Windows, macOS)

### Build & Release Workflows
- **`build.yml`** - Build release binaries for all platforms
- **`upload.yml`** - Upload artifacts to GitHub Releases
- **`publish.yml`** - Publish to package managers (Cargo, Homebrew, AUR, Scoop)
- **`validate.yml`** - Validate published packages work correctly

### Legacy
- **`ci.yml`** - Legacy reusable CI workflow (use quick-ci.yml or full-ci.yml instead)

## Usage

These workflows are called from the main orchestrator workflows in the parent directory:

- **`ci.yml`** - Push events (quick CI + codecov)
- **`ci-pr.yml`** - Pull request events (quick + full CI, no codecov)  
- **`release.yml`** - Version bump releases (full pipeline)

## Path Filtering

The `detect-changes.yml` workflow automatically skips CI when only documentation files are changed:

- **Code changes**: `src/`, `Cargo.toml`, `Cargo.lock`, `build.rs`, `.github/workflows/`, `scripts/`, `tests/`
- **Docs changes**: `*.md`, `docs/`, `*.txt`, `LICENSE`, etc.

## Example Usage

```yaml
jobs:
  quick-ci:
    uses: ./.github/workflows/reusable/quick-ci.yml
    with:
      include-codecov: true
      rust-version: 'stable'
```

## Benefits

- **No Duplication**: Single source of truth for CI logic
- **Consistent**: Same tests run everywhere  
- **Efficient**: Smart path filtering skips unnecessary runs
- **Maintainable**: Easy to update CI logic in one place
- **Modular**: Mix and match workflows as needed