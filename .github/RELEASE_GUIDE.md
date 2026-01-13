# Release Guide

## Making a New Release

Follow these steps to create a new release:

### 1. Update Version

Update version in:
- `Cargo.toml` - Update `version` field
- `CHANGELOG.md` - Add new version section with changes

### 2. Commit Changes

```bash
git add .
git commit -m "chore: bump version to vX.X.X"
git push origin main
```

### 3. Create Release Tag

```bash
# Create and push tag
git tag v0.1.1
git push origin v0.1.1
```

### 4. GitHub Actions Will Automatically:

1. Build binaries for all platforms:
   - `drovity-linux-x64-musl` (static, works everywhere)
   - `drovity-linux-x64` (GNU, GLIBC 2.31+)
   - `drovity-macos-x64`
   - `drovity-macos-arm64`
   - `drovity-windows.exe`

2. Create GitHub Release with all binaries attached

3. Users can install with:
   ```bash
   curl -fsSL https://raw.githubusercontent.com/MixasV/drovity/main/install.sh | bash
   ```

## Build Matrix

| Platform | OS Runner | Target | Output | Notes |
|----------|-----------|--------|--------|-------|
| Linux musl | ubuntu-latest | x86_64-unknown-linux-musl | drovity-linux-x64-musl | Static, max compatibility |
| Linux GNU | ubuntu-20.04 | x86_64-unknown-linux-gnu | drovity-linux-x64 | GLIBC 2.31+ |
| macOS Intel | macos-latest | x86_64-apple-darwin | drovity-macos-x64 | |
| macOS ARM | macos-latest | aarch64-apple-darwin | drovity-macos-arm64 | Apple Silicon |
| Windows | windows-latest | x86_64-pc-windows-msvc | drovity-windows.exe | |

## Install Script Behavior

The `install.sh` script will:

1. Detect OS and architecture
2. For Linux: Try to download musl version first
3. If musl not available: Fallback to GNU version
4. Install to `/usr/local/bin/drovity`

This ensures maximum compatibility across all Linux distributions.
