// 模型名称映射
use std::collections::HashMap;
use once_cell::sync::Lazy;

static CLAUDE_TO_GEMINI: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // 直接支持的模型
    m.insert("claude-opus-4-5-thinking", "claude-opus-4-5-thinking");
    m.insert("claude-sonnet-4-5", "claude-sonnet-4-5");
    m.insert("claude-sonnet-4-5-thinking", "claude-sonnet-4-5-thinking");

    // 别名映射
    m.insert("claude-sonnet-4-5-20250929", "claude-sonnet-4-5-thinking");
    m.insert("claude-3-5-sonnet-20241022", "claude-sonnet-4-5");
    m.insert("claude-3-5-sonnet-20240620", "claude-sonnet-4-5");
    m.insert("claude-opus-4", "claude-opus-4-5-thinking");
    m.insert("claude-opus-4-5-20251101", "claude-opus-4-5-thinking");
    m.insert("claude-haiku-4", "claude-sonnet-4-5");
    m.insert("claude-3-haiku-20240307", "claude-sonnet-4-5");
    m.insert("claude-haiku-4-5-20251001", "claude-sonnet-4-5");
    // OpenAI 协议映射表
    m.insert("gpt-4", "gemini-2.5-pro");
    m.insert("gpt-4-turbo", "gemini-2.5-pro");
    m.insert("gpt-4-turbo-preview", "gemini-2.5-pro");
    m.insert("gpt-4-0125-preview", "gemini-2.5-pro");
    m.insert("gpt-4-1106-preview", "gemini-2.5-pro");
    m.insert("gpt-4-0613", "gemini-2.5-pro");

    m.insert("gpt-4o", "gemini-2.5-pro");
    m.insert("gpt-4o-2024-05-13", "gemini-2.5-pro");
    m.insert("gpt-4o-2024-08-06", "gemini-2.5-pro");

    m.insert("gpt-4o-mini", "gemini-2.5-flash");
    m.insert("gpt-4o-mini-2024-07-18", "gemini-2.5-flash");

    m.insert("gpt-3.5-turbo", "gemini-2.5-flash");
    m.insert("gpt-3.5-turbo-16k", "gemini-2.5-flash");
    m.insert("gpt-3.5-turbo-0125", "gemini-2.5-flash");
    m.insert("gpt-3.5-turbo-1106", "gemini-2.5-flash");
    m.insert("gpt-3.5-turbo-0613", "gemini-2.5-flash");

    // Gemini 协议映射表
    m.insert("gemini-2.5-flash-lite", "gemini-2.5-flash-lite");
    m.insert("gemini-2.5-flash-thinking", "gemini-2.5-flash-thinking");
    m.insert("gemini-3-pro-low", "gemini-3-pro-low");
    m.insert("gemini-3-pro-high", "gemini-3-pro-high");
    m.insert("gemini-3-pro-preview", "gemini-3-pro-preview");
    m.insert("gemini-3-pro", "gemini-3-pro");  // [FIX PR #368] 添加基础模型支持
    m.insert("gemini-2.5-flash", "gemini-2.5-flash");
    m.insert("gemini-3-flash", "gemini-3-flash");
    m.insert("gemini-3-pro-image", "gemini-3-pro-image");


    m
});

/// 别名，用于兼容旧代码
pub fn normalize_to_standard_id(name: &str) -> String {
    map_claude_model_to_gemini(name)
}

pub fn map_claude_model_to_gemini(input: &str) -> String {
    // 1. Check exact match in map
    if let Some(mapped) = CLAUDE_TO_GEMINI.get(input) {
        return mapped.to_string();
    }

    // 2. Pass-through known prefixes (gemini-, -thinking) to support dynamic suffixes
    if input.starts_with("gemini-") || input.contains("thinking") {
        return input.to_string();
    }

    // 3. Fallback to default
    "claude-sonnet-4-5".to_string()
}

/// 获取所有内置支持的模型列表关键字
pub fn get_supported_models() -> Vec<String> {
    CLAUDE_TO_GEMINI.keys().map(|s| s.to_string()).collect()
}

