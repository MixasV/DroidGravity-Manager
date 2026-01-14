use serde_json::Value;

/// Get project_id using Antigravity's loadCodeAssist API
pub async fn fetch_project_id(access_token: &str) -> Result<String, String> {
    let url = "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist";
    
    let request_body = serde_json::json!({
        "metadata": {
            "ideType": "ANTIGRAVITY"
        }
    });
    
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .bearer_auth(access_token)
        .header("Host", "cloudcode-pa.googleapis.com")
        .header("User-Agent", "antigravity/1.11.9 windows/amd64")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("loadCodeAssist request failed: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("loadCodeAssist error {}: {}", status, body));
    }
    
    let data: Value = response.json()
        .await
        .map_err(|e| format!("Parse response failed: {}", e))?;
    
    // Extract cloudaicompanionProject
    if let Some(project_id) = data.get("cloudaicompanionProject")
        .and_then(|v| v.as_str()) {
        return Ok(project_id.to_string());
    }
    
    // Fallback to mock project_id
    let mock_id = generate_mock_project_id();
    tracing::warn!("Account has no cloudaicompanionProject, using mock: {}", mock_id);
    Ok(mock_id)
}

/// Generate random project_id (fallback when API doesn't return one)
/// Format: {adjective}-{noun}-{5 random chars}
pub fn generate_mock_project_id() -> String {
    use rand::Rng;
    
    let adjectives = ["useful", "bright", "swift", "calm", "bold"];
    let nouns = ["fuze", "wave", "spark", "flow", "core"];
    
    let mut rng = rand::thread_rng();
    let adj = adjectives[rng.gen_range(0..adjectives.len())];
    let noun = nouns[rng.gen_range(0..nouns.len())];
    
    // Generate 5 random base36 characters
    let random_num: String = (0..5)
        .map(|_| {
            let chars = "abcdefghijklmnopqrstuvwxyz0123456789";
            let idx = rng.gen_range(0..chars.len());
            chars.chars().nth(idx).unwrap()
        })
        .collect();
    
    format!("{}-{}-{}", adj, noun, random_num)
}
