// Kiro Streaming конвертация (AWS Event Stream → Claude SSE)

use super::models::*;
use bytes::Bytes;
use serde_json::{json, Value};
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

/// Парсит AWS Event Stream бинарный формат
/// 
/// Формат сообщения:
/// - 4 bytes: total_length (big-endian uint32)
/// - 4 bytes: headers_length (big-endian uint32)
/// - 4 bytes: prelude_crc (big-endian uint32)
/// - headers_length bytes: headers
/// - payload bytes: payload
/// - 4 bytes: message_crc (big-endian uint32)
pub fn parse_event_stream(data: &[u8]) -> Result<Vec<EventStreamMessage>, String> {
    let mut messages = Vec::new();
    let mut cursor = Cursor::new(data);
    
    while cursor.position() < data.len() as u64 {
        // Проверяем что осталось минимум 12 байт для prelude
        if (data.len() as u64 - cursor.position()) < 12 {
            break;
        }
        
        // Читаем prelude (12 байт)
        let total_length = cursor.read_u32::<BigEndian>()
            .map_err(|e| format!("Failed to read total_length: {}", e))?;
        let headers_length = cursor.read_u32::<BigEndian>()
            .map_err(|e| format!("Failed to read headers_length: {}", e))?;
        let _prelude_crc = cursor.read_u32::<BigEndian>()
            .map_err(|e| format!("Failed to read prelude_crc: {}", e))?;
        
        // Проверяем что есть достаточно данных
        let remaining = data.len() as u64 - cursor.position();
        let needed = (total_length as u64) - 12;
        if remaining < needed {
            break;
        }
        
        // Читаем headers
        let mut headers = std::collections::HashMap::new();
        let headers_start = cursor.position();
        let headers_end = headers_start + headers_length as u64;
        
        while cursor.position() < headers_end {
            // Header name length (1 byte)
            let name_len = data[cursor.position() as usize];
            cursor.set_position(cursor.position() + 1);
            
            // Header name
            let name_bytes = &data[cursor.position() as usize..(cursor.position() + name_len as u64) as usize];
            let name = String::from_utf8_lossy(name_bytes).to_string();
            cursor.set_position(cursor.position() + name_len as u64);
            
            // Header value type (1 byte) - пропускаем
            cursor.set_position(cursor.position() + 1);
            
            // Header value length (2 bytes, big-endian)
            let value_len = ((data[cursor.position() as usize] as u16) << 8) 
                | (data[(cursor.position() + 1) as usize] as u16);
            cursor.set_position(cursor.position() + 2);
            
            // Header value
            let value_bytes = &data[cursor.position() as usize..(cursor.position() + value_len as u64) as usize];
            let value = String::from_utf8_lossy(value_bytes).to_string();
            cursor.set_position(cursor.position() + value_len as u64);
            
            headers.insert(name, value);
        }
        
        // Читаем payload
        let payload_start = cursor.position();
        let payload_end = headers_start - 12 + total_length as u64 - 4; // -4 для message_crc
        let payload = data[payload_start as usize..payload_end as usize].to_vec();
        
        // Пропускаем message CRC
        cursor.set_position(payload_end + 4);
        
        messages.push(EventStreamMessage { headers, payload });
    }
    
    Ok(messages)
}

/// Конвертирует Kiro event в Claude SSE chunk
pub fn convert_event_to_claude_chunk(msg: &EventStreamMessage) -> Option<Bytes> {
    let event_type = msg.headers.get(":event-type")?;
    let message_type = msg.headers.get(":message-type")?;
    
    if message_type != "event" {
        return None;
    }
    
    match event_type.as_str() {
        "assistantResponseEvent" => {
            // Парсим payload как JSON
            let payload_json: Value = serde_json::from_slice(&msg.payload).ok()?;
            let content = payload_json.get("content")?.as_str()?;
            
            // Конвертируем в Claude SSE формат
            let claude_chunk = json!({
                "type": "content_block_delta",
                "index": 0,
                "delta": {
                    "type": "text_delta",
                    "text": content
                }
            });
            
            let sse_line = format!("event: content_block_delta\ndata: {}\n\n", 
                serde_json::to_string(&claude_chunk).ok()?);
            
            Some(Bytes::from(sse_line))
        }
        "meteringEvent" => {
            // Логируем usage но не отправляем клиенту
            if let Ok(payload_json) = serde_json::from_slice::<Value>(&msg.payload) {
                if let Some(usage) = payload_json.get("usage") {
                    tracing::debug!("[Kiro] Metering: {} credits", usage);
                }
            }
            None
        }
        "contextUsageEvent" => {
            // Логируем context usage
            if let Ok(payload_json) = serde_json::from_slice::<Value>(&msg.payload) {
                if let Some(percentage) = payload_json.get("contextUsagePercentage") {
                    tracing::debug!("[Kiro] Context usage: {}%", percentage);
                }
            }
            None
        }
        _ => None,
    }
}

/// Собирает stream в полный текст (для non-streaming mode)
pub async fn collect_stream_to_text(
    mut stream: impl futures::Stream<Item = Result<Bytes, String>> + Unpin,
) -> Result<String, String> {
    use futures::StreamExt;
    
    let mut full_text = String::new();
    
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        
        // Парсим AWS Event Stream
        let messages = parse_event_stream(&chunk)?;
        
        for msg in messages {
            if let Some(event_type) = msg.headers.get(":event-type") {
                if event_type == "assistantResponseEvent" {
                    if let Ok(payload_json) = serde_json::from_slice::<Value>(&msg.payload) {
                        if let Some(content) = payload_json.get("content").and_then(|c| c.as_str()) {
                            full_text.push_str(content);
                        }
                    }
                }
            }
        }
    }
    
    Ok(full_text)
}
