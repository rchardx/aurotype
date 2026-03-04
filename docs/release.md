# Release Pipeline

Aurotype uses [release-please](https://github.com/googleapis/release-please) for automated versioning and releases.

## How It Works

```
push to main
  → release-please creates/updates a Release PR
    (bumps version in package.json, Cargo.toml, tauri.conf.json, pyproject.toml,
     generates CHANGELOG.md)
  → sync-cargo-lock updates Cargo.lock on the PR branch
  → merge the Release PR
    → GitHub Release + tag (e.g. v0.2.0) created automatically
      → Windows build triggered
        → .msi / .exe installers uploaded as release assets
```

## Version Management

Version is defined in **4 files**, all kept in sync automatically by release-please:

| File                        | Field              |
| --------------------------- | ------------------ |
| `package.json`              | `version`          |
| `src-tauri/tauri.conf.json` | `version`          |
| `src-tauri/Cargo.toml`      | `package.version`  |
| `engine/pyproject.toml`     | `project.version`  |

**Never edit version numbers manually.** Release-please handles this via the Release PR.

## Triggering a Release

1. Push commits to `main` using [Conventional Commits](https://www.conventionalcommits.org/) format
2. Release-please automatically creates (or updates) a Release PR
3. The PR title shows the next version; the body contains the generated CHANGELOG
4. **Merge the Release PR** to trigger the release

### Commit Types → Version Bumps

| Commit prefix | Version bump | Example                              |
| ------------- | ------------ | ------------------------------------ |
| `feat`        | minor        | `feat(engine): add Whisper STT`      |
| `fix`         | patch        | `fix(tauri): handle sidecar crash`   |
| `feat!`       | major        | `feat!: redesign config format`      |
| `chore`       | no release   | `chore(deps): bump tokio`            |
| `docs`        | no release   | `docs: update README`                |
| `test`        | no release   | `test(tauri): add state.rs tests`    |

While the project is pre-1.0 (`0.x.y`), breaking changes bump minor and features bump patch
(configured via `bump-minor-pre-major` and `bump-patch-for-minor-pre-major` in `release-please-config.json`).

## Configuration Files

| File                             | Purpose                                              |
| -------------------------------- | ---------------------------------------------------- |
| `release-please-config.json`     | Release-please settings, extra-files for version sync |
| `.release-please-manifest.json`  | Current version tracker (updated by release-please)   |
| `.github/workflows/release.yml`  | Combined workflow: release-please + Cargo.lock sync + Windows build |

## Build Pipeline

The release workflow (`.github/workflows/release.yml`) runs in three stages:

### Stage 1: release-please

- Runs on every push to `main`
- Creates or updates a Release PR with version bump + CHANGELOG
- When the Release PR is merged, creates a GitHub Release and outputs `release_id`

### Stage 2: sync-cargo-lock

- **Only runs when a Release PR is created or updated** (conditional on stage 1)
- Checks out the Release PR branch
- Runs `cargo update --workspace` to sync `src-tauri/Cargo.lock` with the updated `Cargo.toml` version
- Commits and pushes the updated lockfile back to the PR branch
- This is needed because release-please updates `Cargo.toml` version but cannot update `Cargo.lock`

### Stage 3: build-windows

- **Only runs when a release is created** (conditional on stage 1)
- Builds the Python sidecar via PyInstaller (onefile mode)
- Builds the Tauri app via `tauri-action`
- Uploads `.msi` and `.exe` installers to the GitHub Release

## Adding macOS Builds (Future)

To add macOS support, add a `build-macos` job to `release.yml` with:

- `runs-on: macos-latest`
- Apple Developer certificate secrets (`APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`)
- Notarization secrets (`APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`)
- `tauri-action` with `--target universal-apple-darwin` for universal binary

## Troubleshooting

### Release PR not appearing

- Ensure commits use Conventional Commits format (`feat:`, `fix:`, etc.)
- `chore:`, `docs:`, `test:` commits alone won't trigger a release
- Check the Actions tab for release-please workflow errors

### Version mismatch across files

- Never edit version numbers manually — let release-please handle it
- If versions are out of sync, manually update `.release-please-manifest.json` to the correct version and push

### Build failure on release

- Check the Actions tab for the failing `build-windows` job
- The GitHub Release is already created; fix the issue and re-run the workflow
- Alternatively, delete the release/tag, fix, and re-trigger