/// 动态获取所有可用模型列表 (包含内置与用户自定义)
pub async fn get_all_dynamic_models(
    custom_mapping: &tokio::sync::RwLock<std::collections::HashMap<String, String>>,
) -> Vec<String> {
    use std::collections::HashSet;
    let mut model_ids = HashSet::new();

    // 1. 获取所有内置映射模型
    for m in get_supported_models() {
        model_ids.insert(m);
    }

    // 2. 获取所有自定义映射模型 (Custom)
    {
        let mapping = custom_mapping.read().await;
        for key in mapping.keys() {
            model_ids.insert(key.clone());
        }
    }

    // 5. 确保包含常用的 Gemini/画画模型 ID
    model_ids.insert("gemini-3-pro-low".to_string());
    
    // [NEW] Issue #247: Dynamically generate all Image Gen Combinations
    let base = "gemini-3-pro-image";
    let resolutions = vec!["", "-2k", "-4k"];
    let ratios = vec!["", "-1x1", "-4x3", "-3x4", "-16x9", "-9x16", "-21x9"];
    
    for res in resolutions {
        for ratio in ratios.iter() {
            let mut id = base.to_string();
            id.push_str(res);
            id.push_str(ratio);
            model_ids.insert(id);
        }
    }

    model_ids.insert("gemini-2.0-flash-exp".to_string());
    model_ids.insert("gemini-2.5-flash".to_string());
    model_ids.insert("gemini-2.5-pro".to_string());
    model_ids.insert("gemini-3-flash".to_string());
    model_ids.insert("gemini-3-pro-high".to_string());
    model_ids.insert("gemini-3-pro-low".to_string());


    let mut sorted_ids: Vec<_> = model_ids.into_iter().collect();
    sorted_ids.sort();
    sorted_ids
}

/// 通配符匹配辅助函数
/// 支持多个 * 通配符匹配 (glob-style)
/// 
/// # 示例
/// - `gpt-4*` 匹配 `gpt-4`, `gpt-4-turbo`, `gpt-4-0613` 等
/// - `claude-3-5-sonnet-*` 匹配所有 3.5 sonnet 版本
/// - `*-thinking` 匹配所有以 `-thinking` 结尾的模型
/// - `claude-*-sonnet-*` 匹配 `claude-3-5-sonnet-20241022` 等 (multi-wildcard)
/// - `*thinking*` 匹配包含 "thinking" 的模型
fn wildcard_match(pattern: &str, text: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == text;
    }
    
    let segments: Vec<&str> = pattern.split('*').collect();
    let mut text_pos = 0;
    let text_chars: Vec<char> = text.chars().collect();
    let text_len = text_chars.len();
    
    for (i, segment) in segments.iter().enumerate() {
        if segment.is_empty() {
            continue;
        }
        
        let segment_chars: Vec<char> = segment.chars().collect();
        let segment_len = segment_chars.len();
        
        if i == 0 {
            if text_len < segment_len {
                return false;
            }
            if text_chars[..segment_len].iter().collect::<String>() != *segment {
                return false;
            }
            text_pos = segment_len;
        } else if i == segments.len() - 1 {
            if text_len < text_pos + segment_len {
                return false;
            }
            if text_chars[text_len - segment_len..].iter().collect::<String>() != *segment {
                return false;
            }
        } else {
            let remaining: String = text_chars[text_pos..].iter().collect();
            if let Some(pos) = remaining.find(segment) {
                text_pos += pos + segment_len;
            } else {
                return false;
            }
        }
    }
    
    true
}

/// Calculate pattern specificity (higher = more specific)
fn pattern_specificity(pattern: &str) -> usize {
    let wildcard_count = pattern.matches('*').count();
    let char_count = pattern.chars().count();
    char_count.saturating_sub(wildcard_count)
}

