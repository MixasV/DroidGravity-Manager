# Исправления для ошибки "Missing thought_signature" в Gemini API

## Проблема

При отправке запросов к Gemini API (особенно повторных/catch-up запросов) возникала ошибка:
```
Function call is missing a thought_signature in functionCall parts.
This is required for tools to work correctly, and missing thought_signature
may lead to degraded model performance.
```

## Корневая причина

1. **Gemini API не принимает sentinel-значение** `skip_thought_signature_validator`
   - Claude модели используют этот sentinel как workaround
   - Gemini требует либо **реальную подпись**, либо **полное отсутствие поля**

2. **Проблема с catch-up запросами**:
   - Первый запрос создает подпись, которая кэшируется
   - Второй запрос отправляется ДО того, как первый завершится и закэширует подпись
   - Код пытался добавить sentinel вместо отсутствующей подписи → 400 error

## Внесенные исправления

### 1. `src-tauri/src/proxy/mappers/openai/request.rs`

#### Исправление в thinking parts (~строка 195):
```rust
// БЫЛО:
if let Some(ref sig) = global_thought_sig {
    thought_part["thoughtSignature"] = json!(sig);
} else if !mapped_model.starts_with("projects/") && mapped_model.contains("gemini") {
    thought_part["thoughtSignature"] = json!("skip_thought_signature_validator");
}

// СТАЛО:
// [FIX] Gemini API не принимает skip_thought_signature_validator sentinel
// Только в случае наличия реальной подписи добавляем поле
if let Some(ref sig) = global_thought_sig {
    thought_part["thoughtSignature"] = json!(sig);
}
// Gemini: если нет подписи - не добавляем thoughtSignature поле
```

#### Исправление в tool_calls parts (~строка 320):
```rust
// БЫЛО:
if let Some(ref sig) = global_thought_sig {
    func_call_part["thoughtSignature"] = json!(sig);
} else if is_thinking_model && !mapped_model.starts_with("projects/") {
    func_call_part["thoughtSignature"] = json!("skip_thought_signature_validator");
}

// СТАЛО:
// [FIX] Gemini API не принимает skip_thought_signature_validator sentinel
if let Some(ref sig) = global_thought_sig {
    func_call_part["thoughtSignature"] = json!(sig);
    tracing::debug!("[OpenAI-Signature] Using cached signature for tool_use: {}", tc.id);
} else if is_thinking_model && mapped_model.contains("claude") {
    // Только Claude модели используют sentinel, Gemini не поддерживает
    tracing::debug!("[OpenAI-Signature] Adding CLAUDE_SKIP_SIGNATURE for tool_use: {}", tc.id);
    func_call_part["thoughtSignature"] = json!("skip_thought_signature_validator");
}
// Gemini: если нет подписи - не добавляем thoughtSignature поле
```

### 2. `src-tauri/src/proxy/mappers/gemini/wrapper.rs`

#### Улучшение логики инжекции (~строка 22):
```rust
// БЫЛО:
if part.get("functionCall").is_some() {
    // Only inject if it doesn't already have one
    if part.get("thoughtSignature").is_none() {
        if let Some(sig) = crate::proxy::SignatureCache::global().get_session_signature(s_id) {
            if let Some(obj) = part.as_object_mut() {
                obj.insert("thoughtSignature".to_string(), json!(sig));
            }
        }
    }
}

// СТАЛО:
if part.get("functionCall").is_some() {
    if let Some(obj) = part.as_object_mut() {
        // Проверяем наличие подписи в кэше
        if let Some(sig) = crate::proxy::SignatureCache::global().get_session_signature(s_id) {
            // Есть реальная подпись - добавляем/обновляем
            obj.insert("thoughtSignature".to_string(), json!(sig));
            tracing::debug!("[Gemini-Wrap] Injected signature (len: {}) for session: {}", sig.len(), s_id);
        } else {
            // Нет подписи в кэше - удаляем поле полностью
            if obj.remove("thoughtSignature").is_some() {
                tracing::debug!("[Gemini-Wrap] Removed empty/sentinel thoughtSignature field");
            }
        }
    }
}
```

## Дополнительная проблема: maxOutputTokens

### Обнаруженная проблема в `C:\Users\mihai\.factory\settings.json`:

Все значения `maxOutputTokens` в 10 раз больше, чем должны быть:

| Модель | Текущее значение | Должно быть |
|--------|-----------------|-------------|
| gemini-3-flash | 240576 | 24576 |
| gemini-3-pro-high/low | 320768 | 32768 |
| gemini-2.5-flash | 240576 | 24576 |
| gemini-2.5-flash-lite | 240576 | 24576 |
| gemini-2.5-pro | 320768 | 32768 |
| gemini-2.5-flash-thinking | 240576 | 24576 |
| claude-sonnet-4-5 | 80192 | 8192 |
| claude-sonnet-4-5-thinking | 160384 | 16384 |
| claude-opus-4-5-thinking | 160384 | 16384 |

### Рекомендация:

**Необходимо исправить вручную** файл `C:\Users\mihai\.factory\settings.json` - разделить все значения `maxOutputTokens` на 10.

## Статус

✅ **Исправления кода внесены**:
- `openai/request.rs` - 2 места исправлены
- `gemini/wrapper.rs` - 1 место улучшено

⚠️ **Проверка компиляции заблокирована**:
- Ошибка Windows SDK: `LINK : fatal error LNK1181: не удается открыть входной файл "kernel32.lib"`
- Это проблема окружения сборки, **НЕ ошибка в коде**

❌ **Требуется ручное исправление**:
- `C:\Users\mihai\.factory\settings.json` - исправить maxOutputTokens

## Следующие шаги

1. **Исправить maxOutputTokens** в settings.json (разделить на 10)
2. **Решить проблему Windows SDK** для возможности компиляции
3. **Пересобрать проект**: `cargo build --release`
4. **Протестировать** с реальными Gemini запросами
5. **Опционально**: Применить дополнительные upstream исправления (5xx error handling, 403 validation)
