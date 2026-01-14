# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-01-14

### Fixed
- **Critical**: Google OAuth authorization works with localhost redirect + manual code extraction
  - Works with existing Antigravity credentials (no Device Flow setup needed)
  - User copies authorization code from redirect URL
  - Compatible with headless servers
- **Critical**: Factory Droid model IDs now in correct format (`custom:Model-Name-Index`)
  - Fixes model display issues in Factory Droid CLI
  - All models now properly recognized

### Added
- **Detailed request logging** for debugging:
  - Incoming requests (model, messages, headers)
  - Token refresh status
  - Gemini API requests and responses
  - Content previews for debugging
- **New models**:
  - Gemini 3 Pro Low
  - Gemini 2.5 Flash Lite
  - Gemini 2.5 Flash (Thinking)
  - Claude 4.5 Sonnet (Thinking)
  - Claude 4.5 Opus (Thinking)
- **Smart Factory Droid configuration**: Auto-creates `settings.json` if `.factory` folder exists
  - No need to run `droid` first on fresh installations
  - Perfect for automated server deployments

## [0.1.1] - 2026-01-14

### Added
- **Static musl build for Linux** - all Linux binaries are now statically linked
  - Fixes `GLIBC_2.38 not found` errors on older distributions
  - Works on any Linux distro (Ubuntu, Debian, CentOS, Alpine, etc.)
  - No dependencies on system libraries

### Changed
- All Linux builds now use musl for static linking (no GLIBC dependency)
- Simplified installation script - always downloads musl build for Linux
- Updated documentation to reflect static builds

### Fixed
- **Critical**: Compatibility with all Linux distributions
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
