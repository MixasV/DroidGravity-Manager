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
    
    // Get current account
    let account = {
        let accounts = state.accounts.read().await;
        let index = *state.current_account_index.read().await;
        accounts.get(index).cloned()
    };
    
    let account = match account {
        Some(acc) => {
            tracing::info!("   Using account: {}", acc.email);
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
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": format!("Authentication failed: {}", e)}))
            ).into_response();
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
    
    match forward_to_gemini(&token, &gemini_model, &project_id, &payload).await {
        Ok(response) => {
            tracing::info!("✅ Response received from Gemini");
            response
        },
        Err(e) => {
            tracing::error!("❌ Gemini API error: {}", e);
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": format!("Upstream API error: {}", e)}))
            ).into_response()
        }
    }
}

async fn handle_anthropic_messages(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Response {
    // Convert Anthropic format to OpenAI format
    let openai_payload = json!({
        "model": payload["model"].as_str().unwrap_or("claude-sonnet-4-5"),
        "messages": payload["messages"],
        "max_tokens": payload.get("max_tokens").unwrap_or(&json!(4096)),
        "stream": payload.get("stream").unwrap_or(&json!(false)),
    });
    
    // Forward through chat completions handler
    handle_chat_completions(State(state), HeaderMap::new(), Json(openai_payload)).await
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
    match model {
        "gemini-3-flash" => "gemini-exp-1206".to_string(),
        "gemini-3-pro-high" => "gemini-exp-1206".to_string(),
        "gemini-3-pro-low" => "gemini-exp-1206".to_string(),
        "gemini-2.5-flash" => "gemini-2.0-flash-exp".to_string(),
        "gemini-2.5-flash-lite" => "gemini-2.0-flash-exp".to_string(),
        "gemini-2.5-pro" => "gemini-2.0-flash-thinking-exp-01-21".to_string(),
        "gemini-2.5-flash-thinking" => "gemini-2.0-flash-thinking-exp-01-21".to_string(),
        "claude-sonnet-4-5" => "gemini-2.0-flash-exp".to_string(),
        "claude-sonnet-4-5-thinking" => "gemini-2.0-flash-thinking-exp-01-21".to_string(),
        "claude-opus-4-5-thinking" => "gemini-2.0-flash-thinking-exp-01-21".to_string(),
        _ => "gemini-2.0-flash-exp".to_string(),
    }
}

async fn forward_to_gemini(token: &str, model: &str, project_id: &str, payload: &Value) -> Result<Response> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;
    
    // Convert OpenAI format to Gemini envelope format
    let gemini_payload = convert_to_gemini_format(payload, model, project_id)?;
    
    let url = "https://cloudcode-pa.googleapis.com/v1internal:generateContent";
    
    tracing::info!("   POST {}", url);
    tracing::info!("   Payload size: {} bytes", serde_json::to_string(&gemini_payload)?.len());
    
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Host", "cloudcode-pa.googleapis.com")
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
        anyhow::bail!("Gemini API error: {}", error_text);
    }
    
    let gemini_response: Value = response.json().await?;
    tracing::info!("   Response size: {} bytes", serde_json::to_string(&gemini_response)?.len());
    
    // Extract from envelope: response.candidates[0].content.parts[0].text
    // OR direct: candidates[0].content.parts[0].text
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
        "requestType": "text"
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
