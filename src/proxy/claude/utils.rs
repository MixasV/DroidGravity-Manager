// Claude utilities

use super::models::*;

/// Convert Gemini UsageMetadata to Claude Usage
pub fn to_claude_usage(usage: &UsageMetadata) -> Usage {
    Usage {
        input_tokens: usage.prompt_token_count.unwrap_or(0),
        output_tokens: usage.candidates_token_count.unwrap_or(0),
        cache_read_input_tokens: usage.cached_content_token_count,
        cache_creation_input_tokens: None,
        server_tool_use: None,
    }
}
