// Kiro Request конвертация (Anthropic → Kiro)

use super::models::*;
use crate::proxy::mappers::claude::models::{ClaudeRequest, Message as ClaudeMessage};
use uuid::Uuid;

/// Конвертирует Anthropic Claude запрос в Kiro формат
pub fn convert_claude_to_kiro(
    claude_req: &ClaudeRequest,
    profile_arn: &str,
    conversation_id: Option<String>,
) -> KiroRequest {
    // Генерируем или используем существующий conversation_id
    let conv_id = conversation_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    
    // Извлекаем model_id из claude_req.model
    let model_id = extract_model_id(&claude_req.model);
    
    // Строим контент текущего сообщения
    let current_content = build_current_message_content(claude_req);
    
    // Строим историю из предыдущих сообщений
    let history = build_history(&claude_req.messages);
    
    KiroRequest {
        conversation_state: ConversationState {
            agent_continuation_id: Uuid::new_v4().to_string(),
            agent_task_type: "vibe".to_string(),
            chat_trigger_type: "MANUAL".to_string(),
            conversation_id: conv_id,
            history,
            current_message: CurrentMessage {
                user_input_message: UserInputMessage {
                    content: current_content,
                    model_id,
                    origin: "AI_EDITOR".to_string(),
                    user_input_message_context: None, // Tools пока не поддерживаем
                },
            },
        },
        profile_arn: profile_arn.to_string(),
    }
}

/// Извлекает model_id из имени модели
fn extract_model_id(model: &str) -> String {
    // Маппинг моделей Factory Droid → Kiro:
    // claude-sonnet-4-5 -> claude-sonnet-4.5 (заменяем последний дефис на точку!)
    // claude-haiku-4-5 -> claude-haiku-4.5
    // claude-opus-4-5 -> claude-opus-4.5
    // claude-opus-4-6 -> claude-opus-4.6
    // deepseek-3 -> deepseek-3 (без изменений)
    // minimax-2-1 -> minimax-2-1 (без изменений)
    // qwen3-coder-next -> qwen3-coder-next (без изменений)
    // auto -> auto (smart router)
    
    // Убираем префикс anthropic. если есть
    let clean_model = model.trim_start_matches("anthropic.");
    
    // Для Claude моделей заменяем последний дефис на точку
    // claude-sonnet-4-5 → claude-sonnet-4.5
    if clean_model.starts_with("claude-") {
        // Находим последний дефис
        if let Some(last_dash_pos) = clean_model.rfind('-') {
            // Проверяем что после дефиса идет цифра (версия)
            let after_dash = &clean_model[last_dash_pos + 1..];
            if !after_dash.is_empty() && after_dash.chars().next().unwrap().is_ascii_digit() {
                // Заменяем последний дефис на точку
                let before_dash = &clean_model[..last_dash_pos];
                return format!("{}.{}", before_dash, after_dash);
            }
        }
        // Если не нашли дефис перед версией, возвращаем как есть
        return clean_model.to_string();
    }
    
    // Остальные модели возвращаем без изменений
    clean_model.to_string()
}

/// Строит контент текущего сообщения из последнего user message
fn build_current_message_content(claude_req: &ClaudeRequest) -> String {
    let mut content_parts = Vec::new();
    
    // Добавляем system prompt если есть
    if let Some(system) = &claude_req.system {
        let system_text = match system {
            crate::proxy::mappers::claude::models::SystemPrompt::Text(text) => text.clone(),
            crate::proxy::mappers::claude::models::SystemPrompt::Blocks(blocks) => {
                blocks.iter()
                    .map(|b| b.text.clone())
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        };
        
        if !system_text.is_empty() {
            content_parts.push(format!("<system>\n{}\n</system>", system_text));
        }
    }
    
    // Находим последнее user сообщение
    if let Some(last_user_msg) = claude_req.messages.iter().rev().find(|m| m.role == "user") {
        let user_content = extract_text_from_message(last_user_msg);
        content_parts.push(user_content);
    }
    
    content_parts.join("\n\n")
}

/// Строит историю из предыдущих сообщений (кроме последнего user)
fn build_history(messages: &[ClaudeMessage]) -> Vec<HistoryMessage> {
    let mut history = Vec::new();
    
    // Пропускаем последнее user сообщение (оно в currentMessage)
    let messages_for_history = if messages.last().map(|m| m.role.as_str()) == Some("user") {
        &messages[..messages.len().saturating_sub(1)]
    } else {
        messages
    };
    
    for msg in messages_for_history {
        let history_msg = match msg.role.as_str() {
            "user" => HistoryMessage {
                user_input_message: Some(UserInputMessage {
                    content: extract_text_from_message(msg),
                    model_id: extract_model_id("auto"), // Используем auto для истории
                    origin: "AI_EDITOR".to_string(),
                    user_input_message_context: None,
                }),
                assistant_response_message: None,
            },
            "assistant" => HistoryMessage {
                user_input_message: None,
                assistant_response_message: Some(AssistantResponseMessage {
                    content: extract_text_from_message(msg),
                    tool_uses: Vec::new(), // Tools пока не поддерживаем
                }),
            },
            _ => continue,
        };
        
        history.push(history_msg);
    }
    
    history
}

/// Извлекает текст из Claude сообщения
fn extract_text_from_message(msg: &ClaudeMessage) -> String {
    use crate::proxy::mappers::claude::models::MessageContent;
    
    match &msg.content {
        MessageContent::String(text) => text.clone(),
        MessageContent::Array(blocks) => {
            blocks
                .iter()
                .filter_map(|block| {
                    use crate::proxy::mappers::claude::models::ContentBlock;
                    match block {
                        ContentBlock::Text { text, .. } => Some(text.clone()),
                        ContentBlock::Thinking { thinking, .. } => {
                            Some(format!("<thinking>\n{}\n</thinking>", thinking))
                        }
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_model_id() {
        // Claude модели: заменяем последний дефис на точку
        assert_eq!(extract_model_id("claude-sonnet-4-5"), "claude-sonnet-4.5");
        assert_eq!(extract_model_id("claude-haiku-4-5"), "claude-haiku-4.5");
        assert_eq!(extract_model_id("claude-opus-4-5"), "claude-opus-4.5");
        assert_eq!(extract_model_id("claude-opus-4-6"), "claude-opus-4.6");
        assert_eq!(extract_model_id("anthropic.claude-opus-4-6"), "claude-opus-4.6");
        
        // Остальные модели без изменений
        assert_eq!(extract_model_id("auto"), "auto");
        assert_eq!(extract_model_id("deepseek-3"), "deepseek-3");
        assert_eq!(extract_model_id("minimax-2-1"), "minimax-2-1");
        assert_eq!(extract_model_id("qwen3-coder-next"), "qwen3-coder-next");
    }
}
