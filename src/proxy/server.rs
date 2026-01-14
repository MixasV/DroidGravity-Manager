use anyhow::Result;
use axum::{
    extract::{Json, State},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::config::ProxyConfig;

const MAX_RETRY_ATTEMPTS: usize = 3;  // Reduced from 10 to avoid excessive retries

#[derive(Clone)]
struct AppState {
    accounts: Arc<RwLock<Vec<crate::config::account::Account>>>,
    current_account_index: Arc<RwLock<usize>>,
}

pub async fn start_server(config: ProxyConfig) -> Result<()> {
    // Load accounts
    let accounts = crate::config::account::list_accounts()?;
    
    if accounts.is_empty() {
        anyhow::bail!("No accounts configured. Add accounts first using 'drovity menu'");
    }
    
    let state = AppState {
        accounts: Arc::new(RwLock::new(accounts)),
        current_account_index: Arc::new(RwLock::new(0)),
    };
    
    let app = Router::new()
        // OpenAI compatible endpoints
        .route("/v1/chat/completions", post(handle_chat_completions))
        .route("/v1/messages", post(handle_anthropic_messages))
        .route("/v1/models", get(handle_list_models))
        .route("/healthz", get(health_check))
        .with_state(state);
    
    let addr = format!("{}:{}", config.get_bind_address(), config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("Proxy server started on http://{}", addr);
    tracing::info!("Loaded {} account(s)", crate::config::account::list_accounts()?.len());
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> Response {
    Json(json!({
        "status": "ok",
        "service": "drovity"
    }))
    .into_response()
}

async fn handle_list_models() -> Response {
    Json(json!({
        "object": "list",
        "data": [
            {
                "id": "gemini-3-flash",
                "object": "model",
                "created": 1704067200,
                "owned_by": "google"
            },
            {
                "id": "gemini-2.5-flash",
                "object": "model",
                "created": 1704067200,
                "owned_by": "google"
            },
            {
                "id": "gemini-2.5-pro",
                "object": "model",
                "created": 1704067200,
                "owned_by": "google"
            },
            {
                "id": "claude-sonnet-4-5",
                "object": "model",
                "created": 1704067200,
                "owned_by": "anthropic"
            }
        ]
    }))
    .into_response()
}

async fn handle_chat_completions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut payload): Json<Value>,
) -> Response {
    // Log incoming request
    tracing::info!("📥 Incoming chat completions request");
    tracing::info!("   Headers: {:?}", headers.get("authorization").map(|h| h.to_str().unwrap_or("invalid")));
    tracing::info!("   Model: {}", payload["model"].as_str().unwrap_or("not specified"));
    
    // Factory Droid format conversion: input -> messages
    if let Some(input) = payload.get("input").cloned() {
        tracing::info!("   Converting Factory Droid 'input' to 'messages'");
        payload["messages"] = convert_input_to_messages(input);
        payload.as_object_mut().unwrap().remove("input");
    }
    
    // Log messages
    if let Some(messages) = payload["messages"].as_array() {
        tracing::info!("   Messages count: {}", messages.len());
        for (i, msg) in messages.iter().enumerate() {
            let role = msg["role"].as_str().unwrap_or("unknown");
            let content = msg["content"].as_str().unwrap_or("");
            let preview = if content.len() > 100 {
                format!("{}...", &content[..100])
            } else {
                content.to_string()
            };
            tracing::info!("   Message {}: [{}] {}", i, role, preview);
        }
    }
    
    // Get all accounts for retry loop
    let accounts = state.accounts.read().await.clone();
    let pool_size = accounts.len();
    let max_attempts = MAX_RETRY_ATTEMPTS.min(pool_size).max(1);
    
    let mut last_error = String::new();
    let mut last_email: Option<String> = None;
    
    // Retry loop with account rotation
    for attempt in 0..max_attempts {
        let force_rotate = attempt > 0;
        
        // Select account (rotate on retry)
        let account = {
            let mut index_guard = state.current_account_index.write().await;
            if force_rotate {
                *index_guard = (*index_guard + 1) % pool_size;
                tracing::info!("🔄 Force rotation: switched to account index {}", *index_guard);
            }
            accounts.get(*index_guard).cloned()
        };
        
        let account = match account {
            Some(acc) => {
                tracing::info!("   Using account: {} (attempt {}/{})", acc.email, attempt + 1, max_attempts);
                acc
            },
            None => {
                tracing::error!("❌ No accounts available");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "No accounts available"}))
                ).into_response();
            }
        };
        
        // Check if token needs refresh
        let token = match refresh_token_if_needed(&account).await {
            Ok(t) => {
                tracing::info!("✅ Token valid/refreshed");
                t
            },
            Err(e) => {
                tracing::error!("❌ Failed to refresh token: {}", e);
                last_error = format!("Token refresh failed: {}", e);
                last_email = Some(account.email.clone());
                continue; // Try next account
            }
        };
        
        // Get project_id for this account
        let project_id = match super::project_resolver::fetch_project_id(&token).await {
            Ok(pid) => {
                tracing::info!("   Project ID: {}", pid);
                pid
            },
            Err(e) => {
                tracing::warn!("   Failed to get project_id, using mock: {}", e);
                super::project_resolver::generate_mock_project_id()
            }
        };
        
        // Forward to Gemini API
        let model = payload["model"].as_str().unwrap_or("gemini-2.5-flash");
        let gemini_model = map_model_to_gemini(model);
        
        tracing::info!("🔄 Forwarding to Gemini API");
        tracing::info!("   Requested model: {}", model);
        tracing::info!("   Gemini model: {}", gemini_model);
        
        match forward_to_gemini_stream(&token, &gemini_model, &project_id, &payload).await {
            Ok(response) => {
                tracing::info!("✅ Response received from Gemini");
                return response;
            },
            Err(e) => {
                last_error = e.to_string();
                last_email = Some(account.email.clone());
                
                tracing::error!("❌ Gemini API error (attempt {}/{}): {}", attempt + 1, max_attempts, e);
                
                // Parse error to decide retry strategy
                let error_msg = e.to_string();
                
                // Check for retryable errors
                if error_msg.contains("429") || error_msg.contains("503") || error_msg.contains("500") || error_msg.contains("RESOURCE_EXHAUSTED") {
                    tracing::warn!("   Retryable error detected, rotating to next account");
                    continue; // Retry with next account
                }
                
                // Check for quota exhausted (stop retrying)
                if error_msg.contains("QUOTA_EXHAUSTED") {
                    tracing::error!("   Quota exhausted - stopping retry");
                    return (
                        StatusCode::TOO_MANY_REQUESTS,
                        Json(json!({"error": format!("Quota exhausted: {}", error_msg)}))
                    ).into_response();
                }
                
                // Check for auth errors
                if error_msg.contains("401") || error_msg.contains("403") {
                    tracing::warn!("   Auth error, trying next account");
                    continue;
                }
                
                // Non-retryable error
                if error_msg.contains("404") || error_msg.contains("400") {
                    tracing::error!("   Non-retryable error, returning immediately");
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"error": error_msg}))
                    ).into_response();
                }
                
                // Generic error - retry
                continue;
            }
        }
    }
    
    // All attempts failed
    tracing::error!("❌ All {} attempts failed. Last error: {}", max_attempts, last_error);
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(json!({
            "error": format!("All attempts failed. Last error: {}", last_error),
            "last_account": last_email.unwrap_or_else(|| "unknown".to_string())
        }))
    ).into_response()
}

