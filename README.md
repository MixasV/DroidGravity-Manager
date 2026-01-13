# Drovity

[![Release](https://img.shields.io/github/v/release/MixasV/drovity?style=flat-square)](https://github.com/MixasV/drovity/releases)
[![Build Status](https://img.shields.io/github/actions/workflow/status/MixasV/drovity/release.yml?style=flat-square)](https://github.com/MixasV/drovity/actions)
[![License](https://img.shields.io/github/license/MixasV/drovity?style=flat-square)](LICENSE)

**Google Gemini API Proxy for Factory Droid**

A lightweight CLI tool that provides a local API proxy for Google Gemini models, designed for seamless integration with Factory Droid.

---

**Author**: [@onexv](https://t.me/onexv)  
**Repository**: [github.com/MixasV/drovity](https://github.com/MixasV/drovity)

---

## Quick Start

### Installation

#### One-line install

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/MixasV/drovity/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/MixasV/drovity/main/install.ps1 | iex
```

#### Manual installation

1. Download binary for your system from [Releases](https://github.com/MixasV/drovity/releases):
   - **Linux x64**: `drovity-linux-x64-musl` (static build, recommended for older systems) or `drovity-linux-x64` (requires GLIBC 2.31+)
   - **macOS Intel**: `drovity-macos-x64`
   - **macOS Apple Silicon**: `drovity-macos-arm64`
   - **Windows**: `drovity-windows.exe`

2. Install:

**Linux/macOS:**
```bash
chmod +x drovity-*
sudo mv drovity-* /usr/local/bin/drovity
```

**Windows:**
Move `drovity-windows.exe` to a directory in your PATH (e.g., `C:\Program Files\drovity\`) and rename to `drovity.exe`

That's it! Run `drovity` to start.

### Quick Start

After installation, just run:
```bash
drovity
```

This opens the interactive menu where you can add Google accounts and configure the proxy.

## Features

- Google OAuth Authentication - Secure account management with OOB flow
- Automatic Token Refresh - Handles token expiration automatically
- Factory Droid Integration - Auto-configure Factory Droid settings
- API Proxy Server - Local proxy on port 8045 (customizable)
- Daemon Mode - Run proxy in background
- Cross-Platform - Linux, macOS, and Windows support

## Usage Guide

### 1. Add Google Account

```bash
drovity
# Select: 1. Accounts
# Select: 1. Add New Account
# Follow the OAuth flow:
#   1. Copy and open the URL in your browser
#   2. Authorize with your Google account
#   3. Copy the authorization code from Google
#   4. Paste it back in the terminal
# Account added!
```

**Note**: All models (including Claude) work through Google Gemini API. You only need Google account authorization.

### 2. Start Proxy

**From menu:**
```bash
drovity
# Select: 2. API Proxy
# Select: 1. Start Proxy
```

**Or directly:**
```bash
drovity start  # Run in foreground
drovity hide   # Run in background (daemon)
```

The proxy will start on `http://127.0.0.1:8045`

### 3. Configure Factory Droid

**Automatic setup:**
```bash
drovity
# Select: 3. Droid Settings Setup
# Select: 1. Auto Config Droid Settings
```

This automatically updates `~/.factory/settings.json` with all required models.

**Manual setup:**
If auto-config fails, select "2. Manual Setup" to see the JSON configuration and copy it manually to `~/.factory/settings.json`

### 4. Use with Factory Droid

```bash
droid
> /model
# Select any model with [drovity] suffix
# Example: "Gemini 2.5 Flash [drovity]"
# Start chatting!
```

All models (Gemini and Claude) use `"provider": "anthropic"` and work through Google Gemini API.

## Commands

```bash
drovity              # Start interactive menu (default)
drovity menu         # Start interactive menu
drovity start        # Start proxy in foreground
drovity hide         # Start proxy in background
drovity stop         # Stop background proxy
drovity status       # Check proxy status
```

## Configuration

Drovity stores data in `~/.drovity/`:
- `config.json` - Proxy configuration
- `accounts/*.json` - Account data
- `drovity.pid` - Process ID (when running in background)
- `proxy.log` - Server logs

## Security

- OAuth credentials are stored locally only
- No data is sent to third-party servers
- API key is generated locally
- Uses the same Google OAuth credentials as Antigravity Manager

## Troubleshooting

### GLIBC version error (Linux)

If you get an error like `GLIBC_2.38' not found`:

**Solution 1 (Recommended)**: Use the static musl build:
```bash
# Download musl version
curl -L -o drovity https://github.com/MixasV/drovity/releases/latest/download/drovity-linux-x64-musl
chmod +x drovity
sudo mv drovity /usr/local/bin/
```

**Solution 2**: The install script will automatically try the musl version first:
```bash
curl -fsSL https://raw.githubusercontent.com/MixasV/drovity/main/install.sh | bash
```

The musl build is statically linked and works on any Linux distribution regardless of GLIBC version.

### Proxy won't start
```bash
# Check if port 8045 is already in use
lsof -i :8045  # Linux/macOS
netstat -ano | findstr :8045  # Windows
```

Edit `~/.drovity/config.json` to change the port if needed.

### Factory Droid can't connect
1. Verify proxy is running: `drovity status`
2. Check API key matches in both drovity and Factory settings
3. **Important**: All models must use `"provider": "anthropic"` (not "openai")
4. Base URLs:
   - Gemini: `http://127.0.0.1:8045/` (with trailing slash)
   - Claude: `http://127.0.0.1:8045` (no trailing slash)

### OAuth authorization fails
- Copy the **entire** authorization code from Google
- Revoke old access at https://myaccount.google.com/permissions and try again
- Check internet connection

## Credits

Based on [DroidGravity-Manager](https://github.com/MixasV/DroidGravity-Manager), which is a fork of [Antigravity-Manager](https://github.com/lbjlaq/Antigravity-Manager) by [@lbjlaq](https://github.com/lbjlaq).

## License

MIT License

## Contributing

Contributions are welcome! Please open an issue or pull request.

---

**Contact**: [@onexv on Telegram](https://t.me/onexv)