/// 核心模型路由解析引擎
/// 优先级：精确匹配 > 通配符匹配(按specificity排序) > 系统默认映射
/// 
/// # 参数
/// - `original_model`: 原始模型名称
/// - `custom_mapping`: 用户自定义映射表
/// 
/// # 返回
/// 映射后的目标模型名称
pub fn resolve_model_route(
    original_model: &str,
    custom_mapping: &std::collections::HashMap<String, String>,
) -> String {
    // 1. 精确匹配 (最高优先级)
    if let Some(target) = custom_mapping.get(original_model) {
        crate::modules::logger::log_info(&format!("[Router] 精确映射: {} -> {}", original_model, target));
        return target.clone();
    }
    
    // 2. 通配符匹配 - 收集所有匹配的规则并按specificity排序
    let mut matches: Vec<(&String, &String, usize)> = custom_mapping
        .iter()
        .filter(|(pattern, _)| pattern.contains('*') && wildcard_match(pattern, original_model))
        .map(|(pattern, target)| (pattern, target, pattern_specificity(pattern)))
        .collect();
    
    matches.sort_by(|a, b| b.2.cmp(&a.2));
    
    if let Some((pattern, target, specificity)) = matches.first() {
        crate::modules::logger::log_info(&format!(
            "[Router] 通配符映射: {} -> {} (规则: {}, specificity: {})", 
            original_model, target, pattern, specificity
        ));
        return (*target).clone();
    }
    
    // 3. 系统默认映射
    let result = map_claude_model_to_gemini(original_model);
    if result != original_model {
        crate::modules::logger::log_info(&format!("[Router] 系统默认映射: {} -> {}", original_model, result));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_mapping() {
        assert_eq!(
            map_claude_model_to_gemini("claude-3-5-sonnet-20241022"),
            "claude-sonnet-4-5"
        );
        assert_eq!(
            map_claude_model_to_gemini("claude-opus-4"),
            "claude-opus-4-5-thinking"
        );
        assert_eq!(
            map_claude_model_to_gemini("gemini-2.5-flash-mini-test"),
            "gemini-2.5-flash-mini-test"
        );
        assert_eq!(
            map_claude_model_to_gemini("unknown-model"),
            "claude-sonnet-4-5"
        );
    }

    #[test]
    fn test_wildcard_single() {
        assert!(wildcard_match("gpt-4*", "gpt-4"));
        assert!(wildcard_match("gpt-4*", "gpt-4-turbo"));
        assert!(!wildcard_match("gpt-4*", "gpt-3.5-turbo"));
        assert!(wildcard_match("*-thinking", "claude-opus-4-5-thinking"));
        assert!(!wildcard_match("*-thinking", "claude-opus-4-5"));
        assert!(wildcard_match("claude-*-sonnet", "claude-3-5-sonnet"));
    }

    #[test]
    fn test_wildcard_multi() {
        assert!(wildcard_match("claude-*-sonnet-*", "claude-3-5-sonnet-20241022"));
        assert!(!wildcard_match("claude-*-sonnet-*", "claude-3-5-opus-20241022"));
        assert!(wildcard_match("*thinking*", "claude-opus-4-5-thinking"));
        assert!(wildcard_match("*thinking*", "gemini-thinking-pro"));
        assert!(!wildcard_match("*thinking*", "claude-opus-4-5"));
    }

    #[test]
    fn test_pattern_specificity() {
        assert!(pattern_specificity("claude-opus-4-5-thinking") > pattern_specificity("claude-*"));
        assert!(pattern_specificity("claude-*-sonnet-*") > pattern_specificity("claude-*"));
    }

    #[test]
    fn test_resolve_with_specificity() {
        use std::collections::HashMap;
        
        let mut custom = HashMap::new();
        custom.insert("claude-*".to_string(), "fallback".to_string());
        custom.insert("claude-*-sonnet-*".to_string(), "specific-sonnet".to_string());
        
        let result = resolve_model_route("claude-3-5-sonnet-20241022", &custom);
        assert_eq!(result, "specific-sonnet");
        
        let result2 = resolve_model_route("claude-opus-4", &custom);
        assert_eq!(result2, "fallback");
    }
}
