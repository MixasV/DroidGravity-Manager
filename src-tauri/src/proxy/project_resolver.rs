use serde_json::Value;
use reqwest;

/// 使用 Antigravity 的 loadCodeAssist API 获取 project_id
/// 这是获取 cloudaicompanionProject 的正确方式
pub async fn fetch_project_id(access_token: &str) -> Result<String, String> {
    // 使用 Prod 环境， Sandbox 不稳定且容易返回 403/429
    let url = "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist";
    
    let request_body = serde_json::json!({
        "metadata": {
            "ideType": "ANTIGRAVITY"
        }
    });
    
    let client = crate::utils::http::get_client();
    let response: reqwest::Response = client
        .post(url)
        .bearer_auth(access_token)
        .header("Host", "cloudcode-pa.googleapis.com")
        .header("User-Agent", crate::constants::USER_AGENT.as_str())
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e: reqwest::Error| format!("loadCodeAssist 请求失败: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("loadCodeAssist 返回错误 {}: {}", status, body));
    }
    
    let data: Value = response.json::<Value>()
        .await
        .map_err(|e: reqwest::Error| format!("解析响应失败: {}", e))?;
    
    // 提取 cloudaicompanionProject
    if let Some(project_id) = data.get("cloudaicompanionProject")
        .and_then(|v| v.as_str()) {
        return Ok(project_id.to_string());
    }
    
    // 如果没有返回 project_id，说明账号无资格，返回错误以触发 token_manager 的稳定兜底逻辑
    Err("账号无资格获取官方 cloudaicompanionProject".to_string())
}
