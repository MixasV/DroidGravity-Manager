# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-01-14

### Added
- **Static musl builds for Linux** (`drovity-linux-x64-musl`) for maximum compatibility
  - Fixes `GLIBC_2.38 not found` errors on older Linux distributions
  - Statically linked binary works on any Linux distro
- Automatic fallback in install script: tries musl version first, then GNU version

### Changed
- Linux GNU builds now use Ubuntu 20.04 (GLIBC 2.31+) instead of latest Ubuntu
- Install script now intelligently selects the best binary for your system
- Updated README with GLIBC troubleshooting section

### Fixed
- **Critical**: Compatibility with older Linux servers (Ubuntu 18.04, 20.04, Debian 10, CentOS 7, etc.)
- GLIBC version mismatch errors on production servers

## [0.1.0] - 2026-01-13

### Added
- Initial release of Drovity CLI proxy
- Google OAuth authentication with OOB flow (out-of-band)
- Automatic token refresh for Google accounts
- API proxy server on port 8045 (configurable)
- Factory Droid integration with auto-configuration
- Interactive CLI menu for easy management
- Daemon mode (background operation)
- Support for Gemini models (3 Flash, 3 Pro, 2.5 Flash, 2.5 Pro, Thinking variants)
- Support for Claude models (4.5 Sonnet, Opus with Thinking)
- Automatic conversion of Factory Droid request format to OpenAI-compatible format
- Cross-platform support (Linux x64/ARM64, macOS Intel/Apple Silicon, Windows x64)
- One-line installation script for Linux/macOS
- Comprehensive README with usage examples
- GitHub Actions for automated releases

### Technical Details
- Built with Rust for performance and reliability
- Uses Axum web framework for HTTP server
- OAuth 2.0 implementation with Google APIs
- SQLite for local data storage
- Dialoguer for interactive CLI menus
- Support for multiple accounts with automatic rotation

### Known Limitations
- All models (including Claude) work through Google Gemini API only
- Requires Google account authorization (no Anthropic account needed)
- Windows daemon mode requires manual process management

---

**Author**: [@onexv](https://t.me/onexv)  
**Repository**: https://github.com/MixasV/drovity  
**License**: MIT
