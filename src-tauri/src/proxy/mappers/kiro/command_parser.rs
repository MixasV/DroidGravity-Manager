// Kiro Command Parser
// Парсит команды из текста Kiro ответа и конвертирует в Claude tool calls

use crate::proxy::mappers::claude::models::ContentBlock;
use regex::Regex;

/// Парсит текст и извлекает команды в формате XML
/// Возвращает список content blocks (text + tool_use)
pub fn parse_commands_from_text(text: &str) -> Vec<ContentBlock> {
    let mut blocks = Vec::new();
    let mut current_text = String::new();
    let mut pos = 0;
    
    // Регулярки для команд
    let command_patterns = vec![
        (r"<readCode><file>(.*?)</file></readCode>", "readCode", vec!["file"]),
        (r"<readFile><file>(.*?)</file></readFile>", "readFile", vec!["file"]),
        (r"<ls><path>(.*?)</path></ls>", "ls", vec!["path"]),
        (r"<grep><pattern>(.*?)</pattern><path>(.*?)</path></grep>", "grep", vec!["pattern", "path"]),
        (r"<glob><pattern>(.*?)</pattern><path>(.*?)</path></glob>", "glob", vec!["pattern", "path"]),
    ];
    
    while pos < text.len() {
        let mut found_command = false;
        
        // Ищем ближайшую команду
        for (pattern, tool_name, param_names) in &command_patterns {
            let re = Regex::new(pattern).unwrap();
            
            if let Some(captures) = re.captures(&text[pos..]) {
                if let Some(m) = captures.get(0) {
                    let match_start = pos + m.start();
                    let match_end = pos + m.end();
                    
                    // Добавляем текст до команды
                    if match_start > pos {
                        current_text.push_str(&text[pos..match_start]);
                    }
                    
                    // Flush текст если есть
                    if !current_text.trim().is_empty() {
                        blocks.push(ContentBlock::Text {
                            text: current_text.clone(),
                            cache_control: None,
                        });
                        current_text.clear();
                    }
                    
                    // Создаём tool_use
                    let mut input = serde_json::Map::new();
                    for (i, param_name) in param_names.iter().enumerate() {
                        if let Some(value) = captures.get(i + 1) {
                            input.insert(
                                param_name.to_string(),
                                serde_json::Value::String(value.as_str().to_string())
                            );
                        }
                    }
                    
                    blocks.push(ContentBlock::ToolUse {
                        id: format!("toolu_{}", uuid::Uuid::new_v4().simple()),
                        name: tool_name.to_string(),
                        input: serde_json::Value::Object(input),
                        signature: None,
                        cache_control: None,
                    });
                    
                    pos = match_end;
                    found_command = true;
                    break;
                }
            }
        }
        
        if !found_command {
            // Нет команд, добавляем символ к текущему тексту
            if let Some(ch) = text[pos..].chars().next() {
                current_text.push(ch);
                pos += ch.len_utf8();
            } else {
                break;
            }
        }
    }
    
    // Добавляем оставшийся текст
    if !current_text.trim().is_empty() {
        blocks.push(ContentBlock::Text {
            text: current_text,
            cache_control: None,
        });
    }
    
    // Если нет блоков, возвращаем весь текст
    if blocks.is_empty() {
        blocks.push(ContentBlock::Text {
            text: text.to_string(),
            cache_control: None,
        });
    }
    
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_readcode_command() {
        let text = "Let me read the file: <readCode><file>src/main.rs</file></readCode>";
        let blocks = parse_commands_from_text(text);
        
        assert_eq!(blocks.len(), 2);
        
        // First block should be text
        if let ContentBlock::Text { text, .. } = &blocks[0] {
            assert_eq!(text, "Let me read the file: ");
        } else {
            panic!("Expected text block");
        }
        
        // Second block should be tool_use
        if let ContentBlock::ToolUse { name, input, .. } = &blocks[1] {
            assert_eq!(name, "readCode");
            assert_eq!(input["file"], "src/main.rs");
        } else {
            panic!("Expected tool_use block");
        }
    }
    
    #[test]
    fn test_parse_multiple_commands() {
        let text = "First <ls><path>.</path></ls> then <grep><pattern>TODO</pattern><path>src</path></grep>";
        let blocks = parse_commands_from_text(text);
        
        assert_eq!(blocks.len(), 4); // text, tool, text, tool
    }
    
    #[test]
    fn test_no_commands() {
        let text = "Just plain text without any commands";
        let blocks = parse_commands_from_text(text);
        
        assert_eq!(blocks.len(), 1);
        if let ContentBlock::Text { text: t, .. } = &blocks[0] {
            assert_eq!(t, text);
        }
    }
}
