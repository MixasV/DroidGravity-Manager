use anyhow::{Result, Context};
use std::path::PathBuf;

/// Auto-configure Factory Droid settings.json
pub async fn auto_configure(settings_path: &PathBuf) -> Result<()> {
    // Read existing settings
    let content = std::fs::read_to_string(settings_path)
        .context("Failed to read Factory settings")?;
    
    let mut settings: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse Factory settings")?;
    
    // Get current API key
    let config = crate::config::load_config()?;
    let api_key = &config.proxy.api_key;
    let base_url = format!("http://127.0.0.1:{}", config.proxy.port);
    
    // Generate models config
    let models = generate_models_array(api_key, &base_url)?;
    
    // Merge with existing customModels
    if let Some(existing_models) = settings.get_mut("customModels") {
        if let Some(existing_array) = existing_models.as_array_mut() {
            // Remove old drovity models (cleanup)
            existing_array.retain(|m| {
                !m["id"].as_str().unwrap_or("").starts_with("custom:Gemini-") &&
                !m["id"].as_str().unwrap_or("").starts_with("custom:Claude-")
            });
            
            // Add new models
            for model in models.as_array().unwrap() {
                existing_array.push(model.clone());
            }
        }
    } else {
        // No existing customModels, create new
        settings["customModels"] = models;
    }
    
    // Write back
    let updated_content = serde_json::to_string_pretty(&settings)?;
    std::fs::write(settings_path, updated_content)
        .context("Failed to write Factory settings")?;
    
    Ok(())
}

/// Generate JSON config for manual setup
pub fn generate_config_json() -> Result<String> {
    let config = crate::config::load_config()?;
    let api_key = &config.proxy.api_key;
    let base_url = format!("http://127.0.0.1:{}", config.proxy.port);
    
    let models = generate_models_array(api_key, &base_url)?;
    let json = serde_json::to_string_pretty(&models)?;
    
    Ok(json)
}

fn generate_models_array(api_key: &str, base_url: &str) -> Result<serde_json::Value> {
    Ok(serde_json::json!([
        {
            "model": "gemini-3-flash",
            "id": "custom:Gemini-3-Flash-0",
            "index": 0,
            "baseUrl": format!("{}/", base_url),
            "apiKey": api_key,
            "displayName": "Gemini 3 Flash",
            "maxOutputTokens": 24576,
            "noImageSupport": false,
            "provider": "anthropic"
        },
        {
            "model": "gemini-3-pro-high",
            "id": "custom:Gemini-3-Pro-High-1",
            "index": 1,
            "baseUrl": format!("{}/", base_url),
            "apiKey": api_key,
            "displayName": "Gemini 3 Pro High",
            "maxOutputTokens": 32768,
            "noImageSupport": false,
            "provider": "anthropic"
        },
        {
            "model": "gemini-3-pro-low",
            "id": "custom:Gemini-3-Pro-Low-2",
            "index": 2,
            "baseUrl": format!("{}/", base_url),
            "apiKey": api_key,
            "displayName": "Gemini 3 Pro Low",
            "maxOutputTokens": 32768,
            "noImageSupport": false,
            "provider": "anthropic"
        },
        {
            "model": "gemini-2.5-flash",
            "id": "custom:Gemini-2.5-Flash-3",
            "index": 3,
            "baseUrl": format!("{}/", base_url),
            "apiKey": api_key,
            "displayName": "Gemini 2.5 Flash",
            "maxOutputTokens": 24576,
            "noImageSupport": false,
            "provider": "anthropic"
        },
        {
            "model": "gemini-2.5-flash-lite",
            "id": "custom:Gemini-2.5-Flash-Lite-4",
            "index": 4,
            "baseUrl": format!("{}/", base_url),
            "apiKey": api_key,
            "displayName": "Gemini 2.5 Flash Lite",
            "maxOutputTokens": 24576,
            "noImageSupport": false,
            "provider": "anthropic"
        },
        {
            "model": "gemini-2.5-pro",
            "id": "custom:Gemini-2.5-Pro-5",
            "index": 5,
            "baseUrl": format!("{}/", base_url),
            "apiKey": api_key,
            "displayName": "Gemini 2.5 Pro",
            "maxOutputTokens": 32768,
            "noImageSupport": false,
            "provider": "anthropic"
        },
        {
            "model": "gemini-2.5-flash-thinking",
            "id": "custom:Gemini-2.5-Flash-(Thinking)-6",
            "index": 6,
            "baseUrl": format!("{}/", base_url),
            "apiKey": api_key,
            "displayName": "Gemini 2.5 Flash (Thinking)",
            "maxOutputTokens": 24576,
            "noImageSupport": false,
            "provider": "anthropic"
        },
        {
            "model": "claude-sonnet-4-5",
            "id": "custom:Claude-4.5-Sonnet-7",
            "index": 7,
            "baseUrl": base_url,
            "apiKey": api_key,
            "displayName": "Claude 4.5 Sonnet",
            "maxOutputTokens": 8192,
            "noImageSupport": false,
            "provider": "anthropic"
        },
        {
            "model": "claude-sonnet-4-5-thinking",
            "id": "custom:Claude-4.5-Sonnet-(Thinking)-8",
            "index": 8,
            "baseUrl": base_url,
            "apiKey": api_key,
            "displayName": "Claude 4.5 Sonnet (Thinking)",
            "maxOutputTokens": 16384,
            "noImageSupport": false,
            "provider": "anthropic"
        },
        {
            "model": "claude-opus-4-5-thinking",
            "id": "custom:Claude-4.5-Opus-(Thinking)-9",
            "index": 9,
            "baseUrl": base_url,
            "apiKey": api_key,
            "displayName": "Claude 4.5 Opus (Thinking)",
            "maxOutputTokens": 16384,
            "noImageSupport": false,
            "provider": "anthropic"
        }
    ]))
}
