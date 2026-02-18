use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;

// Kiro OAuth Configuration
const KIRO_API_URL: &str = "https://app.kiro.dev";

#[derive(Debug, Serialize, Deserialize)]
pub struct InitiateLoginRequest {
    pub idp: String,
    #[serde(rename = "redirectUri")]
    pub redirect_uri: String,
    pub state: String,
    #[serde(rename = "codeChallenge")]
    pub code_challenge: String,
    #[serde(rename = "codeChallengeMethod")]
    pub code_challenge_method: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitiateLoginResponse {
    #[serde(rename = "redirectUrl")]
    pub redirect_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTokenRequest {
    pub code: String,
    pub code_verifier: String,
    pub redirect_uri: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct KiroTokenResponse {
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: i64,
    #[serde(rename = "profileArn")]
    pub profile_arn: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserInfoRequest {
    pub origin: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KiroUserInfo {
    pub email: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub idp: String,
    pub status: String,
    #[serde(rename = "featureFlags", default)]
    pub feature_flags: std::collections::HashMap<String, bool>,
}

/// Generate PKCE code_verifier and code_challenge
pub fn generate_pkce() -> (String, String) {
    // Generate code_verifier (128 random characters)
    let mut rng = rand::thread_rng();
    let verifier: String = (0..128)
        .map(|_| {
            let idx = rng.gen_range(0..62);
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
                .chars()
                .nth(idx)
                .unwrap()
        })
        .collect();
    
    // Generate code_challenge = BASE64URL(SHA256(verifier))
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    let challenge = general_purpose::URL_SAFE_NO_PAD.encode(hash);
    
    (verifier, challenge)
}

/// Initiate Kiro OAuth login - вызывает InitiateLogin API для получения redirectUrl
pub async fn initiate_login(redirect_uri: &str, auth_provider: Option<&str>) -> Result<(String, String, String), String> {
    // Generate PKCE
    let (code_verifier, code_challenge) = generate_pkce();
    let state = uuid::Uuid::new_v4().to_string();
    
    // Определяем провайдера (по умолчанию Google)
    let idp = match auth_provider.unwrap_or("google") {
        "github" => "GitHub",
        "google" | _ => "Google",
    };
    
    // Формируем redirect_uri с login_option как в оригинальном клиенте
    let redirect_uri_with_option = format!("{}/oauth/callback?login_option={}", 
        redirect_uri.trim_end_matches('/'), 
        auth_provider.unwrap_or("google")
    );
    
    let client = crate::utils::http::create_client(15);
    
    let request = InitiateLoginRequest {
        idp: idp.to_string(),
        redirect_uri: redirect_uri_with_option,
        state: state.clone(),
        code_challenge: code_challenge.clone(),
        code_challenge_method: "S256".to_string(),
    };
    
    crate::modules::logger::log_info(&format!(
        "Calling InitiateLogin API (state: {}, challenge: {}..., idp: {})",
        &state[..8],
        &code_challenge[..16],
        idp
    ));
    
    let response = client
        .post(format!("{}/service/KiroWebPortalService/InitiateLogin", KIRO_API_URL))
        .header("Content-Type", "application/x-amz-json-1.1")
        .header("X-Amz-Target", "KiroWebPortalService.InitiateLogin")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("InitiateLogin request failed: {}", e))?;
    
    let status = response.status();
    let response_text = response.text().await.unwrap_or_default();
    
    crate::modules::logger::log_info(&format!(
        "InitiateLogin response: status={}, body={}",
        status,
        &response_text[..response_text.len().min(500)]
    ));
    
    if status.is_success() {
        let init_response: InitiateLoginResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse InitiateLogin response: {} (response: {})", e, &response_text[..response_text.len().min(200)]))?;
        
        crate::modules::logger::log_info(&format!(
            "Received redirectUrl: {}",
            &init_response.redirect_url[..init_response.redirect_url.len().min(100)]
        ));
        
        Ok((init_response.redirect_url, code_verifier, state))
    } else {
        Err(format!("InitiateLogin failed with status {}: {}", status, response_text))
    }
}

/// Exchange authorization code for tokens
pub async fn exchange_code(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<KiroTokenResponse, String> {
    let client = crate::utils::http::create_client(15);
    
    let request = GetTokenRequest {
        code: code.to_string(),
        code_verifier: code_verifier.to_string(),
        redirect_uri: redirect_uri.to_string(),
    };
    
    crate::modules::logger::log_info("Exchanging authorization code for Kiro tokens...");
    
    // Попробуем endpoint GetToken
    let response = client
        .post(format!("{}/service/KiroWebPortalService/GetToken", KIRO_API_URL))
        .header("Content-Type", "application/x-amz-json-1.1")
        .header("X-Amz-Target", "KiroWebPortalService.GetToken")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("GetToken request failed: {}", e))?;
    
    let status = response.status();
    let response_text = response.text().await.unwrap_or_default();
    
    crate::modules::logger::log_info(&format!(
        "GetToken response: status={}, body={}",
        status,
        &response_text[..response_text.len().min(500)]
    ));
    
    if status.is_success() {
        let tokens: KiroTokenResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse GetToken response: {} (response: {})", e, &response_text[..response_text.len().min(200)]))?;
        
        crate::modules::logger::log_info(&format!(
            "Kiro tokens received successfully (expires_in: {}s)",
            tokens.expires_in
        ));
        
        Ok(tokens)
    } else {
        Err(format!("GetToken failed with status {}: {}", status, response_text))
    }
}

/// Get user information
pub async fn get_user_info(access_token: &str) -> Result<KiroUserInfo, String> {
    let client = crate::utils::http::create_client(15);
    
    let request = GetUserInfoRequest {
        origin: "KIRO_IDE".to_string(),
    };
    
    crate::modules::logger::log_info("Fetching Kiro user info...");
    
    let response = client
        .post(format!("{}/service/KiroWebPortalService/GetUserInfo", KIRO_API_URL))
        .header("Content-Type", "application/x-amz-json-1.1")
        .header("X-Amz-Target", "KiroWebPortalService.GetUserInfo")
        .bearer_auth(access_token)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("GetUserInfo request failed: {}", e))?;
    
    let status = response.status();
    let response_text = response.text().await.unwrap_or_default();
    
    crate::modules::logger::log_info(&format!(
        "GetUserInfo response: status={}, body={}",
        status,
        &response_text[..response_text.len().min(500)]
    ));
    
    if status.is_success() {
        let user_info: KiroUserInfo = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse GetUserInfo response: {} (response: {})", e, &response_text[..response_text.len().min(200)]))?;
        
        crate::modules::logger::log_info(&format!(
            "Kiro user info received: {} ({})",
            user_info.email,
            user_info.status
        ));
        
        Ok(user_info)
    } else {
        Err(format!("GetUserInfo failed with status {}: {}", status, response_text))
    }
}

/// Refresh Kiro access token
pub async fn refresh_access_token(refresh_token: &str) -> Result<KiroTokenResponse, String> {
    let client = crate::utils::http::create_client(15);
    
    crate::modules::logger::log_info("Refreshing Kiro access token...");
    
    // Kiro uses the same endpoint with refresh_token
    let mut request_body = serde_json::Map::new();
    request_body.insert("refresh_token".to_string(), serde_json::Value::String(refresh_token.to_string()));
    
    let response = client
        .post(format!("{}/service/KiroWebPortalService/GetToken", KIRO_API_URL))
        .header("Content-Type", "application/x-amz-json-1.1")
        .header("X-Amz-Target", "KiroWebPortalService.GetToken")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Token refresh request failed: {}", e))?;
    
    let status = response.status();
    let response_text = response.text().await.unwrap_or_default();
    
    crate::modules::logger::log_info(&format!(
        "Token refresh response: status={}, body={}",
        status,
        &response_text[..response_text.len().min(500)]
    ));
    
    if status.is_success() {
        let tokens: KiroTokenResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse refresh response: {} (response: {})", e, &response_text[..response_text.len().min(200)]))?;
        
        crate::modules::logger::log_info("Kiro token refreshed successfully");
        Ok(tokens)
    } else {
        Err(format!("Token refresh failed with status {}: {}", status, response_text))
    }
}
