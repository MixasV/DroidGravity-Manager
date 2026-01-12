# Building DroidGravity Manager

This guide will help you build DroidGravity Manager from source.

## Prerequisites

### Required Software

1. **Node.js 18+**: [Download](https://nodejs.org/)
2. **Rust**: [Install via rustup](https://rustup.rs/)
3. **System Dependencies** (varies by OS):

#### Windows
- **Microsoft Visual Studio** (with "Desktop development with C++" workload)
- **WebView2**: Usually pre-installed on Windows 10/11

#### macOS
```bash
xcode-select --install
```

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

---

## Build Steps

### 1. Clone and Navigate

```bash
git clone https://github.com/YOUR_USERNAME/DroidGravity-Manager.git
cd DroidGravity-Manager
```

### 2. Install Node Dependencies

**Windows (if PowerShell execution policy error)**:
```powershell
# Run PowerShell as Administrator
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser

# Then install
npm install
```

**macOS/Linux**:
```bash
npm install
```

### 3. Build the Application

**Development Build** (faster, for testing):
```bash
npm run tauri dev
```

**Production Build** (optimized):
```bash
npm run tauri build
```

### 4. Locate the Build Output

After `npm run tauri build` completes:

- **Windows**: `src-tauri\target\release\droidgravity-manager.exe`
- **macOS**: `src-tauri/target/release/bundle/macos/DroidGravity Manager.app`
- **Linux**: `src-tauri/target/release/droidgravity-manager`

---

## Troubleshooting

### Build Fails with "cargo not found"

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env  # Linux/macOS
```

### Windows: "error: linker 'link.exe' not found"

Install Visual Studio with C++ desktop development tools.

### macOS: "xcode-select: error"

Install Xcode Command Line Tools:
```bash
xcode-select --install
```

### Linux: WebKit errors

Install WebKit dependencies:
```bash
sudo apt install libwebkit2gtk-4.0-dev
```

### npm install hangs

Try with legacy peer deps:
```bash
npm install --legacy-peer-deps
```

---

## Development Tips

### Hot Reload (Development Mode)

```bash
npm run tauri dev
```

This starts the app in development mode with hot reload for frontend changes.

### Clean Build

If you encounter weird build errors:

```bash
# Clean Rust build
cd src-tauri
cargo clean
cd ..

# Clean Node modules
rm -rf node_modules package-lock.json  # Linux/macOS
# OR
rmdir /s node_modules  # Windows
del package-lock.json

# Reinstall
npm install
npm run tauri build
```

### Build Logs

Check build logs here:
- Rust: `src-tauri/target/`
- Tauri: Check console output during build

---

## Release Checklist

Before creating a release:

1. ✅ Update version in `src-tauri/Cargo.toml`
2. ✅ Update version in `package.json`
3. ✅ Update `CHANGELOG` section in README
4. ✅ Test on target platform
5. ✅ Build with `npm run tauri build`
6. ✅ Test the built executable
7. ✅ Create GitHub release with binaries

---

**Need help?** Open an issue on GitHub!
