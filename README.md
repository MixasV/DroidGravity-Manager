# DroidGravity Manager 🚀

> [!CAUTION]
> **DO NOT SYNC WITH UPSTREAM (lbjlaq/Antigravity-Manager)!**
> This is a separate standalone project. Syncing with upstream will overwrite Droid-specific changes and localized branding.

**Version 1.1.4**

A fork of [Antigravity-Manager](https://github.com/lbjlaq/Antigravity-Manager) with **Factory Droid** support for seamless integration with Google Gemini and Anthropic Claude models.

## 🌟 What's New in v1.1.4

- **Protocol Synchronization**: Harmonized rate limiting and error handling across all protocols (OpenAI, Claude, Gemini, and Audio).
- **Fine-Grained Rate Limiting**: Implemented async-aware rate limit tracking with support for specific model weights.
- **Enhanced Monitoring**: Updated database schemas and models to support better log filtering and detailed request metadata.
- **Improved Stability**: Resolved unresolved names and ensured thread-safe token management during high-concurrency requests.

## 🌟 What's New in v1.1.0

- **Rebranded**: UI updated from "Antigravity Tools" to "DroidGravity Manager"
- **Update Checker**: Now checks for updates from this repository
- **Flash Lite Fix**: Replaced `gemini-2.5-flash-lite` with `gemini-2.5-flash` to fix 429 rate limit errors
- **Multi-Wildcard Routing**: Support for patterns like `claude-*-sonnet-*` in model mapping
- **Specificity-Based Priority**: More specific wildcard rules now take precedence

### Key Features

- **Factory Droid Support**: Automatic conversion of Factory Droid's request format to OpenAI-compatible format
- **Gemini Integration**: Full support for Google Gemini models (3 Flash, 3 Pro, 2.5 Flash, 2.5 Pro, Thinking variants)
- **Claude Integration**: Native Anthropic API support for Claude 3.5 Sonnet and Opus
- **Automatic Account Rotation**: Seamlessly switches between accounts when hitting rate limits
- **No Manual Configuration**: Pre-configured model settings included

---

## 📦 Installation

### Prerequisites

- **Node.js** 18+ and npm
- **Rust** toolchain (for Tauri)
- **Windows**, macOS, or Linux

### Build from Source

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/DroidGravity-Manager.git
cd DroidGravity-Manager

# Install dependencies
npm install

# Build the application
npm run tauri build
```

The compiled application will be in `src-tauri/target/release/`.

---

## ⚙️ Setup for Factory Droid

### Step 1: Configure DroidGravity Manager

1. **Launch DroidGravity Manager**
2. **Add Accounts**:
   - For Gemini: Go to **Accounts** → **Add Account** → **OAuth** → Authorize with your Google account
   - For Claude: Go to **Accounts** → **Add Account** → **OAuth** → Authorize with your Anthropic account
3. **Start the Proxy**: Navigate to **API Proxy** and enable the server (default port: `8045`)

### Step 2: Configure Factory Droid

1. **Locate Factory Settings**:
   - Open `~/.factory/settings.json` (Linux/macOS) or `C:\Users\YOUR_USERNAME\.factory\settings.json` (Windows)

2. **Copy Pre-configured Models**:
   - Use the included `factory-droid-settings.json` as a template
   - **Important**: Replace `"sk-..."` with the API key shown in DroidGravity Manager (found in **API Proxy** section)

3. **Merge Settings**:
   Add the `customModels` array to your existing Factory settings:

```json
{
  "customModels": [
    {
      "model": "gemini-3-flash",
      "id": "gemini-3-flash-0",
      "index": 0,
      "baseUrl": "http://127.0.0.1:8045/",
      "apiKey": "YOUR_DROIDGRAVITY_API_KEY_HERE",
      "displayName": "Gemini 3 Flash",
      "maxOutputTokens": 24576,
      "noImageSupport": false,
      "provider": "anthropic"
    },
    {
      "model": "claude-sonnet-4-5",
      "id": "claude-sonnet-4-5-7",
      "index": 7,
      "baseUrl": "http://127.0.0.1:8045",
      "apiKey": "YOUR_DROIDGRAVITY_API_KEY_HERE",
      "displayName": "Claude 4.5 Sonnet",
      "maxOutputTokens": 8192,
      "noImageSupport": false,
      "provider": "anthropic"
    }
  ]
}
```

**Important Configuration Notes**:
- **Both Gemini and Claude models**: Use `"provider": "anthropic"` 
- **Gemini models**: Use `"baseUrl": "http://127.0.0.1:8045/"` (with trailing slash)
- **Claude models**: Use `"baseUrl": "http://127.0.0.1:8045"` (no trailing slash)

### Step 3: Select Models in Factory Droid

1. In your Factory Droid CLI, type: `/model`
2. Select one of your custom models (e.g., "Gemini 3 Flash" or "Claude 4.5 Sonnet")
3. Start chatting! DroidGravity will automatically manage account rotation and quotas

---

## 🎯 Features

### Intelligent Account Management

- **Automatic Rotation**: Switches accounts when hitting rate limits (429/403 errors)
- **Quota Tracking**: Real-time monitoring of API quotas per account
- **OAuth Integration**: Easy account addition via Google/Anthropic OAuth flow
- **Smart Routing**: Routes requests to accounts with available quotas

### Model Support

**Google Gemini**:
- Gemini 3 Flash
- Gemini 3 Pro High
- Gemini 3 Pro Low
- Gemini 2.5 Flash
- Gemini 2.5 Flash Lite
- Gemini 2.5 Pro
- Gemini 2.5 Flash (Thinking)

**Anthropic Claude**:
- Claude 4.5 Sonnet
- Claude 4.5 Sonnet (Thinking)
- Claude 4.5 Opus (Thinking)

### Factory Droid Integration

DroidGravity automatically converts Factory Droid's proprietary request format:

```json
{
  "model": "gemini-3-flash",
  "input": [
    {
      "role": "user",
      "content": [
        {"type": "input_text", "text": "Your message"}
      ]
    }
  ]
}
```

Into standard OpenAI format:

```json
{
  "model": "gemini-3-flash",
  "messages": [
    {
      "role": "user",
      "content": "Your message"
    }
  ]
}
```

---

## 🔧 Advanced Configuration

### Model Mapping

Access **Model Mapping** in the UI to create custom route mappings:
- Map custom model names to upstream models
- Create aliases for easier access
- Route specific requests to specific account types

### Port Configuration

Default port is `8045`. To change:
1. Open DroidGravity settings
2. Navigate to **API Proxy** → **Settings**
3. Update port and restart proxy

---

## 📝 Changelog

### Version 1.1.4 (2026-02-02)

- ⚡ Synchronized rate limiting logic across Gemini, Claude, and OpenAI handlers.
- 🛠️ Fixed missing rate limit marking in Audio transcription handler.
- 📊 Enhanced proxy database with support for client IP tracking and token reasoning metadata.
- 🔄 Optimized `TokenManager` to handle concurrent account reloads more efficiently.
- 🐛 Resolved various "unresolved name" errors and improved type safety in handlers.

### Version 1.1.0 (2026-01-23)

- 🎨 Rebranded UI from "Antigravity Tools" to "DroidGravity Manager"
- 🔄 Update checker now points to this repository
- 🐛 Fixed 429 rate limit errors by replacing `gemini-2.5-flash-lite` with `gemini-2.5-flash`
- ✨ Added multi-wildcard pattern support in model routing (e.g., `claude-*-sonnet-*`)
- ✨ Added specificity-based priority for wildcard rules
- 🧪 Added unit tests for wildcard matching

### Version 1.0.0 (2026-01-12)

**Initial DroidGravity Release**:
- ✨ Added Factory Droid request format support
- ✨ Automatic conversion of `input` → `messages` format
- ✨ Support for `type: "input_text"` content blocks
- ✨ Pre-configured model settings for easy setup
- 📚 English documentation and setup guide
- 🔧 Optimized for Factory Droid CLI integration

---

## 🤝 Credits

This project is a fork of the excellent [Antigravity-Manager](https://github.com/lbjlaq/Antigravity-Manager) by [@lbjlaq](https://github.com/lbjlaq).

**Original Project**: [https://github.com/lbjlaq/Antigravity-Manager](https://github.com/lbjlaq/Antigravity-Manager)

DroidGravity builds on top of Antigravity's solid foundation to add Factory Droid compatibility.

---

## 📄 License

Inherits the license from [Antigravity-Manager](https://github.com/lbjlaq/Antigravity-Manager).

---

## 🐛 Troubleshooting

### Factory Droid shows "400 Bad Request"

- **Check API Key**: Ensure the API key in `settings.json` matches the one shown in DroidGravity Manager
- **Verify Provider**: Gemini models need `"provider": "openai"`, Claude needs `"provider": "anthropic"`
- **Check Base URL**: Gemini uses `/v1` suffix, Claude doesn't

### Models not appearing in Factory

1. Type `/model` in Factory Droid
2. If models don't show, restart Factory Droid CLI
3. Verify `customModels` array is properly formatted in `~/.factory/settings.json`

### DroidGravity not rotating accounts

- Verify accounts are properly authorized (green status in UI)
- Check that accounts have available quota
- Review logs in **Settings** → **Logs** for error details

---

**Happy coding with DroidGravity! 🚀**
