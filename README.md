# DroidGravity Manager 🚀

> [!CAUTION]
> **DO NOT SYNC WITH UPSTREAM (lbjlaq/Antigravity-Manager)!**
> This is a separate standalone project. Syncing with upstream will overwrite Droid-specific changes and localized branding.

**Version 2.0.0**

A fork of [Antigravity-Manager](https://github.com/lbjlaq/Antigravity-Manager) with **Factory Droid** support for seamless integration with Google Gemini, Anthropic Claude, and **Kiro** models.

## 🌟 What's New in v2.0.0 - Kiro Integration

- **🚀 Kiro Support**: Full integration with Kiro AI platform including OAuth authentication and all supported models
- **🎯 All Kiro Models**: Support for Claude (Sonnet, Haiku, Opus) + Open Weight models (DeepSeek 3, Minimax 2.1, Qwen3 Coder Next)
- **🔐 OAuth Authentication**: Seamless Kiro account addition via AWS Cognito with PKCE security
- **🌐 Individual Proxies**: Each Kiro account can use its own HTTP/SOCKS5 proxy for enhanced privacy
- **💰 Credit Optimization**: Open Weight models with reduced credit costs (DeepSeek: 0.25x, Minimax: 0.15x, Qwen: 0.05x)
- **🔄 Smart Routing**: Automatic account rotation and provider selection (Gemini/Kiro)
- **📊 Enhanced Monitoring**: Full support for Kiro API monitoring and quota tracking

### Supported Kiro Models

**Claude Models (Premium)**:
- `auto` - Smart Router (recommended)
- `claude-sonnet-4` / `claude-sonnet-4-5` - Balanced quality and speed
- `claude-haiku-4-5` - Fast and economical
- `claude-opus-4-5` / `claude-opus-4-6` - Most powerful for coding

**Open Weight Models (Cost-Effective)**:
- `deepseek-3` (0.25x credits) - Best for agentic workflows and code generation
- `minimax-2-1` (0.15x credits) - Best for multilingual programming and UI generation
- `qwen3-coder-next` (0.05x credits) - Best for coding agents with 256K context window

## 🌟 What's New in v1.2.9

- **Claude Opus 4.6 Support**: Full compatibility with the latest `claude-opus-4-6-thinking` model. Automatic redirection from legacy Opus 4.5/4.0 IDs.
- **Improved Rotation Logic**: Fixed a bug where **404 NOT_FOUND** (Google Resource missing) errors didn't trigger account rotation. Now it instantly switches to the next working account.
- **Progressive Context Compression**: Implemented 3-layer progressive compression (Tool trimming, Thinking purification, XML summary) to handle massive contexts up to 2M tokens.
- **Interrupted Session Recovery**: Improved "Heal Session" logic that automatically closes broken tool loops. You can now continue your chat even if the previous response was cut off.
- **Fake 200 Error Detection**: The proxy now detects model deprecation notices (e.g., "switch to 4.6") hidden inside successful responses and triggers automatic account rotation.
- **Restored Prompt Caching**: Re-enabled Anthropic's Prompt Caching (`cache_control`), reducing token costs by up to 90% for long conversations.
- **Robust Error Handling**:
  - Fixed **Error 400** caused by incorrect `thoughtSignature` nesting in Gemini API.
  - Added account rotation for **404 NOT_FOUND** (Google Resource missing) errors.
  - Eliminated "undefined is not an object" UI crashes during token usage reporting.
- **Enhanced Monitoring**: Restored Request/Response payload visibility in the Monitor Dashboard with safe truncation for large streams.

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
   - **For Gemini**: Go to **Accounts** → **Add Account** → Select **Gemini** → **OAuth** → Authorize with your Google account
   - **For Claude**: Go to **Accounts** → **Add Account** → Select **Gemini** → **OAuth** → Authorize with your Anthropic account  
   - **For Kiro**: Go to **Accounts** → **Add Account** → Select **Kiro** → **OAuth** → Authorize with your Kiro account via AWS Cognito
3. **Start the Proxy**: Navigate to **API Proxy** and enable the server (default port: `8045`)

### Step 2: Configure Factory Droid

1. **Locate Factory Settings**:
   - Open `~/.factory/settings.json` (Linux/macOS) or `C:\Users\YOUR_USERNAME\.factory\settings.json` (Windows)

2. **Copy Pre-configured Models**:
   - Use the included `factory-droid-settings.json` as a template
   - **Important**: Replace `"sk-..."` with the API key shown in DroidGravity Manager (found in **API Proxy** section)

3. **Merge Settings**:
   Add the `customModels` array to your existing Factory settings. **Note**: All model IDs must use the `custom:` prefix:

```json
{
  "customModels": [
    {
      "model": "gemini-3-flash",
      "id": "custom:Gemini-3-Flash-0",
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
      "id": "custom:Claude-4.5-Sonnet-7",
      "index": 7,
      "baseUrl": "http://127.0.0.1:8045",
      "apiKey": "YOUR_DROIDGRAVITY_API_KEY_HERE",
      "displayName": "Claude 4.5 Sonnet",
      "maxOutputTokens": 8192,
      "noImageSupport": false,
      "provider": "anthropic"
    },
    {
      "model": "auto",
      "id": "custom:Kiro-Auto-10",
      "index": 10,
      "baseUrl": "http://127.0.0.1:8045",
      "apiKey": "YOUR_DROIDGRAVITY_API_KEY_HERE",
      "displayName": "Kiro Auto (Smart Router)",
      "maxOutputTokens": 32000,
      "noImageSupport": false,
      "provider": "anthropic"
    },
    {
      "model": "deepseek-3",
      "id": "custom:Kiro-DeepSeek-3-16",
      "index": 16,
      "baseUrl": "http://127.0.0.1:8045",
      "apiKey": "YOUR_DROIDGRAVITY_API_KEY_HERE",
      "displayName": "Kiro DeepSeek 3 (0.25x credits, Agentic)",
      "maxOutputTokens": 32000,
      "noImageSupport": false,
      "provider": "anthropic"
    }
  ]
}
```

**Important Configuration Notes**:
- **All models**: Use `"provider": "anthropic"` for compatibility
- **Model IDs**: Must include `custom:` prefix (e.g., `custom:Kiro-Auto-10`)
- **Gemini models**: Use `"baseUrl": "http://127.0.0.1:8045/"` (with trailing slash)
- **Claude/Kiro models**: Use `"baseUrl": "http://127.0.0.1:8045"` (no trailing slash)

### Step 3: Select Models in Factory Droid

1. In your Factory Droid CLI, type: `/model`
2. Select one of your custom models:
   - **Gemini**: "Gemini 3 Flash", "Gemini 2.5 Pro", etc.
   - **Claude**: "Claude 4.5 Sonnet", "Claude 4.6 Opus", etc.
   - **Kiro**: "Kiro Auto", "Kiro DeepSeek 3", "Kiro Qwen3 Coder Next", etc.
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
- Claude 4.6 Opus (Thinking) [Recommended]
- Claude 3.5 Sonnet & Haiku (Via Legacy IDs)

**Kiro AI Platform**:
- Auto (Smart Router) [Recommended]
- Claude Sonnet 4.0/4.5/4.6 (Latest: 4.6 with 1.3x credits)
- Claude Haiku 4.5 (Fast)
- Claude Opus 4.5/4.6 (Powerful)
- DeepSeek 3 (0.25x credits, Agentic)
- Minimax 2.1 (0.15x credits, Multilingual)
- Qwen3 Coder Next (0.05x credits, 256K context)

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

### Version 2.0.0 (2026-02-18) - Kiro Integration

- 🚀 **Kiro Platform Support**: Full integration with Kiro AI platform
- 🔐 **OAuth Authentication**: AWS Cognito-based authentication with PKCE security
- 🎯 **All Kiro Models**: Support for Claude models + Open Weight models (DeepSeek, Minimax, Qwen)
- 🌐 **Individual Proxies**: Per-account HTTP/SOCKS5 proxy support for enhanced privacy
- 💰 **Cost Optimization**: Open Weight models with reduced credit multipliers
- 🔄 **Provider Selection**: UI support for choosing between Gemini and Kiro providers
- 📊 **Enhanced Monitoring**: Full Kiro API monitoring and quota tracking
- ⚡ **Smart Routing**: Automatic account rotation across multiple providers
- 🛠️ **Backend Architecture**: Complete OAuth module, token management, and proxy handlers
- 📋 **Factory Integration**: Pre-configured Kiro models in factory-droid-settings.json

### Version 1.2.9 (2026-02-11)

- ✨ **Account Rotation Fix**: Added **404 NOT_FOUND** error detection to `should_rotate_account` logic.
- 🔄 **Opus 4.6 Migration**: Full support for native `claude-opus-4-6-thinking`.
- ✨ **UI Updates**: All references updated from Claude 4.5 to **Claude 4.6 Opus**.
- 🩹 **Heal Session Logic**: Enhanced recovery for interrupted thinking/tool streams.
- ⚡ **Prompt Caching**: Restored `cache_control` integration.
- 📊 **Monitor Dashboard**: Restored Payloads and fixed token usage UI crashes.

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