async fn handle_anthropic_messages(
    State(state): State<AppState>,
    Json(claude_payload): Json<Value>,
) -> Response {
    tracing::info!("📥 [CLAUDE/Anthropic] Incoming request");
    let model = claude_payload["model"].as_str().unwrap_or("claude-sonnet-4-5");
    tracing::info!("   Requested model: {}", model);
    
    let gemini_model = map_model_to_gemini(model);
    tracing::info!("   Mapped to Gemini model: {}", gemini_model);
    
    // Account selection and retry logic
    let accounts = state.accounts.read().await;
    let pool_size = accounts.len();
    let max_attempts = MAX_RETRY_ATTEMPTS.min(pool_size).max(1);
    
    let mut last_error = String::new();
    let mut last_email = None;
    
    for attempt in 0..max_attempts {
        let force_rotate = attempt > 0;
        
        // Select account
        let account = {
            let mut index_guard = state.current_account_index.write().await;
            if force_rotate {
                *index_guard = (*index_guard + 1) % pool_size;
                tracing::info!("🔄 Rotated to account index {}", *index_guard);
            }
            accounts.get(*index_guard).cloned()
        };
        
        let account = match account {
            Some(acc) => acc,
            None => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({"error": "No accounts available"}))
                ).into_response();
            }
        };
        
        tracing::info!("   Account: {} (attempt {}/{})", account.email, attempt + 1, max_attempts);
        last_email = Some(account.email.clone());
        
        // Get token
        let token = match refresh_token_if_needed(&account).await {
            Ok(t) => {
                tracing::info!("✅ Token OK");
                t
            },
            Err(e) => {
                last_error = e.to_string();
                tracing::error!("❌ Token error: {}", e);
                continue;
            }
        };
        
        // Get project ID
        let project_id = match super::project_resolver::fetch_project_id(&token).await {
            Ok(pid) => {
                tracing::info!("   Project: {}", pid);
                pid
            },
            Err(e) => {
                tracing::warn!("   Project fetch failed, using mock: {}", e);
                super::project_resolver::generate_mock_project_id()
            }
        };
        
        
        // DEBUG: Log incoming payload BEFORE parsing
        tracing::debug!("🔍 RAW Claude payload: {}", serde_json::to_string_pretty(&claude_payload).unwrap_or_else(|_| "Failed to serialize".to_string()));
        
        // FULL CONVERSION: Parse Claude request into typed structure
        let claude_request: super::claude::models::ClaudeRequest = match serde_json::from_value(claude_payload.clone()) {
            Ok(r) => r,
            Err(e) => {
                last_error = format!("Failed to parse Claude request: {}", e);
                tracing::error!("❌ Deserialization error: {}", last_error);
                tracing::error!("   Payload was: {}", serde_json::to_string_pretty(&claude_payload).unwrap_or_else(|_| "N/A".to_string()));
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": last_error}))
                ).into_response();
            }
        };
        
        // Convert using FULL DroidGravity-Manager logic
        let gemini_payload = match super::claude::transform_claude_request_in(&claude_request, &project_id) {
            Ok(p) => p,
            Err(e) => {
                last_error = format!("Claude→Gemini conversion error: {}", e);
                tracing::error!("❌ {}", last_error);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": last_error}))
                ).into_response();
            }
        };
        
        
        // Forward to Gemini (DIRECT - payload already in Gemini format!)
        match send_gemini_payload_direct(&token, &gemini_payload).await {
            Ok(gemini_axum_response) => {
                // Extract JSON from Axum Response
                let (_parts, body) = gemini_axum_response.into_parts();
                let bytes = match axum::body::to_bytes(body, usize::MAX).await {
                    Ok(b) => b,
                    Err(e) => {
                        last_error = format!("Body read error: {}", e);
                        tracing::error!("❌ {}", last_error);
                        continue;
                    }
                };
                
                let gemini_json: Value = match serde_json::from_slice(&bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        last_error = format!("JSON parse error: {}", e);
                        tracing::error!("❌ {}", last_error);
                        continue;
                    }
                };
                
                // Parse Gemini response into typed structure
                let gemini_response: super::claude::models::GeminiResponse = match serde_json::from_value(gemini_json) {
                    Ok(r) => r,
                    Err(e) => {
                        last_error = format!("Failed to parse Gemini response: {}", e);
                        tracing::error!("❌ {}", last_error);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"error": last_error}))
                        ).into_response();
                    }
                };
                
                // Convert using FULL DroidGravity-Manager logic
                match super::claude::transform_response(&gemini_response) {
                    Ok(claude_response) => {
                        tracing::info!("✅ Gemini→Claude conversion OK");
                        let response_json = match serde_json::to_value(&claude_response) {
                            Ok(v) => v,
                            Err(e) => {
                                tracing::error!("Failed to serialize Claude response: {}", e);
                                return (
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    Json(json!({"error": "Response serialization error"}))
                                ).into_response();
                            }
                        };
                        
                        return (
                            StatusCode::OK,
                            Json(response_json)
                        ).into_response();
                    },
                    Err(e) => {
                        last_error = e;
                        tracing::error!("❌ Gemini→Claude error: {}", last_error);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({"error": last_error}))
                        ).into_response();
                    }
                }
            },
            Err(e) => {
                last_error = e.to_string();
                tracing::error!("❌ Gemini API error (attempt {}/{}): {}", attempt + 1, max_attempts, e);
                
                let error_msg = e.to_string();
                
                // Retryable: 429, 500, 503
                if error_msg.contains("429") || error_msg.contains("503") || error_msg.contains("500") || error_msg.contains("RESOURCE_EXHAUSTED") {
                    tracing::warn!("   Retryable error → next account");
                    continue;
                }
                
                // Non-retryable: 400, 404
                if error_msg.contains("404") || error_msg.contains("400") {
                    tracing::error!("   Non-retryable error");
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({"error": error_msg}))
                    ).into_response();
                }
                
                continue;
            }
        }
    }
    
    // All attempts failed
    tracing::error!("❌ All {} attempts failed. Last: {}", max_attempts, last_error);
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(json!({
            "error": format!("All attempts failed. Last: {}", last_error),
            "last_account": last_email.unwrap_or_else(|| "unknown".to_string())
        }))
    ).into_response()
}

