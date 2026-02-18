// Kiro Handler
use axum::{extract::State, http::StatusCode, response::IntoResponse, body::Body, http::Request};
use tracing::{debug, error, info};
use serde_json::Value;

use crate::proxy::server::AppState;
use crate::proxy::handlers::common::{determine_retry_strategy, apply_retry_strategy, should_rotate_account};

const MAX_RETRY_ATTEMPTS: usize = 3;
const KIRO_API_BASE: &str = "https://app.kiro.dev";

/// Handle Kiro API requests
/// Routes all requests to Kiro API with proper authentication
/// 
/// Supported models:
/// - Claude: claude-sonnet-4, claude-sonnet-4-5, claude-haiku-4-5, claude-opus-4-5, claude-opus-4-6, auto
/// - DeepSeek: deepseek-3 (0.25x credit multiplier, best for agentic workflows)
/// - Minimax: minimax-2-1 (0.15x credit multiplier, best for multilingual programming)
/// - Qwen: qwen3-coder-next (0.05x credit multiplier, best for coding agents with 256K context)
pub async fn handle_kiro_request(
    State(state): State<AppState>,
    req: Request<Body>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path();
    
    info!("Received Kiro request: {} {}", method, path);
    
    let token_manager = state.token_manager;
    let pool_size = token_manager.len();
    let max_attempts = MAX_RETRY_ATTEMPTS.min(pool_size).max(1);
    
    let mut last_error = String::new();
    let mut failed_accounts = std::collections::HashSet::new();
    
    // Extract body for retry
    let (parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => return Err((StatusCode::BAD_REQUEST, format!("Failed to read request body: {}", e))),
    };
    
    for attempt in 0..max_attempts {
        // Get Kiro token
        let force_rotate = attempt > 0;
        let (access_token, _refresh_token, email, account_id, _wait_ms) = match token_manager.get_token(
            "kiro",
            force_rotate,
            None, // session_id
            "kiro-default", // target_model
            if failed_accounts.is_empty() { None } else { Some(&failed_accounts) },
        ).await {
            Ok(t) => t,
            Err(e) => {
                last_error = format!("Failed to get Kiro token: {}", e);
                error!("{}", last_error);
                continue;
            }
        };
        
        info!("Using Kiro account: {} (attempt {}/{})", email, attempt + 1, max_attempts);
        
        // [NEW] Get individual proxy for this account
        let individual_proxy = token_manager.get_individual_proxy(&account_id);
        
        // Build HTTP client with individual proxy if configured
        let client = if let Some(proxy_url) = individual_proxy {
            debug!("Using individual proxy for account {}: {}", email, proxy_url);
            match reqwest::Proxy::all(&proxy_url) {
                Ok(proxy) => {
                    match reqwest::Client::builder().proxy(proxy).build() {
                        Ok(c) => c,
                        Err(e) => {
                            error!("Failed to build client with proxy {}: {}", proxy_url, e);
                            reqwest::Client::new()
                        }
                    }
                }
                Err(e) => {
                    error!("Invalid proxy URL {}: {}", proxy_url, e);
                    reqwest::Client::new()
                }
            }
        } else {
            reqwest::Client::new()
        };
        
        let kiro_url = format!("{}{}", KIRO_API_BASE, path);
        
        let mut kiro_req = client
            .request(method.clone(), &kiro_url)
            .bearer_auth(&access_token)
            .body(body_bytes.clone());
        
        // Copy headers
        for (key, value) in parts.headers.iter() {
            if key != "host" && key != "authorization" {
                kiro_req = kiro_req.header(key, value);
            }
        }
        
        // Send request
        match kiro_req.send().await {
            Ok(response) => {
                let status = response.status();
                
                if status.is_success() {
                    info!("Kiro request successful: {}", email);
                    
                    // Convert response
                    let response_bytes = match response.bytes().await {
                        Ok(b) => b,
                        Err(e) => {
                            last_error = format!("Failed to read Kiro response: {}", e);
                            continue;
                        }
                    };
                    
                    return Ok((status, response_bytes));
                } else {
                    let status_code = status.as_u16();
                    let error_text = response.text().await.unwrap_or_default();
                    
                    error!("Kiro API error {}: {}", status_code, error_text);
                    
                    // Check if should rotate
                    if should_rotate_account(status_code) {
                        failed_accounts.insert(account_id.clone());
                        last_error = format!("Kiro API error {}: {}", status_code, error_text);
                        
                        // Apply retry strategy
                        let retry_strategy = determine_retry_strategy(status_code, &error_text, false);
                        if !apply_retry_strategy(retry_strategy, attempt, max_attempts, status_code, "kiro").await {
                            return Err((StatusCode::TOO_MANY_REQUESTS, "Retry limit exceeded".to_string()));
                        }
                        continue;
                    } else {
                        // Non-retryable error
                        return Err((status, error_text));
                    }
                }
            }
            Err(e) => {
                last_error = format!("Kiro request failed: {}", e);
                error!("{}", last_error);
                
                failed_accounts.insert(account_id);
                continue;
            }
        }
    }
    
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        format!("All Kiro accounts exhausted after {} attempts. Last error: {}", max_attempts, last_error)
    ))
}

