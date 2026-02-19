// Kiro API 数据模型

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Kiro API Request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KiroRequest {
    pub conversation_state: ConversationState,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub history: Vec<HistoryMessage>,
    pub profile_arn: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationState {
    pub agent_continuation_id: String,
    pub agent_task_type: String, // "vibe" or "simple-task"
    pub chat_trigger_type: String, // "MANUAL"
    pub conversation_id: String,
    pub current_message: CurrentMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentMessage {
    pub user_input_message: UserInputMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInputMessage {
    pub content: String,
    pub model_id: String, // "auto", "claude-sonnet-4-5", etc.
    pub origin: String, // "AI_EDITOR"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_input_message_context: Option<UserInputMessageContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInputMessageContext {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tools: Vec<Value>, // Tool specifications (optional)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_input_message: Option<UserInputMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_response_message: Option<AssistantResponseMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantResponseMessage {
    pub content: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tool_uses: Vec<Value>,
}

/// AWS Event Stream Event Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = ":event-type")]
pub enum KiroEvent {
    #[serde(rename = "assistantResponseEvent")]
    AssistantResponse { content: String },
    
    #[serde(rename = "meteringEvent")]
    Metering {
        unit: String,
        #[serde(rename = "unitPlural")]
        unit_plural: String,
        usage: f64,
    },
    
    #[serde(rename = "contextUsageEvent")]
    ContextUsage {
        #[serde(rename = "contextUsagePercentage")]
        context_usage_percentage: f64,
    },
}

/// AWS Event Stream Message
#[derive(Debug, Clone)]
pub struct EventStreamMessage {
    pub headers: std::collections::HashMap<String, String>,
    pub payload: Vec<u8>,
}
