// Minimal Claude ↔ Gemini converter for drovity
// Handles basic text messages for Factory Droid compatibility

use anyhow::Result;
use serde_json::{json, Value};

/// Convert Claude request to Gemini v1internal format
/// Simplified version - handles text messages only
pub fn claude_to_gemini_request(claude_body: &Value, model: &str, project_id: &str) -> Result<Value> {
    // Extract Claude request fields
    let messages = claude_body["messages"].as_array()
        .ok_or_else(|| anyhow::anyhow!("Missing messages"))?;
    
    let system_prompt = claude_body.get("system");
    
    // Build Gemini contents
    let mut contents = Vec::new();
    
    for msg in messages {
        let role = match msg["role"].as_str().unwrap_or("user") {
            "assistant" => "model",
            role => role,
        };
        
        // Extract text from Claude content format
        let text = match &msg["content"] {
            Value::String(s) => s.clone(),
            Value::Array(arr) => {
                // Claude uses [{type: "text", text: "..."}]
                arr.iter()
                    .filter_map(|block| {
                        if block["type"] == "text" {
                            block["text"].as_str().map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            _ => String::new(),
        };
        
        if !text.is_empty() {
            contents.push(json!({
                "role": role,
                "parts": [{"text": text}]
            }));
        }
    }
    
    // Build system instruction
    let system_instruction = if let Some(sys) = system_prompt {
        let system_text = match sys {
            Value::String(s) => s.clone(),
            Value::Array(arr) => {
                arr.iter()
                    .filter_map(|block| {
                        if block["type"] == "text" {
                            block["text"].as_str().map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            _ => String::new(),
        };
        
        if !system_text.is_empty() {
            Some(json!({
                "role": "user",
                "parts": [{"text": system_text}]
            }))
        } else {
            None
        }
    } else {
        None
    };
    
    // Build inner request
    let mut inner_request = json!({
        "contents": contents,
        "generationConfig": {
            "maxOutputTokens": claude_body.get("max_tokens").unwrap_or(&json!(8192)),
            "temperature": claude_body.get("temperature").unwrap_or(&json!(1.0)),
        },
        "safetySettings": [
            {"category": "HARM_CATEGORY_HARASSMENT", "threshold": "OFF"},
            {"category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "OFF"},
            {"category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "OFF"},
            {"category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "OFF"},
            {"category": "HARM_CATEGORY_CIVIC_INTEGRITY", "threshold": "OFF"}
        ]
    });
    
    if let Some(sys_inst) = system_instruction {
        inner_request["systemInstruction"] = sys_inst;
    }
    
    // Wrap in v1internal envelope
    Ok(json!({
        "project": project_id,
        "requestId": format!("drovity-{}", uuid::Uuid::new_v4()),
        "request": inner_request,
        "model": model,
        "userAgent": "antigravity",
        "requestType": "agent"
    }))
}

/// Convert Gemini response to Claude response format
/// Simplified version - handles text messages only
pub fn gemini_to_claude_response(gemini_resp: &Value, model: &str) -> Result<Value> {
    // Extract Gemini response fields
    let response_data = gemini_resp.get("response").unwrap_or(gemini_resp);
    
    let candidates = response_data["candidates"].as_array()
        .ok_or_else(|| anyhow::anyhow!("Missing candidates"))?;
    
    let candidate = candidates.get(0)
        .ok_or_else(|| anyhow::anyhow!("Empty candidates"))?;
    
    let parts = candidate["content"]["parts"].as_array()
        .ok_or_else(|| anyhow::anyhow!("Missing parts"))?;
    
    // Build Claude content blocks
    let mut content_blocks = Vec::new();
    
    for part in parts {
        if let Some(text) = part["text"].as_str() {
            if !text.is_empty() {
                content_blocks.push(json!({
                    "type": "text",
                    "text": text
                }));
            }
        }
    }
    
    // Ensure at least one content block
    if content_blocks.is_empty() {
        content_blocks.push(json!({
            "type": "text",
            "text": ""
        }));
    }
    
    // Extract usage
    let usage_metadata = response_data.get("usageMetadata");
    let input_tokens = usage_metadata
        .and_then(|u| u["promptTokenCount"].as_i64())
        .unwrap_or(0);
    let output_tokens = usage_metadata
        .and_then(|u| u["candidatesTokenCount"].as_i64())
        .unwrap_or(0);
    
    // Build Claude response
    Ok(json!({
        "id": format!("msg_{}", uuid::Uuid::new_v4()),
        "type": "message",
        "role": "assistant",
        "model": model,
        "content": content_blocks,
        "stop_reason": "end_turn",
        "stop_sequence": null,
        "usage": {
            "input_tokens": input_tokens,
            "output_tokens": output_tokens
        }
    }))
}
