//! Ultra Priority Tests for High-End Models (Opus 4.6/4.5)
//!
//! 这些测试验证高端模型（如 Claude Opus 4.6/4.5）优先使用 Ultra 账号的逻辑。
//!
//! ## 背景
//! 用户的账号池包含大量 Gemini Pro 账号和少量 Ultra 账号。当请求 Claude Opus 4.6 模型时，
//! 系统按配额优先的策略可能会选择 Pro 账号，但 Pro 账号无法访问 Opus 4.6，导致 API 返回错误。
//!
//! ## 解决方案
//! 当用户请求高端模型时，优先选择 Ultra 账号；只有 Ultra 账号都不可用时才降级到 Pro/Free 账号。
//!
//! ## 测试覆盖
//! - `test_is_ultra_required_model`: 验证模型识别逻辑
//! - `test_ultra_priority_for_high_end_models`: 验证 Ultra 优先于 Pro（即使 Pro 配额更高）
//! - `test_ultra_accounts_sorted_by_quota`: 验证同为 Ultra 时按配额排序
//! - `test_full_sorting_mixed_accounts`: 验证混合账号池的完整排序

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::proxy::token_manager::ProxyToken;

/// 创建测试用的 ProxyToken
fn create_test_token(
    email: &str,
    tier: Option<&str>,
    health_score: f32,
    reset_time: Option<i64>,
    remaining_quota: Option<i32>,
) -> ProxyToken {
    ProxyToken {
        account_id: email.to_string(),
        access_token: "test_token".to_string(),
        refresh_token: "test_refresh".to_string(),
        expires_in: 3600,
        timestamp: chrono::Utc::now().timestamp() + 3600,
        email: email.to_string(),
        account_path: PathBuf::from("/tmp/test"),
        project_id: None,
        subscription_tier: tier.map(|s| s.to_string()),
        remaining_quota,
        protected_models: HashSet::new(),
        health_score,
        reset_time,
        validation_blocked: false,
        validation_blocked_until: 0,
        model_quotas: HashMap::new(),
    }
}

/// 需要 Ultra 账号的高端模型列表
const ULTRA_REQUIRED_MODELS: &[&str] = &[
    "claude-opus-4-6",
    "claude-opus-4-5",
    "opus", // 通配匹配
];

/// 检查模型是否需要 Ultra 账号
fn is_ultra_required_model(model: &str) -> bool {
    let lower = model.to_lowercase();
    ULTRA_REQUIRED_MODELS.iter().any(|m| lower.contains(m))
}

/// 测试 is_ultra_required_model 辅助函数
#[test]
fn test_is_ultra_required_model() {
    // 应该识别为高端模型
    assert!(is_ultra_required_model("claude-opus-4-6"));
    assert!(is_ultra_required_model("claude-opus-4-5"));
    assert!(is_ultra_required_model("Claude-Opus-4-6")); // 大小写不敏感
    assert!(is_ultra_required_model("CLAUDE-OPUS-4-5")); // 大小写不敏感
    assert!(is_ultra_required_model("opus")); // 通配匹配
    assert!(is_ultra_required_model("opus-4-6-latest"));
    assert!(is_ultra_required_model("models/claude-opus-4-6"));

    // 应该识别为普通模型
    assert!(!is_ultra_required_model("claude-sonnet-4-5"));
    assert!(!is_ultra_required_model("claude-sonnet"));
    assert!(!is_ultra_required_model("gemini-1.5-flash"));
    assert!(!is_ultra_required_model("gemini-2.0-pro"));
    assert!(!is_ultra_required_model("claude-haiku"));
}

/// 模拟 token_manager.rs 中的排序逻辑
fn compare_tokens_for_model(a: &ProxyToken, b: &ProxyToken, target_model: &str) -> Ordering {
    let requires_ultra = is_ultra_required_model(target_model);

    let tier_priority = |tier: &Option<String>| {
        let t = tier.as_deref().unwrap_or("").to_lowercase();
        if t.contains("ultra") { 0 }
        else if t.contains("pro") { 1 }
        else if t.contains("free") { 2 }
        else { 3 }
    };

    // Priority 0: 高端模型时，订阅等级优先
    if requires_ultra {
        let tier_cmp = tier_priority(&a.subscription_tier)
            .cmp(&tier_priority(&b.subscription_tier));
        if tier_cmp != Ordering::Equal {
            return tier_cmp;
        }
    }

    // Priority 1: Quota (higher is better)
    let quota_a = a.remaining_quota.unwrap_or(0);
    let quota_b = b.remaining_quota.unwrap_or(0);
    let quota_cmp = quota_b.cmp(&quota_a);
    if quota_cmp != Ordering::Equal {
        return quota_cmp;
    }

    // Priority 2: Health score
    let health_cmp = b.health_score.partial_cmp(&a.health_score)
        .unwrap_or(Ordering::Equal);
    if health_cmp != Ordering::Equal {
        return health_cmp;
    }

    // Priority 3: Tier (for non-high-end models)
    if !requires_ultra {
        let tier_cmp = tier_priority(&a.subscription_tier)
            .cmp(&tier_priority(&b.subscription_tier));
        if tier_cmp != Ordering::Equal {
            return tier_cmp;
        }
    }

    Ordering::Equal
}

