// Kiro Handler - AWS CodeWhisperer Streaming API
// Конвертирует Anthropic Claude API → Kiro API

use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bytes::Bytes;
use serde_json::json;
use tracing::{debug, error, info};

use crate::proxy::{
    handlers::common::{apply_retry_strategy, determine_retry_strategy, should_rotate_account},
    mappers::{
        claude::models::ClaudeRequest,
        kiro::{
            convert_claude_to_kiro, convert_event_to_claude_chunk, convert_kiro_to_claude,
            parse_event_stream, collect_stream_to_text,
        },
    },
    server::AppState,
};

const MAX_RETRY_ATTEMPTS: usize = 3;
const KIRO_API_ENDPOINT: &str = "https://q.us-east-1.amazonaws.com/generateAssistantResponse";
const MACHINE_ID: &str = "82f653a1e60a7c25cf6152578375f4e011db8e8015d712b91f2cf3ec8e9b8ed8";

/// Handle Kiro chat completions (Anthropic Claude API compatible)
/// 
/// Конвертирует Anthropic формат в Kiro формат и обратно
/// 
/// Supported models:
/// - auto - Smart router (рекомендуется)
/// - claude-sonnet-4, claude-sonnet-4-5 - Баланс качества и скорости
/// - claude-haiku-4-5 - Быстрая и дешевая
/// - claude-opus-4-5, claude-opus-4-6 - Самая мощная
/// - deepseek-3 - 0.25x credits, лучше для agentic workflows
/// - minimax-2-1 - 0.15x credits, лучше для multilingual programming
/// - qwen3-coder-next - 0.05x credits, 256K context, лучше для coding agents
pub async fn handle_kiro_messages(
    State(state): State<AppState>,
    Json(claude_req): Json<ClaudeRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!("Received Kiro messages request (model: {})", claude_req.model);
    
    let token_manager = state.token_manager;
    let pool_size = token_manager.len();
    let max_attempts = MAX_RETRY_ATTEMPTS.min(pool_size).max(1);
    
    let mut last_error = String::new();
    let mut failed_accounts = std::collections::HashSet::new();
    let mut conversation_id: Option<String> = None;
    
    // Определяем streaming mode
    let is_stream = claude_req.stream;
    
    for attempt in 0..max_attempts {
        let force_rotate = attempt > 0;
        
        // Получаем Kiro токен
        let (access_token, _refresh_token, email, account_id, _wait_ms) =
            match token_manager
                .get_token(
                    "kiro",
                    force_rotate,
                    None,
                    &claude_req.model,
                    if failed_accounts.is_empty() {
                        None
                    } else {
                        Some(&failed_accounts)
                    },
                )
                .await
            {
                Ok(t) => t,
                Err(e) => {
                    last_error = format!("Failed to get Kiro token: {}", e);
                    error!("{}", last_error);
                    continue;
                }
            };
        
        info!(
            "Using Kiro account: {} for model {} (attempt {}/{})",
            email,
            claude_req.model,
            attempt + 1,
            max_attempts
        );
        
        // Получаем profile ARN из token manager
        let profile_arn = match token_manager.get_kiro_profile_arn(&account_id).await {
            Ok(arn) => arn,
            Err(e) => {
                last_error = format!("Failed to get profile ARN: {}", e);
                error!("{}", last_error);
                continue;
            }
        };
        
        // Конвертируем Claude request в Kiro format
        let kiro_req = convert_claude_to_kiro(&claude_req, &profile_arn, conversation_id.clone());
        
        // Сохраняем conversation_id для следующих попыток
        conversation_id = Some(kiro_req.conversation_state.conversation_id.clone());
        
        // Получаем individual proxy если настроен
        let individual_proxy = token_manager.get_individual_proxy(&account_id);
        
        // Строим HTTP client с proxy
        let client = if let Some(proxy_url) = individual_proxy {
            debug!("Using individual proxy for account {}: {}", email, proxy_url);
            match reqwest::Proxy::all(&proxy_url) {
                Ok(proxy) => match reqwest::Client::builder().proxy(proxy).build() {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to build client with proxy {}: {}", proxy_url, e);
                        reqwest::Client::new()
                    }
                },
                Err(e) => {
                    error!("Invalid proxy URL {}: {}", proxy_url, e);
                    reqwest::Client::new()
                }
            }
        } else {
            reqwest::Client::new()
        };
        
        // Генерируем invocation_id
        let invocation_id = uuid::Uuid::new_v4().to_string();
        
        // Строим headers
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());
        headers.insert("x-amzn-codewhisperer-optout", "true".parse().unwrap());
        headers.insert(
            "x-amzn-kiro-agent-mode",
            "intent-classification".parse().unwrap(),
        );
        headers.insert(
            "x-amz-user-agent",
            format!("aws-sdk-js/1.0.27 KiroIDE-0.9.47-{}", MACHINE_ID)
                .parse()
                .unwrap(),
        );
        headers.insert(
            "user-agent",
            format!(
                "aws-sdk-js/1.0.27 ua/2.1 os/win32#10.0.26100 lang/js md/nodejs#22.21.1 api/codewhispererstreaming#1.0.27 m/E KiroIDE-0.9.47-{}",
                MACHINE_ID
            )
            .parse()
            .unwrap(),
        );
        headers.insert("host", "q.us-east-1.amazonaws.com".parse().unwrap());
        headers.insert("amz-sdk-invocation-id", invocation_id.parse().unwrap());
        headers.insert("amz-sdk-request", "attempt=1; max=3".parse().unwrap());
        headers.insert(
            "Authorization",
            format!("Bearer {}", access_token).parse().unwrap(),
        );
        
        // Отправляем запрос
        let response = match client
            .post(KIRO_API_ENDPOINT)
            .headers(headers)
            .json(&kiro_req)
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
            info!("Kiro request successful: {}", email);
            
            // [AUTO-CONVERSION] Всегда собираем stream и парсим команды
            // Это нужно для правильной конвертации команд в tool_use blocks
            if is_stream {
                // Client wants streaming - собираем stream, парсим команды, и отправляем как streaming
                return handle_streaming_with_commands(response, claude_req.model.clone(), email).await;
            } else {
                // Non-streaming mode - собираем stream и конвертируем в Claude JSON
                return handle_non_streaming_response(response, claude_req.model.clone(), email)
                    .await;
            }
        } else {
            let status_code = status.as_u16();
            let error_text = response.text().await.unwrap_or_default();
            
            error!("Kiro API error {}: {}", status_code, error_text);
            
            // [LAZY REFRESH] Проверяем на истекший токен
            // AWS возвращает 403 с "ExpiredToken" или "ExpiredTokenException"
            let is_expired_token = (status_code == 403 || status_code == 401) 
                && (error_text.contains("ExpiredToken") 
                    || error_text.contains("expired") 
                    || error_text.contains("security token included in the request is expired"));
            
            if is_expired_token {
                error!(
                    "⚠️  Kiro token expired for account {}. Marking as failed and trying next account.",
                    email
                );
                failed_accounts.insert(account_id);
                last_error = format!("Kiro token expired: {}", error_text);
                continue;
            }
            
            if should_rotate_account(status_code) {
                failed_accounts.insert(account_id);
                last_error = format!("Kiro API error {}: {}", status_code, error_text);
                
                let retry_strategy = determine_retry_strategy(status_code, &error_text, false);
                if !apply_retry_strategy(
                    retry_strategy,
                    attempt,
                    max_attempts,
                    status_code,
                    "kiro",
                )
                .await
                {
                    return Err((
                        StatusCode::TOO_MANY_REQUESTS,
                        "Retry limit exceeded".to_string(),
                    ));
                }
                continue;
            } else {
                return Err((status, error_text));
            }
        }
    }
    
    Err((
        StatusCode::SERVICE_UNAVAILABLE,
        format!(
            "All Kiro accounts exhausted after {} attempts. Last error: {}",
            max_attempts, last_error
        ),
    ))
}