/// Handle Kiro chat completions (Anthropic Claude API compatible endpoint)
/// 
/// Supported models:
/// - Claude: claude-sonnet-4, claude-sonnet-4-5, claude-haiku-4-5, claude-opus-4-5, claude-opus-4-6, auto
/// - DeepSeek: deepseek-3 (0.25x credit multiplier)
/// - Minimax: minimax-2-1 (0.15x credit multiplier) 
/// - Qwen: qwen3-coder-next (0.05x credit multiplier)
pub async fn handle_kiro_chat_completions(
    State(state): State<AppState>,
    axum::Json(body): axum::Json<Value>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!("Received Kiro chat completion request");
    
    let token_manager = state.token_manager;
    let pool_size = token_manager.len();
    let max_attempts = MAX_RETRY_ATTEMPTS.min(pool_size).max(1);
    
    let mut last_error = String::new();
    let mut failed_accounts = std::collections::HashSet::new();
    
    let model = body.get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("claude-sonnet-4-5");  // Keep Claude as default for compatibility
    
    for attempt in 0..max_attempts {
        let force_rotate = attempt > 0;
        let (access_token, _refresh_token, email, account_id, _wait_ms) = match token_manager.get_token(
            "kiro",
            force_rotate,
            None,
            model,
            if failed_accounts.is_empty() { None } else { Some(&failed_accounts) },
        ).await {
            Ok(t) => t,
            Err(e) => {
                last_error = format!("Failed to get Kiro token: {}", e);
                error!("{}", last_error);
                continue;
            }
        };
        
        info!("Using Kiro account: {} for model {} (attempt {}/{})", email, model, attempt + 1, max_attempts);
        
        // [NEW] Get individual proxy for this account
        let individual_proxy = token_manager.get_individual_proxy(&account_id);
        
        // Build HTTP client with individual proxy if configured
        let client = if let Some(proxy_url) = individual_proxy {
            debug!("Using individual proxy for account {}: {}", email, proxy_url);
            match reqwest::Proxy::all(&proxy_url) {
                Ok(proxy) => {
                    match reqwest::Client::builder().proxy(proxy).build() {
                        Ok(c) => c,
                        Err(e) => {
                            error!("Failed to build client with proxy {}: {}", proxy_url, e);
                            reqwest::Client::new()
                        }
                    }
                }
                Err(e) => {
                    error!("Invalid proxy URL {}: {}", proxy_url, e);
                    reqwest::Client::new()
                }
            }
        } else {
            reqwest::Client::new()
        };
        
        // Send to Kiro API (Anthropic Claude format, NOT OpenAI format!)
        let response = match client
            .post(format!("{}/v1/messages", KIRO_API_BASE))  // Changed from /v1/chat/completions
            .bearer_auth(&access_token)
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                last_error = format!("Kiro request failed: {}", e);
                error!("{}", last_error);
                failed_accounts.insert(account_id);
                continue;
            }
        };
        
        let status = response.status();
        
        if status.is_success() {
            info!("Kiro chat completion successful: {}", email);
            
            let response_json: Value = match response.json().await {
                Ok(j) => j,
                Err(e) => {
                    last_error = format!("Failed to parse Kiro response: {}", e);
                    continue;
                }
            };
            
            return Ok(axum::Json(response_json));
        } else {
            let status_code = status.as_u16();
            let error_text = response.text().await.unwrap_or_default();
            
            error!("Kiro API error {}: {}", status_code, error_text);
            
            if should_rotate_account(status_code) {
                failed_accounts.insert(account_id);
                last_error = format!("Kiro API error {}: {}", status_code, error_text);
                
                let retry_strategy = determine_retry_strategy(status_code, &error_text, false);
                if !apply_retry_strategy(retry_strategy, attempt, max_attempts, status_code, "kiro").await {
                    return Err((StatusCode::TOO_MANY_REQUESTS, "Retry limit exceeded".to_string()));
                }
                continue;
            } else {
                return Err((status, error_text));
            }
        }
    }
    
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        format!("All Kiro accounts exhausted. Last error: {}", last_error)
    ))
}

/// Handle Kiro models list endpoint
pub async fn handle_kiro_models(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!("Received Kiro models list request");
    
    let token_manager = state.token_manager;
    
    // Get any available Kiro token for the request
    let (access_token, _refresh_token, _email, _account_id, _wait_ms) = match token_manager.get_token(
        "kiro",
        false,
        None,
        "claude-sonnet-4-5",
        None,
    ).await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to get Kiro token for models list: {}", e);
            return Err((StatusCode::SERVICE_UNAVAILABLE, format!("No Kiro accounts available: {}", e)));
        }
    };
    
    // Send request to Kiro API
    let client = reqwest::Client::new();
    let response = match client
        .get(format!("{}/v1/models", KIRO_API_BASE))
        .bearer_auth(&access_token)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            error!("Kiro models request failed: {}", e);
            return Err((StatusCode::SERVICE_UNAVAILABLE, format!("Failed to fetch models: {}", e)));
        }
    };
    
    let status = response.status();
    
    if status.is_success() {
        info!("Kiro models list successful");
        
        let response_json: Value = match response.json().await {
            Ok(j) => j,
            Err(e) => {
                error!("Failed to parse Kiro models response: {}", e);
                return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse response: {}", e)));
            }
        };
        
        Ok(axum::Json(response_json))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        error!("Kiro models API error {}: {}", status.as_u16(), error_text);
        Err((status, error_text))
    }
}