fn convert_input_to_messages(input: Value) -> Value {
    if let Some(array) = input.as_array() {
        let messages: Vec<Value> = array.iter().map(|msg| {
            let role = msg["role"].as_str().unwrap_or("user");
            let content = if let Some(content_array) = msg["content"].as_array() {
                // Extract text from content blocks
                let text_parts: Vec<String> = content_array.iter()
                    .filter_map(|block| {
                        if block["type"] == "input_text" {
                            block["text"].as_str().map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                json!(text_parts.join("\n"))
            } else {
                msg["content"].clone()
            };
            
            json!({
                "role": role,
                "content": content
            })
        }).collect();
        
        json!(messages)
    } else {
        json!([])
    }
}

async fn refresh_token_if_needed(account: &crate::config::account::Account) -> Result<String> {
    use chrono::Utc;
    
    let now = Utc::now().timestamp();
    
    // If token expires in less than 5 minutes, refresh
    if account.token.expiry_timestamp < now + 300 {
        let token_response = crate::oauth::refresh_access_token(&account.token.refresh_token).await?;
        Ok(token_response.access_token)
    } else {
        Ok(account.token.access_token.clone())
    }
}

fn map_model_to_gemini(model: &str) -> String {
    // EXACT COPY from DroidGravity-Manager/src/proxy/common/model_mapping.rs
    match model {
        // Claude models - PASS THROUGH AS-IS (Antigravity API supports them directly!)
        "claude-opus-4-5-thinking" => "claude-opus-4-5-thinking".to_string(),
        "claude-sonnet-4-5" => "claude-sonnet-4-5".to_string(),
        "claude-sonnet-4-5-thinking" => "claude-sonnet-4-5-thinking".to_string(),
        
        // Claude aliases
        "claude-sonnet-4-5-20250929" => "claude-sonnet-4-5-thinking".to_string(),
        "claude-3-5-sonnet-20241022" => "claude-sonnet-4-5".to_string(),
        "claude-3-5-sonnet-20240620" => "claude-sonnet-4-5".to_string(),
        "claude-opus-4" => "claude-opus-4-5-thinking".to_string(),
        "claude-opus-4-5-20251101" => "claude-opus-4-5-thinking".to_string(),
        "claude-haiku-4" => "claude-sonnet-4-5".to_string(),
        "claude-3-haiku-20240307" => "claude-sonnet-4-5".to_string(),
        "claude-haiku-4-5-20251001" => "claude-sonnet-4-5".to_string(),
        
        // OpenAI → Gemini
        "gpt-4" | "gpt-4-turbo" | "gpt-4-turbo-preview" | "gpt-4-0125-preview" 
        | "gpt-4-1106-preview" | "gpt-4-0613" | "gpt-4o" | "gpt-4o-2024-05-13" 
        | "gpt-4o-2024-08-06" => "gemini-2.5-pro".to_string(),
        
        "gpt-4o-mini" | "gpt-4o-mini-2024-07-18" | "gpt-3.5-turbo" | "gpt-3.5-turbo-16k" 
        | "gpt-3.5-turbo-0125" | "gpt-3.5-turbo-1106" | "gpt-3.5-turbo-0613" 
        => "gemini-2.5-flash".to_string(),
        
        // Gemini pass-through
        "gemini-2.5-flash-lite" => "gemini-2.5-flash-lite".to_string(),
        "gemini-2.5-flash-thinking" => "gemini-2.5-flash-thinking".to_string(),
        "gemini-3-pro-low" => "gemini-3-pro-low".to_string(),
        "gemini-3-pro-high" => "gemini-3-pro-high".to_string(),
        "gemini-3-pro-preview" => "gemini-3-pro-preview".to_string(),
        "gemini-3-pro" => "gemini-3-pro".to_string(),
        "gemini-2.5-flash" => "gemini-2.5-flash".to_string(),
        "gemini-3-flash" => "gemini-3-flash".to_string(),
        "gemini-3-pro-image" => "gemini-3-pro-image".to_string(),
        "gemini-2.0-flash-exp" => "gemini-2.0-flash-exp".to_string(),
        "gemini-2.5-pro" => "gemini-2.5-pro".to_string(),
        
        // Pass-through for gemini-* prefix or *-thinking suffix
        model if model.starts_with("gemini-") || model.contains("thinking") => model.to_string(),
        
        // Default fallback
        _ => "claude-sonnet-4-5".to_string(),
    }
}

// NEW: Direct Gemini payload sender (no conversion needed)
// Used when payload is already in Gemini format (from transform_claude_request_in)
async fn send_gemini_payload_direct(token: &str, gemini_payload: &Value) -> Result<Response> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;
    
    let url = "https://cloudcode-pa.googleapis.com/v1internal:streamGenerateContent?alt=sse";
    
    tracing::debug!("   POST {} (STREAM - direct)", url);
    tracing::debug!("   📤 Gemini payload: {}", serde_json::to_string_pretty(gemini_payload)?);
    
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "antigravity/1.11.9 windows/amd64")
        .header("Content-Type", "application/json")
        .json(gemini_payload)
        .send()
        .await?;
    
    let status = response.status();
    tracing::info!("   Response status: {}", status);
    
    if !status.is_success() {
        let error_text = response.text().await?;
        tracing::error!("❌ Gemini API error response: {}", error_text);
        anyhow::bail!("Gemini API error {}: {}", status, error_text);
    }
    
    // Collect SSE stream into single response
    let stream_body = response.text().await?;
    tracing::debug!("   Stream received: {} bytes", stream_body.len());
    
    // Parse SSE and collect final response
    let gemini_response = parse_sse_stream(&stream_body)?;
    
    Ok((StatusCode::OK, Json(gemini_response)).into_response())
}

// Use STREAM for better quota (like DroidGravity-Manager)
async fn forward_to_gemini_stream(token: &str, model: &str, project_id: &str, payload: &Value) -> Result<Response> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;
    
    // Convert OpenAI format to Gemini envelope format  
    let gemini_payload = convert_to_gemini_format(payload, model, project_id)?;
    
    // Use streamGenerateContent for better quota
    let url = "https://cloudcode-pa.googleapis.com/v1internal:streamGenerateContent?alt=sse";
    
    tracing::info!("   POST {} (STREAM)", url);
    let payload_string = serde_json::to_string(&gemini_payload)?;
    tracing::info!("   Payload size: {} bytes", payload_string.len());
    
    // CRITICAL: Log full payload for troubleshooting
    tracing::info!("   📤 SENDING PAYLOAD: {}", serde_json::to_string_pretty(&gemini_payload)?);
    
    
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "antigravity/1.11.9 windows/amd64")
        .header("Content-Type", "application/json")
        .json(&gemini_payload)
        .send()
        .await?;
    
    let status = response.status();
    tracing::info!("   Response status: {}", status);
    
    if !status.is_success() {
        let error_text = response.text().await?;
        tracing::error!("❌ Gemini API error response: {}", error_text);
        anyhow::bail!("Gemini API error {}: {}", status, error_text);
    }
    
    // Collect SSE stream into single response
    let stream_body = response.text().await?;
    tracing::info!("   Stream received: {} bytes", stream_body.len());
    
    // Parse SSE and collect final response
    let gemini_response = parse_sse_stream(&stream_body)?;
    
    // Extract from envelope
    let response_data = gemini_response.get("response").unwrap_or(&gemini_response);
    
    // Log first part of content
    if let Some(text) = response_data["candidates"][0]["content"]["parts"][0]["text"].as_str() {
        let preview = if text.len() > 200 {
            format!("{}...", &text[..200])
        } else {
            text.to_string()
        };
        tracing::info!("   Content preview: {}", preview);
    }
    
    // Convert Gemini response back to OpenAI format
    let openai_response = convert_gemini_to_openai_response(response_data, model)?;
    
    Ok(Json(openai_response).into_response())
}