/// Обрабатывает streaming response с парсингом команд
/// Собирает весь stream, парсит команды и отправляет правильные SSE события
async fn handle_streaming_with_commands(
    response: reqwest::Response,
    model: String,
    email: String,
) -> Result<Response, (StatusCode, String)> {
    use futures::StreamExt;
    
    let stream = response.bytes_stream().map(|r| r.map_err(|e| e.to_string()));
    
    // Собираем весь stream в текст
    let full_text = match collect_stream_to_text(stream).await {
        Ok(text) => text,
        Err(e) => {
            error!("Failed to collect stream: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Stream collection error: {}", e),
            ));
        }
    };
    
    // Парсим команды из текста
    use crate::proxy::mappers::kiro::command_parser::parse_commands_from_text;
    let content_blocks = parse_commands_from_text(&full_text);
    
    // Генерируем ID для сообщения
    let message_id = format!("msg_{}", uuid::Uuid::new_v4().simple());
    
    // Создаем SSE события
    let mut sse_events = Vec::new();
    
    // 1. message_start
    let message_start = json!({
        "type": "message_start",
        "message": {
            "id": message_id,
            "type": "message",
            "role": "assistant",
            "content": [],
            "model": model,
            "stop_reason": null,
            "stop_sequence": null,
            "usage": {
                "input_tokens": 0,
                "output_tokens": 0
            }
        }
    });
    sse_events.push(format!("event: message_start\ndata: {}\n\n", 
        serde_json::to_string(&message_start).unwrap()));
    
    // 2. Отправляем content blocks
    for (index, block) in content_blocks.iter().enumerate() {
        use crate::proxy::mappers::claude::models::ContentBlock;
        
        match block {
            ContentBlock::Text { text, .. } => {
                // content_block_start
                let block_start = json!({
                    "type": "content_block_start",
                    "index": index,
                    "content_block": {
                        "type": "text",
                        "text": ""
                    }
                });
                sse_events.push(format!("event: content_block_start\ndata: {}\n\n", 
                    serde_json::to_string(&block_start).unwrap()));
                
                // content_block_delta
                let block_delta = json!({
                    "type": "content_block_delta",
                    "index": index,
                    "delta": {
                        "type": "text_delta",
                        "text": text
                    }
                });
                sse_events.push(format!("event: content_block_delta\ndata: {}\n\n", 
                    serde_json::to_string(&block_delta).unwrap()));
                
                // content_block_stop
                let block_stop = json!({
                    "type": "content_block_stop",
                    "index": index
                });
                sse_events.push(format!("event: content_block_stop\ndata: {}\n\n", 
                    serde_json::to_string(&block_stop).unwrap()));
            }
            ContentBlock::ToolUse { id, name, input, .. } => {
                // content_block_start
                let block_start = json!({
                    "type": "content_block_start",
                    "index": index,
                    "content_block": {
                        "type": "tool_use",
                        "id": id,
                        "name": name
                    }
                });
                sse_events.push(format!("event: content_block_start\ndata: {}\n\n", 
                    serde_json::to_string(&block_start).unwrap()));
                
                // content_block_delta with input
                let block_delta = json!({
                    "type": "content_block_delta",
                    "index": index,
                    "delta": {
                        "type": "input_json_delta",
                        "partial_json": serde_json::to_string(input).unwrap()
                    }
                });
                sse_events.push(format!("event: content_block_delta\ndata: {}\n\n", 
                    serde_json::to_string(&block_delta).unwrap()));
                
                // content_block_stop
                let block_stop = json!({
                    "type": "content_block_stop",
                    "index": index
                });
                sse_events.push(format!("event: content_block_stop\ndata: {}\n\n", 
                    serde_json::to_string(&block_stop).unwrap()));
            }
            _ => {
                // Другие типы блоков (thinking и т.д.) пока не поддерживаем
                debug!("Skipping unsupported content block type in streaming");
            }
        }
    }
    
    // 3. message_delta
    let has_tool_use = content_blocks.iter().any(|b| matches!(b, 
        crate::proxy::mappers::claude::models::ContentBlock::ToolUse { .. }));
    
    let stop_reason = if has_tool_use { "tool_use" } else { "end_turn" };
    
    let message_delta = json!({
        "type": "message_delta",
        "delta": {
            "stop_reason": stop_reason,
            "stop_sequence": null
        },
        "usage": {
            "output_tokens": 0
        }
    });
    sse_events.push(format!("event: message_delta\ndata: {}\n\n", 
        serde_json::to_string(&message_delta).unwrap()));
    
    // 4. message_stop
    let message_stop = json!({
        "type": "message_stop"
    });
    sse_events.push(format!("event: message_stop\ndata: {}\n\n", 
        serde_json::to_string(&message_stop).unwrap()));
    
    // Объединяем все события в один stream
    let full_response = sse_events.join("");
    
    Ok(Response::builder()
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .header("X-Accel-Buffering", "no")
        .header("X-Account-Email", &email)
        .header("X-Mapped-Model", &model)
        .body(Body::from(full_response))
        .unwrap())
}