/// 测试高端模型排序：Ultra 账号优先于 Pro 账号（即使 Pro 配额更高）
#[test]
fn test_ultra_priority_for_high_end_models() {
    // 创建测试账号：Ultra 低配额 vs Pro 高配额
    let ultra_low_quota = create_test_token("ultra@test.com", Some("ULTRA"), 1.0, None, Some(20));
    let pro_high_quota = create_test_token("pro@test.com", Some("PRO"), 1.0, None, Some(80));

    // 高端模型 (Opus 4.6): Ultra 应该优先，即使配额低
    assert_eq!(
        compare_tokens_for_model(&ultra_low_quota, &pro_high_quota, "claude-opus-4-6"),
        Ordering::Less, // Ultra 排在前面
        "Opus 4.6 should prefer Ultra account over Pro even with lower quota"
    );

    // 高端模型 (Opus 4.5): Ultra 应该优先
    assert_eq!(
        compare_tokens_for_model(&ultra_low_quota, &pro_high_quota, "claude-opus-4-5"),
        Ordering::Less,
        "Opus 4.5 should prefer Ultra account over Pro"
    );

    // 普通模型 (Sonnet): 高配额 Pro 应该优先
    assert_eq!(
        compare_tokens_for_model(&ultra_low_quota, &pro_high_quota, "claude-sonnet-4-5"),
        Ordering::Greater, // Pro (高配额) 排在前面
        "Sonnet should prefer high-quota Pro over low-quota Ultra"
    );

    // 普通模型 (Flash): 高配额 Pro 应该优先
    assert_eq!(
        compare_tokens_for_model(&ultra_low_quota, &pro_high_quota, "gemini-1.5-flash"),
        Ordering::Greater,
        "Flash should prefer high-quota Pro over low-quota Ultra"
    );
}

/// 测试排序：同为 Ultra 时按配额排序
#[test]
fn test_ultra_accounts_sorted_by_quota() {
    let ultra_high = create_test_token("ultra_high@test.com", Some("ULTRA"), 1.0, None, Some(80));
    let ultra_low = create_test_token("ultra_low@test.com", Some("ULTRA"), 1.0, None, Some(20));

    // Opus 4.6: 同为 Ultra，高配额优先
    assert_eq!(
        compare_tokens_for_model(&ultra_high, &ultra_low, "claude-opus-4-6"),
        Ordering::Less, // ultra_high 排在前面
        "Among Ultra accounts, higher quota should come first"
    );
}

/// 测试完整排序场景：混合账号池
#[test]
fn test_full_sorting_mixed_accounts() {
    fn sort_tokens_for_model(tokens: &mut Vec<ProxyToken>, target_model: &str) {
        tokens.sort_by(|a, b| compare_tokens_for_model(a, b, target_model));
    }

    // 创建混合账号池
    let ultra_high = create_test_token("ultra_high@test.com", Some("ULTRA"), 1.0, None, Some(80));
    let ultra_low = create_test_token("ultra_low@test.com", Some("ULTRA"), 1.0, None, Some(20));
    let pro_high = create_test_token("pro_high@test.com", Some("PRO"), 1.0, None, Some(90));
    let pro_low = create_test_token("pro_low@test.com", Some("PRO"), 1.0, None, Some(30));
    let free = create_test_token("free@test.com", Some("FREE"), 1.0, None, Some(100));

    // 高端模型 (Opus 4.6) 排序
    let mut tokens_opus = vec![pro_high.clone(), free.clone(), ultra_low.clone(), pro_low.clone(), ultra_high.clone()];
    sort_tokens_for_model(&mut tokens_opus, "claude-opus-4-6");

    let emails_opus: Vec<&str> = tokens_opus.iter().map(|t| t.email.as_str()).collect();
    // 期望顺序: Ultra(高配额) > Ultra(低配额) > Pro(高配额) > Pro(低配额) > Free
    assert_eq!(
        emails_opus,
        vec!["ultra_high@test.com", "ultra_low@test.com", "pro_high@test.com", "pro_low@test.com", "free@test.com"],
        "Opus 4.6 should sort Ultra first, then by quota within each tier"
    );

    // 普通模型 (Sonnet) 排序
    let mut tokens_sonnet = vec![pro_high.clone(), free.clone(), ultra_low.clone(), pro_low.clone(), ultra_high.clone()];
    sort_tokens_for_model(&mut tokens_sonnet, "claude-sonnet-4-5");

    let emails_sonnet: Vec<&str> = tokens_sonnet.iter().map(|t| t.email.as_str()).collect();
    // 期望顺序: Free(100%) > Pro(90%) > Ultra(80%) > Pro(30%) > Ultra(20%) - 按配额优先
    assert_eq!(
        emails_sonnet,
        vec!["free@test.com", "pro_high@test.com", "ultra_high@test.com", "pro_low@test.com", "ultra_low@test.com"],
        "Sonnet should sort by quota first, then by tier as tiebreaker"
    );
}
