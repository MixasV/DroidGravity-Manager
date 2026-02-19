// Kiro Response конвертация (Kiro → Anthropic Claude)
// Non-streaming response

use super::models::*;
use crate::proxy::mappers::claude::models::{ClaudeResponse, ContentBlock, Usage};
use serde_json::Value;

/// Конвертирует собранный Kiro response в Claude формат
pub fn convert_kiro_to_claude(
    content: String,
    model: String,
    usage: Option<(u32, u32)>, // (input_tokens, output_tokens)
) -> ClaudeResponse {
    let (input_tokens, output_tokens) = usage.unwrap_or((0, 0));
    
    ClaudeResponse {
        id: format!("msg_{}", uuid::Uuid::new_v4()),
        type_: "message".to_string(),
        role: "assistant".to_string(),
        content: vec![ContentBlock::Text {
            type_: "text".to_string(),
            text: content,
            cache_control: None,
        }],
        model,
        stop_reason: Some("end_turn".to_string()),
        stop_sequence: None,
        usage: Usage {
            input_tokens,
            output_tokens,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        },
    }
}