fn parse_sse_stream(stream_body: &str) -> Result<Value> {
    let mut accumulated_text = String::new();
    
    // Parse SSE events
    for line in stream_body.lines() {
        if line.starts_with("data: ") {
            let data = &line[6..]; // Remove "data: " prefix
            if data.trim() == "[DONE]" {
                continue;
            }
            
            if let Ok(event) = serde_json::from_str::<Value>(data) {
                // Extract text from streaming chunks
                if let Some(text) = event["response"]["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                    accumulated_text.push_str(text);
                } else if let Some(text) = event["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                    accumulated_text.push_str(text);
                }
            }
        }
    }
    
    // Build final response structure
    Ok(json!({
        "candidates": [{
            "content": {
                "parts": [{
                    "text": accumulated_text
                }]
            }
        }]
    }))
}

fn convert_to_gemini_format(payload: &Value, model: &str, project_id: &str) -> Result<Value> {
    let messages = payload["messages"].as_array()
        .ok_or_else(|| anyhow::anyhow!("Missing messages field"))?;
    
    let contents: Vec<Value> = messages.iter().filter(|msg| {
        msg["role"].as_str().unwrap_or("user") != "system"
    }).map(|msg| {
        let role = match msg["role"].as_str().unwrap_or("user") {
            "assistant" => "model",
            role => role,
        };
        
        json!({
            "role": role,
            "parts": [{
                "text": msg["content"].as_str().unwrap_or("")
            }]
        })
    }).collect();
    
    // Extract system message for systemInstruction
    let system_text: Vec<String> = messages.iter()
        .filter(|msg| msg["role"].as_str() == Some("system"))
        .filter_map(|msg| msg["content"].as_str().map(|s| s.to_string()))
        .collect();
    
    let inner_request = json!({
        "contents": contents,
        "generationConfig": {
            "maxOutputTokens": payload.get("max_tokens").unwrap_or(&json!(8192)),
            "temperature": payload.get("temperature").unwrap_or(&json!(1.0)),
        },
        "systemInstruction": if !system_text.is_empty() {
            json!({
                "role": "user",
                "parts": system_text.iter().map(|s| json!({"text": s})).collect::<Vec<_>>()
            })
        } else {
            json!(null)
        },
        "safetySettings": [
            { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "OFF" },
            { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "OFF" },
            { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "OFF" },
            { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "OFF" },
            { "category": "HARM_CATEGORY_CIVIC_INTEGRITY", "threshold": "OFF" }
        ]
    });
    
    // Wrap in v1internal envelope format (like DroidGravity-Manager)
    Ok(json!({
        "project": project_id,
        "requestId": format!("drovity-{}", uuid::Uuid::new_v4()),
        "request": inner_request,
        "model": model,
        "userAgent": "antigravity",
        "requestType": "agent"  // CRITICAL: Must be "agent" not "text" for proper quota!
    }))
}

fn convert_gemini_to_openai_response(gemini_response: &Value, model: &str) -> Result<Value> {
    let text = gemini_response["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("");
    
    Ok(json!({
        "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": model,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": text
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0
        }
    }))
}
