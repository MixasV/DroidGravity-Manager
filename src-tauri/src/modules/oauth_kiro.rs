use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;

// Kiro OAuth Configuration
const KIRO_API_URL: &str = "https://app.kiro.dev";

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTokenRequest {
    pub code: String,
    pub code_verifier: String,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KiroTokenResponse {
    #[serde(rename = "accessToken")]
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

/// Initiate Kiro OAuth login - generate Kiro signin URL (как в оригинальном клиенте)
pub async fn initiate_login(redirect_uri: &str, _auth_provider: Option<&str>) -> Result<(String, String, String), String> {
    // Generate PKCE
    let (code_verifier, code_challenge) = generate_pkce();
    let state = uuid::Uuid::new_v4().to_string();
    
    // Build Kiro signin URL (как в оригинальном Kiro клиенте)
    let auth_url = format!(
        "https://app.kiro.dev/signin?state={}&code_challenge={}&code_challenge_method=S256&redirect_uri={}&redirect_from=KiroIDE",
        state,
        code_challenge,
        urlencoding::encode(redirect_uri)
    );
    
    crate::modules::logger::log_info(&format!(
        "Generated Kiro signin URL (state: {}, challenge: {}...)",
        &state[..8],
        &code_challenge[..16]
    ));
    
    Ok((auth_url, code_verifier, state))
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
    
    let response = client
        .post(format!("{}/service/KiroWebPortalService/GetToken", KIRO_API_URL))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("GetToken request failed: {}", e))?;
    
    if response.status().is_success() {
        let tokens: KiroTokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse GetToken response: {}", e))?;
        
        crate::modules::logger::log_info(&format!(
            "Kiro tokens received successfully (expires_in: {}s)",
            tokens.expires_in
        ));
        
        Ok(tokens)
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("GetToken failed: {}", error_text))
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
        .bearer_auth(access_token)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("GetUserInfo request failed: {}", e))?;
    
    if response.status().is_success() {
        let user_info: KiroUserInfo = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse GetUserInfo response: {}", e))?;
        
        crate::modules::logger::log_info(&format!(
            "Kiro user info received: {} ({})",
            user_info.email,
            user_info.status
        ));
        
        Ok(user_info)
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("GetUserInfo failed: {}", error_text))
    }
}

/// Refresh Kiro access token
pub async fn refresh_access_token(refresh_token: &str) -> Result<KiroTokenResponse, String> {
    let client = crate::utils::http::create_client(15);
    
    crate::modules::logger::log_info("Refreshing Kiro access token...");
    
    // Kiro uses the same GetToken endpoint with refresh_token
    let mut request_body = serde_json::Map::new();
    request_body.insert("refresh_token".to_string(), serde_json::Value::String(refresh_token.to_string()));
    
    let response = client
        .post(format!("{}/service/KiroWebPortalService/GetToken", KIRO_API_URL))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Token refresh request failed: {}", e))?;
    
    if response.status().is_success() {
        let tokens: KiroTokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse refresh response: {}", e))?;
        
        crate::modules::logger::log_info("Kiro token refreshed successfully");
        Ok(tokens)
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Token refresh failed: {}", error_text))
    }
}
