use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::models::QuotaData;

const QUOTA_API_URL: &str = "https://cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels";

#[derive(Debug, Serialize, Deserialize)]
struct QuotaResponse {
    models: std::collections::HashMap<String, ModelInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelInfo {
    #[serde(rename = "quotaInfo")]
    quota_info: Option<QuotaInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QuotaInfo {
    #[serde(rename = "remainingFraction")]
    remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LoadProjectResponse {
    #[serde(rename = "cloudaicompanionProject")]
    project_id: Option<String>,
    #[serde(rename = "currentTier")]
    current_tier: Option<Tier>,
    #[serde(rename = "paidTier")]
    paid_tier: Option<Tier>,
}

#[derive(Debug, Deserialize)]
struct Tier {
    id: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "quotaTier")]
    quota_tier: Option<String>,
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
    slug: Option<String>,
}

/// 创建配置好的 HTTP Client
fn create_client() -> reqwest::Client {
    crate::utils::http::create_client(15)
}

const CLOUD_CODE_BASE_URL: &str = "https://cloudcode-pa.googleapis.com";

/// 获取项目 ID 和订阅类型
async fn fetch_project_id(access_token: &str, email: &str) -> (Option<String>, Option<String>) {
    let client = create_client();
    let meta = json!({"metadata": {"ideType": "ANTIGRAVITY"}});

    let res = client
        .post(format!("{}/v1internal:loadCodeAssist", CLOUD_CODE_BASE_URL))
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", access_token))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::USER_AGENT, crate::constants::USER_AGENT.as_str())
        .json(&meta)
        .send()
        .await;

    match res {
        Ok(res) => {
            if res.status().is_success() {
                if let Ok(data) = res.json::<LoadProjectResponse>().await {
                    let project_id = data.project_id.clone();
                    
                    // 核心逻辑：优先从 paid_tier 获取订阅 ID，这比 current_tier 更能反映真实账户权益
                    let subscription_tier = data.paid_tier
                        .and_then(|t| t.id)
                        .or_else(|| data.current_tier.and_then(|t| t.id));
                    
                    if let Some(ref tier) = subscription_tier {
                        crate::modules::logger::log_info(&format!(
                            "📊 [{}] 订阅识别成功: {}", email, tier
                        ));
                    }
                    
                    return (project_id, subscription_tier);
                }
            } else {
                crate::modules::logger::log_warn(&format!(
                    "⚠️  [{}] loadCodeAssist 失败: Status: {}", email, res.status()
                ));
            }
        }
        Err(e) => {
            crate::modules::logger::log_error(&format!("❌ [{}] loadCodeAssist 网络错误: {}", email, e));
        }
    }
    
    (None, None)
}

/// 预热模型
pub async fn warmup_model_directly(_token: &str, _model: &str, _pid: &str, _email: &str, _pct: i32) -> Result<bool, String> {
    // 简化实现
    Ok(true)
}

/// 获取用于预热的有效 Token
pub async fn get_valid_token_for_warmup(account: &crate::models::Account) -> Result<(String, String), String> {
    let token = crate::modules::oauth::ensure_fresh_token(&account.token).await?;
    let pid = account.token.project_id.clone().unwrap_or_else(|| "bamboo-precept-lgxtn".to_string());
    Ok((token.access_token, pid))
}

/// 带缓存的配额查询
pub async fn fetch_quota_with_cache(access_token: &str, email: &str, _pid: Option<&str>) -> crate::error::AppResult<(QuotaData, Option<String>)> {
    fetch_quota(access_token, email).await
}

/// 查询账号配额的统一入口
pub async fn fetch_quota(access_token: &str, email: &str) -> crate::error::AppResult<(QuotaData, Option<String>)> {
    fetch_quota_inner(access_token, email).await
}

/// 查询账号配额逻辑
pub async fn fetch_quota_inner(access_token: &str, email: &str) -> crate::error::AppResult<(QuotaData, Option<String>)> {
    use crate::error::AppError;
    
    // Detect provider by token format
    // Kiro tokens start with "aoa" (access) or "aor" (refresh)
    let is_kiro = access_token.starts_with("aoa") || access_token.starts_with("aor");
    
    if is_kiro {
        crate::modules::logger::log_info(&format!("[{}] Detected Kiro provider, using Kiro API", email));
        return fetch_kiro_quota(access_token, email).await;
    }
    
    // Default: Gemini provider
    crate::modules::logger::log_info(&format!("[{}] Using Gemini API", email));
    
    // 1. 获取 Project ID 和订阅类型
    let (project_id, subscription_tier) = fetch_project_id(access_token, email).await;
    
    let final_project_id = project_id.as_deref().unwrap_or("bamboo-precept-lgxtn");
    
    let client = create_client();
    let payload = json!({
        "project": final_project_id
    });
    
    let url = QUOTA_API_URL;
    let max_retries = 3;
    let mut last_error: Option<AppError> = None;

    for attempt in 1..=max_retries {
        match client
            .post(url)
            .bearer_auth(access_token)
            .header("User-Agent", crate::constants::USER_AGENT.as_str())
            .json(&json!(payload))
            .send()
            .await
        {
            Ok(response) => {
                // 将 HTTP 错误状态转换为 AppError
                if let Err(_) = response.error_for_status_ref() {
                    let status = response.status();
                    
                    // ✅ 特殊处理 403 Forbidden - 直接返回,不重试
                    if status == reqwest::StatusCode::FORBIDDEN {
                        crate::modules::logger::log_warn(&format!(
                            "账号无权限 (403 Forbidden),标记为 forbidden 状态"
                        ));
                        let mut q = QuotaData::new();
                        q.is_forbidden = true;
                        q.subscription_tier = subscription_tier.clone();
                        return Ok((q, project_id.clone()));
                    }
                    
                    // 其他错误继续重试逻辑
                    if attempt < max_retries {
                         let text = response.text().await.unwrap_or_default();
                         crate::modules::logger::log_warn(&format!("API 错误: {} - {} (尝试 {}/{})", status, text, attempt, max_retries));
                         last_error = Some(AppError::Unknown(format!("HTTP {} - {}", status, text)));
                         tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                         continue;
                    } else {
                         let text = response.text().await.unwrap_or_default();
                         return Err(AppError::Unknown(format!("API 错误: {} - {}", status, text)));
                    }
                }

                let quota_response: QuotaResponse = response
                    .json()
                    .await
                    .map_err(|e| AppError::Network(e))?;
                
                let mut quota_data = QuotaData::new();
                
                // 使用 debug 级别记录详细信息，避免控制台噪音
                tracing::debug!("Quota API 返回了 {} 个模型", quota_response.models.len());

                for (name, info) in quota_response.models {
                    if let Some(quota_info) = info.quota_info {
                        let percentage = quota_info.remaining_fraction
                            .map(|f| (f * 100.0) as i32)
                            .unwrap_or(0);
                        
                        let reset_time = quota_info.reset_time.unwrap_or_default();
                        
                        // 只保存我们关心的模型
                        if name.contains("gemini") || name.contains("claude") {
                            quota_data.add_model(name, percentage, reset_time);
                        }
                    }
                }
                
                // 设置订阅类型
                quota_data.subscription_tier = subscription_tier.clone();
                
                return Ok((quota_data, project_id.clone()));
            },
            Err(e) => {
                crate::modules::logger::log_warn(&format!("请求失败: {} (尝试 {}/{})", e, attempt, max_retries));
                last_error = Some(AppError::Network(e));
                if attempt < max_retries {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| AppError::Unknown("配额查询失败".to_string())))
}

/// Fetch Kiro quota using Web Portal API
async fn fetch_kiro_quota(access_token: &str, email: &str) -> crate::error::AppResult<(QuotaData, Option<String>)> {
    use crate::error::AppError;
    
    crate::modules::logger::log_info(&format!("[{}] Fetching Kiro quota via Web Portal API", email));
    
    // Call Kiro API
    let usage_result = crate::modules::oauth_kiro::get_user_usage_and_limits(access_token).await;
    
    match usage_result {
        Ok(json_value) => {
            crate::modules::logger::log_info(&format!("[{}] Kiro API response received", email));
            
            let mut quota_data = QuotaData::new();
            
            // Parse Kiro response format
            // Expected structure from captured traffic:
            // {
            //   "usageBreakdownList": [
            //     {
            //       "resourceType": "CREDIT",
            //       "currentUsageWithPrecision": 21.72,
            //       "usageLimitWithPrecision": 50.0,
            //       "freeTrialInfo": {
            //         "currentUsageWithPrecision": 21.72,
            //         "usageLimitWithPrecision": 500.0,
            //         "freeTrialStatus": "ACTIVE",
            //         "freeTrialExpiry": 1774129297.746
            //       },
            //       "displayName": "Credit",
            //       ...
            //     }
            //   ],
            //   "subscriptionInfo": {
            //     "subscriptionTitle": "KIRO FREE",
            //     "type": "Q_DEVELOPER_STANDALONE_FREE"
            //   },
            //   "daysUntilReset": 0,
            //   "nextDateReset": 1772323200.0
            // }
            
            if let Some(usage_list) = json_value.get("usageBreakdownList").and_then(|v| v.as_array()) {
                for item in usage_list {
                    let resource_type = item.get("resourceType").and_then(|v| v.as_str()).unwrap_or("");
                    
                    if resource_type == "CREDIT" {
                        // Regular credits (monthly limit)
                        let current_usage = item.get("currentUsageWithPrecision").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let usage_limit = item.get("usageLimitWithPrecision").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        
                        // Free trial credits
                        let mut trial_current = 0.0;
                        let mut trial_limit = 0.0;
                        let mut trial_status = "INACTIVE".to_string();
                        let mut trial_expiry: Option<i64> = None;
                        
                        if let Some(trial_info) = item.get("freeTrialInfo") {
                            trial_current = trial_info.get("currentUsageWithPrecision").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            trial_limit = trial_info.get("usageLimitWithPrecision").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            trial_status = trial_info.get("freeTrialStatus").and_then(|v| v.as_str()).unwrap_or("INACTIVE").to_string();
                            trial_expiry = trial_info.get("freeTrialExpiry").and_then(|v| v.as_f64()).map(|f| f as i64);
                        }
                        
                        // Calculate total available and used
                        let total_limit = usage_limit + trial_limit;
                        let total_used = current_usage + trial_current;
                        let total_remaining = total_limit - total_used;
                        
                        let percentage = if total_limit > 0.0 {
                            ((total_remaining / total_limit) * 100.0) as i32
                        } else {
                            0
                        };
                        
                        // Store as "kiro-credits" model with detailed info
                        let reset_time = if let Some(expiry) = trial_expiry {
                            expiry.to_string()
                        } else {
                            json_value.get("nextDateReset")
                                .and_then(|v| v.as_f64())
                                .map(|f| (f as i64).to_string())
                                .unwrap_or_default()
                        };
                        
                        quota_data.add_model(
                            "kiro-credits".to_string(),
                            percentage,
                            reset_time
                        );
                        
                        // Store Kiro-specific metadata in quota_data
                        // We'll use the models HashMap to store additional info
                        quota_data.add_model(
                            "kiro-monthly-limit".to_string(),
                            ((usage_limit - current_usage) / usage_limit * 100.0) as i32,
                            format!("{:.2}", usage_limit)
                        );
                        
                        quota_data.add_model(
                            "kiro-monthly-used".to_string(),
                            0,
                            format!("{:.2}", current_usage)
                        );
                        
                        quota_data.add_model(
                            "kiro-trial-limit".to_string(),
                            if trial_limit > 0.0 { ((trial_limit - trial_current) / trial_limit * 100.0) as i32 } else { 0 },
                            format!("{:.2}", trial_limit)
                        );
                        
                        quota_data.add_model(
                            "kiro-trial-used".to_string(),
                            0,
                            format!("{:.2}", trial_current)
                        );
                        
                        quota_data.add_model(
                            "kiro-trial-status".to_string(),
                            0,
                            trial_status
                        );
                        
                        crate::modules::logger::log_info(&format!(
                            "[{}] Kiro credits: Total {:.2}/{:.2} (Monthly: {:.2}/{:.2}, Trial: {:.2}/{:.2}, {}%)",
                            email, total_remaining, total_limit, 
                            usage_limit - current_usage, usage_limit,
                            trial_limit - trial_current, trial_limit,
                            percentage
                        ));
                    }
                }
            }
            
            // Extract subscription tier
            if let Some(sub_info) = json_value.get("subscriptionInfo") {
                if let Some(sub_title) = sub_info.get("subscriptionTitle").and_then(|v| v.as_str()) {
                    quota_data.subscription_tier = Some(sub_title.to_string());
                }
            }
            
            Ok((quota_data, None)) // No project_id for Kiro
        }
        Err(e) => {
            crate::modules::logger::log_error(&format!("[{}] Kiro quota fetch failed: {}", email, e));
            Err(AppError::Unknown(format!("Kiro quota fetch failed: {}", e)))
        }
    }
}

/// 批量查询所有账号配额 (备用功能)
#[allow(dead_code)]
pub async fn fetch_all_quotas(accounts: Vec<(String, String)>) -> Vec<(String, crate::error::AppResult<QuotaData>)> {
    let mut results = Vec::new();
    
    for (account_id, access_token) in accounts {
        // 在批量查询中，我们将 account_id 传入以供日志标识
        let result = fetch_quota(&access_token, &account_id).await.map(|(q, _)| q);
        results.push((account_id, result));
    }
    
    results
}