/// Обрабатывает non-streaming response (собирает stream и конвертирует в Claude JSON)
async fn handle_non_streaming_response(
    response: reqwest::Response,
    model: String,
    email: String,
) -> Result<Response, (StatusCode, String)> {
    use futures::StreamExt;
    
    let stream = response.bytes_stream().map(|r| r.map_err(|e| e.to_string()));
    
    // Собираем весь stream в текст
    let full_text = match collect_stream_to_text(stream).await {
        Ok(text) => text,
        Err(e) => {
            error!("Failed to collect stream: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Stream collection error: {}", e),
            ));
        }
    };
    
    // Конвертируем в Claude формат
    let claude_response = convert_kiro_to_claude(full_text, model.clone(), None);
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("X-Account-Email", &email)
        .header("X-Mapped-Model", &model)
        .body(Body::from(serde_json::to_string(&claude_response).unwrap()))
        .unwrap())
}

/// Handle Kiro models list endpoint
pub async fn handle_kiro_models(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    info!("Received Kiro models list request");
    
    // Возвращаем статический список поддерживаемых моделей
    let models = json!({
        "object": "list",
        "data": [
            {
                "id": "auto",
                "object": "model",
                "created": 1700000000,
                "owned_by": "kiro",
                "description": "Smart router - automatically selects the best model"
            },
            {
                "id": "claude-sonnet-4",
                "object": "model",
                "created": 1700000000,
                "owned_by": "anthropic"
            },
            {
                "id": "claude-sonnet-4-5",
                "object": "model",
                "created": 1700000000,
                "owned_by": "anthropic"
            },
            {
                "id": "claude-haiku-4-5",
                "object": "model",
                "created": 1700000000,
                "owned_by": "anthropic",
                "description": "Fast and cheap"
            },
            {
                "id": "claude-opus-4-5",
                "object": "model",
                "created": 1700000000,
                "owned_by": "anthropic",
                "description": "Most powerful"
            },
            {
                "id": "claude-opus-4-6",
                "object": "model",
                "created": 1700000000,
                "owned_by": "anthropic",
                "description": "Best for coding"
            },
            {
                "id": "deepseek-3",
                "object": "model",
                "created": 1700000000,
                "owned_by": "deepseek",
                "description": "0.25x credits, best for agentic workflows and code generation"
            },
            {
                "id": "minimax-2-1",
                "object": "model",
                "created": 1700000000,
                "owned_by": "minimax",
                "description": "0.15x credits, best for multilingual programming and UI generation"
            },
            {
                "id": "qwen3-coder-next",
                "object": "model",
                "created": 1700000000,
                "owned_by": "qwen",
                "description": "0.05x credits, 256K context, best for coding agents"
            }
        ]
    });
    
    Ok(Json(models))
}
