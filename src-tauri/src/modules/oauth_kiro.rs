use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use rand::Rng;

// Kiro OAuth Configuration
const KIRO_API_URL: &str = "https://app.kiro.dev";
const KIRO_COGNITO_DOMAIN: &str = "https://kiro-prod.auth.us-east-1.amazoncognito.com";
const KIRO_CLIENT_ID: &str = "59bd15eh40ee7pc20h0bkcu7id";

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

/// Initiate Kiro OAuth login - generate correct Cognito URL
pub async fn initiate_login(redirect_uri: &str, _auth_provider: Option<&str>) -> Result<(String, String, String), String> {
    // Generate PKCE
    let (code_verifier, code_challenge) = generate_pkce();
    let state = uuid::Uuid::new_v4().to_string();

    // Build Kiro signin URL (like KiroIDE does) - NOT direct Cognito URL
    let kiro_signin_url = "https://app.kiro.dev/signin";

    // Use simple redirect_uri without /oauth/callback?login_option=google
    let simple_redirect_uri = redirect_uri; // Just http://localhost:3128

    let auth_url = format!(
        "{}?state={}&code_challenge={}&code_challenge_method=S256&redirect_uri={}&redirect_from=KiroIDE",
        kiro_signin_url,
        state,
        code_challenge,
        urlencoding::encode(simple_redirect_uri)
    );

    crate::modules::logger::log_info(&format!(
        "Generated Kiro signin URL (like KiroIDE): state={}, challenge={}...",
        &state[..8],
        &code_challenge[..16]
    ));

    Ok((auth_url, code_verifier, state))
}


/// Exchange authorization code for tokens (with fallback to manual input)
pub async fn exchange_code(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<KiroTokenResponse, String> {
    let client = crate::utils::http::create_client(15);

    crate::modules::logger::log_info(&format!(
        "=== KIRO TOKEN EXCHANGE ===\nCode: {}...\nCode Verifier: {}...\nRedirect URI: {}",
        &code[..code.len().min(20)],
        &code_verifier[..code_verifier.len().min(20)],
        redirect_uri
    ));

    // Use Kiro API endpoint for token exchange (not Cognito directly)
    let kiro_api_url = "https://app.kiro.dev/api/v1";
    let full_redirect_uri = format!("{}/oauth/callback?login_option=google", redirect_uri);

    let token_request = GetTokenRequest {
        code: code.to_string(),
        code_verifier: code_verifier.to_string(),
        redirect_uri: full_redirect_uri,
    };

    crate::modules::logger::log_info("Attempting Kiro API token exchange...");

    let response = client
        .post(format!("{}/GetToken", kiro_api_url))
        .header("Content-Type", "application/json")
        .json(&token_request)
        .send()
        .await;

    match response {
        Ok(response) => {
            let status = response.status();
            let response_text = response.text().await.unwrap_or_default();

            crate::modules::logger::log_info(&format!(
                "Kiro API response: status={}, body={}",
                status,
                &response_text[..response_text.len().min(500)]
            ));

            if status.is_success() {
                // Try to parse Kiro token response
                match serde_json::from_str::<KiroTokenResponse>(&response_text) {
                    Ok(token_response) => {
                        crate::modules::logger::log_info("SUCCESS! Got tokens from Kiro API");
                        return Ok(token_response);
                    }
                    Err(e) => {
                        crate::modules::logger::log_error(&format!("Failed to parse token response: {}", e));

                        // Try to parse as generic JSON to see what we got
                        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
                            if let Some(access_token) = json_value.get("accessToken").and_then(|v| v.as_str()) {
                                crate::modules::logger::log_info("Found accessToken in response, converting format");

                                return Ok(KiroTokenResponse {
                                    access_token: access_token.to_string(),
                                    refresh_token: json_value.get("refreshToken")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    expires_in: json_value.get("expiresIn")
                                        .and_then(|v| v.as_i64())
                                        .unwrap_or(3600),
                                    profile_arn: json_value.get("profileArn")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("arn:aws:codewhisperer:us-east-1:699475941385:profile/KIRO")
                                        .to_string(),
                                });
                            }
                        }
                    }
                }
            } else {
                crate::modules::logger::log_error(&format!("Kiro API returned error: {}", response_text));
            }
        }
        Err(e) => {
            crate::modules::logger::log_error(&format!("Kiro API request failed: {}", e));
        }
    }

    // API failed, return error with detailed instructions for manual token input
    crate::modules::logger::log_error("Kiro API token exchange failed - using manual token input flow");
    
    Err(format!(
        "⚠️  АВТОМАТИЧЕСКИЙ ОБМЕН ТОКЕНОВ НЕ УДАЛСЯ\n\
        \n\
        ✅ OAuth авторизация прошла успешно!\n\
        ✅ Получен authorization code: {}\n\
        ❌ Kiro API требует специальной подписи для обмена токенов\n\
        \n\
        📋 ИСПОЛЬЗУЙТЕ РУЧНОЙ ВВОД ТОКЕНОВ:\n\
        1. Откройте DevTools в браузере (F12)\n\
        2. Перейдите на вкладку Network (Сеть)\n\
        3. Обновите страницу или повторите авторизацию\n\
        4. Найдите запрос 'GetToken' с токенами в ответе\n\
        5. Скопируйте accessToken и refreshToken\n\
        6. Используйте кнопку 'Manual Token Input' в менеджере\n\
        \n\
        🔑 ДАННЫЕ ДЛЯ ОТЛАДКИ:\n\
        Authorization code: {}\n\
        Code verifier: {}\n\
        Redirect URI: http://localhost:3128/oauth/callback?login_option=google\n\
        \n\
        💡 АЛЬТЕРНАТИВА: Проверьте Local Storage браузера на app.kiro.dev\n\
        Ищите ключи: kiro_access_token, kiro_refresh_token",
        &code[..code.len().min(50)],
        code,
        code_verifier
    ))
}


/// Get user information using Web Portal API
pub async fn get_user_info(access_token: &str) -> Result<KiroUserInfo, String> {
    let client = crate::utils::http::create_client(15);
    
    crate::modules::logger::log_info("Fetching Kiro user info via Web Portal API (CBOR)...");
    
    // Use Web Portal API endpoint with CBOR format
    // POST https://app.kiro.dev/service/KiroWebPortalService/operation/GetUserInfo
    // Content-Type: application/cbor
    // Body: CBOR encoded {"origin": "KIRO_IDE"}
    
    let request_body = serde_json::json!({
        "origin": "KIRO_IDE"
    });
    
    // Encode to CBOR
    let mut cbor_body = Vec::new();
    ciborium::ser::into_writer(&request_body, &mut cbor_body)
        .map_err(|e| format!("Failed to encode CBOR: {}", e))?;
    
    let response = client
        .post(format!("{}/service/KiroWebPortalService/operation/GetUserInfo", KIRO_API_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/cbor")
        .header("Accept", "application/cbor")
        .header("smithy-protocol", "rpc-v2-cbor")
        .header("User-Agent", "DroidGravity-Manager/2.0.2")
        .header("Cookie", format!("AccessToken={}; Idp=Google", access_token))  // ВАЖНО! Web Portal API требует Cookie с Idp
        .body(cbor_body)
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("GetUserInfo request failed: {}", e);
            crate::modules::logger::log_error(&error_msg);
            error_msg
        })?;
    
    let status = response.status();
    let response_bytes = response.bytes().await.unwrap_or_default();
    
    crate::modules::logger::log_info(&format!(
        "GetUserInfo response: status={}, bytes={}",
        status,
        response_bytes.len()
    ));
    
    if !status.is_success() {
        let error_msg = format!("GetUserInfo failed with status {}", status);
        crate::modules::logger::log_error(&error_msg);
        return Err(error_msg);
    }
    
    // Decode CBOR response
    let json_value: serde_json::Value = ciborium::de::from_reader(&response_bytes[..])
        .map_err(|e| {
            let error_msg = format!("Failed to decode CBOR response: {}", e);
            crate::modules::logger::log_error(&error_msg);
            error_msg
        })?;
    
    crate::modules::logger::log_info(&format!("Parsing user info from CBOR: {:?}", json_value));
    
    // Extract email from various possible locations
    let email = json_value.get("email")
        .or_else(|| json_value.get("userInfo").and_then(|v| v.get("email")))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            let error_msg = "No email found in response".to_string();
            crate::modules::logger::log_error(&error_msg);
            error_msg
        })?;
    
    let user_id = json_value.get("userId")
        .or_else(|| json_value.get("userInfo").and_then(|v| v.get("userId")))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    
    let user_info = KiroUserInfo {
        email: email.to_string(),
        user_id,
        idp: "Google".to_string(),
        status: "Active".to_string(),
        feature_flags: std::collections::HashMap::new(),
    };
    
    crate::modules::logger::log_info(&format!("✅ Kiro user info: {}", user_info.email));
    Ok(user_info)
}

/// Manual token input for testing (temporary solution)
pub async fn manual_token_input(
    access_token: &str,
    refresh_token: Option<&str>,
    expires_in: Option<i64>,
    profile_arn: Option<&str>,
) -> Result<KiroTokenResponse, String> {
    crate::modules::logger::log_info("Using manually provided Kiro tokens");
    
    // DO NOT clean tokens - Kiro uses format "token:signature" and both parts are needed!
    // From captured traffic: Bearer aoaAAAAAGmXdh0VsKaWo3YbJXAryPQo....:MGUCMA...
    
    // Profile ARN MUST be provided or we'll get 403 "User is not authorized"
    // Profile ARN is returned in GetToken response during OAuth flow
    // For manual input, user must provide it or we use a default (which may not work)
    let arn = match profile_arn {
        Some(arn) if !arn.is_empty() => arn.to_string(),
        _ => {
            crate::modules::logger::log_warn(
                "⚠️  No Profile ARN provided! Using default ARN which may not work.\n\
                Profile ARN should be obtained from GetToken response or browser DevTools."
            );
            // Default ARN for stofelsusana@gmail.com account (may not work for other accounts!)
            "arn:aws:codewhisperer:us-east-1:699475941385:profile/EHGA3GRVQMUK".to_string()
        }
    };
    
    let tokens = KiroTokenResponse {
        access_token: access_token.to_string(),
        refresh_token: refresh_token.unwrap_or("").to_string(),
        expires_in: expires_in.unwrap_or(3600),
        profile_arn: arn.clone(),
    };
    
    crate::modules::logger::log_info(&format!(
        "Manual tokens configured: access_token={}..., refresh_token={}..., expires_in={}s, profile_arn={}",
        &tokens.access_token[..tokens.access_token.len().min(20)],
        &tokens.refresh_token[..tokens.refresh_token.len().min(20)],
        tokens.expires_in,
        &tokens.profile_arn
    ));
    
    Ok(tokens)
}

/// Get Kiro user usage and limits via Web Portal API
pub async fn get_user_usage_and_limits(access_token: &str) -> Result<serde_json::Value, String> {
    let client = crate::utils::http::create_client(15);
    
    crate::modules::logger::log_info("Fetching Kiro usage and limits via Web Portal API (CBOR)...");
    
    // Use Web Portal API endpoint with CBOR format
    // POST https://app.kiro.dev/service/KiroWebPortalService/operation/GetUserUsageAndLimits
    // Content-Type: application/cbor
    // Body: CBOR encoded {"origin": "KIRO_IDE", "isEmailRequired": false}
    
    let request_body = serde_json::json!({
        "origin": "KIRO_IDE",
        "isEmailRequired": false
    });
    
    // Encode to CBOR
    let mut cbor_body = Vec::new();
    ciborium::ser::into_writer(&request_body, &mut cbor_body)
        .map_err(|e| format!("Failed to encode CBOR: {}", e))?;
    
    let response = client
        .post(format!("{}/service/KiroWebPortalService/operation/GetUserUsageAndLimits", KIRO_API_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/cbor")
        .header("Accept", "application/cbor")
        .header("smithy-protocol", "rpc-v2-cbor")
        .header("User-Agent", "DroidGravity-Manager/2.0.2")
        .header("Cookie", format!("AccessToken={}", access_token))  // ВАЖНО! Web Portal API требует Cookie
        .body(cbor_body)
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("GetUserUsageAndLimits request failed: {}", e);
            crate::modules::logger::log_error(&error_msg);
            error_msg
        })?;
    
    let status = response.status();
    let response_bytes = response.bytes().await.unwrap_or_default();
    
    crate::modules::logger::log_info(&format!(
        "GetUserUsageAndLimits response: status={}, bytes={}",
        status,
        response_bytes.len()
    ));
    
    if !status.is_success() {
        let error_msg = format!(
            "GetUserUsageAndLimits failed with status {}",
            status
        );
        crate::modules::logger::log_error(&error_msg);
        return Err(error_msg);
    }
    
    // Decode CBOR response
    // First, try to log raw bytes for debugging
    crate::modules::logger::log_info(&format!(
        "CBOR response bytes (first 100): {:?}",
        &response_bytes[..response_bytes.len().min(100)]
    ));
    
    // Try to decode CBOR as raw Value first, then convert to JSON
    let json_value: serde_json::Value = match ciborium::de::from_reader::<ciborium::Value, _>(&response_bytes[..]) {
        Ok(cbor_value) => {
            // Convert CBOR Value to JSON Value
            match cbor_to_json(cbor_value) {
                Ok(json) => json,
                Err(e) => {
                    let error_msg = format!("Failed to convert CBOR to JSON: {}. Response bytes length: {}", e, response_bytes.len());
                    crate::modules::logger::log_error(&error_msg);
                    
                    // Try to decode as plain JSON as fallback
                    if let Ok(text) = String::from_utf8(response_bytes.to_vec()) {
                        crate::modules::logger::log_info(&format!("Response as text: {}", &text[..text.len().min(500)]));
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                            crate::modules::logger::log_info("Successfully parsed as JSON instead of CBOR");
                            json
                        } else {
                            return Err(error_msg);
                        }
                    } else {
                        return Err(error_msg);
                    }
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to decode CBOR response: {}. Response bytes length: {}", e, response_bytes.len());
            crate::modules::logger::log_error(&error_msg);
            
            // Try to decode as plain JSON as fallback
            if let Ok(text) = String::from_utf8(response_bytes.to_vec()) {
                crate::modules::logger::log_info(&format!("Response as text: {}", &text[..text.len().min(500)]));
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    crate::modules::logger::log_info("Successfully parsed as JSON instead of CBOR");
                    json
                } else {
                    return Err(error_msg);
                }
            } else {
                return Err(error_msg);
            }
        }
    };
    
    crate::modules::logger::log_info("✅ Successfully parsed usage and limits from CBOR");
    Ok(json_value)
}

/// Convert CBOR Value to JSON Value
fn cbor_to_json(cbor: ciborium::Value) -> Result<serde_json::Value, String> {
    match cbor {
        ciborium::Value::Null => Ok(serde_json::Value::Null),
        ciborium::Value::Bool(b) => Ok(serde_json::Value::Bool(b)),
        ciborium::Value::Integer(i) => {
            if let Ok(n) = i128::try_from(i) {
                if let Ok(n64) = i64::try_from(n) {
                    Ok(serde_json::json!(n64))
                } else {
                    Ok(serde_json::json!(n.to_string()))
                }
            } else {
                Err("Integer too large".to_string())
            }
        }
        ciborium::Value::Float(f) => Ok(serde_json::json!(f)),
        ciborium::Value::Bytes(b) => {
            // Try to decode as UTF-8 string, otherwise base64
            if let Ok(s) = String::from_utf8(b.clone()) {
                Ok(serde_json::Value::String(s))
            } else {
                Ok(serde_json::Value::String(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &b)))
            }
        }
        ciborium::Value::Text(s) => Ok(serde_json::Value::String(s)),
        ciborium::Value::Array(arr) => {
            let json_arr: Result<Vec<_>, _> = arr.into_iter().map(cbor_to_json).collect();
            Ok(serde_json::Value::Array(json_arr?))
        }
        ciborium::Value::Map(map) => {
            let mut json_map = serde_json::Map::new();
            for (k, v) in map {
                let key = match k {
                    ciborium::Value::Text(s) => s,
                    ciborium::Value::Integer(i) => i.to_string(),
                    _ => return Err("Map key must be string or integer".to_string()),
                };
                json_map.insert(key, cbor_to_json(v)?);
            }
            Ok(serde_json::Value::Object(json_map))
        }
        ciborium::Value::Tag(tag, value) => {
            // Handle CBOR tags (like datetime)
            match tag {
                0 => {
                    // DateTime string (RFC3339)
                    cbor_to_json(*value)
                }
                1 => {
                    // DateTime timestamp (Unix epoch)
                    if let ciborium::Value::Integer(i) = *value {
                        if let Ok(timestamp) = i64::try_from(i) {
                            Ok(serde_json::json!(timestamp))
                        } else {
                            cbor_to_json(ciborium::Value::Integer(i))
                        }
                    } else {
                        cbor_to_json(*value)
                    }
                }
                _ => {
                    // Unknown tag, just convert the value
                    cbor_to_json(*value)
                }
            }
        }
        _ => Err(format!("Unsupported CBOR type: {:?}", cbor)),
    }
}

/// Get Kiro user balance/credits information (deprecated - use get_user_usage_and_limits)
pub async fn get_user_balance(access_token: &str) -> Result<serde_json::Value, String> {
    // Redirect to new function
    get_user_usage_and_limits(access_token).await
}

/// Refresh Kiro access token using AWS Cognito
pub async fn refresh_access_token(refresh_token: &str) -> Result<KiroTokenResponse, String> {
    crate::modules::logger::log_info("Refreshing Kiro access token via AWS Cognito...");
    
    let client = crate::utils::http::create_client(15);
    
    // AWS Cognito configuration for Kiro
    let cognito_domain = "https://kiro-prod-us-east-1.auth.us-east-1.amazoncognito.com";
    let client_id = "59bd15eh40ee7pc20h0bkcu7id";
    
    // DO NOT clean token - Kiro uses format "token:signature"
    crate::modules::logger::log_info(&format!(
        "Using refresh token: {}... (len: {})",
        &refresh_token[..refresh_token.len().min(20)],
        refresh_token.len()
    ));
    
    // Prepare form data for token refresh
    let form_data = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", client_id),
    ];
    
    let response = client
        .post(format!("{}/oauth2/token", cognito_domain))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("User-Agent", "DroidGravity-Manager/2.0.2")
        .form(&form_data)
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("Cognito token refresh request failed: {}", e);
            crate::modules::logger::log_error(&error_msg);
            error_msg
        })?;
    
    let status = response.status();
    let response_text = response.text().await.unwrap_or_default();
    
    crate::modules::logger::log_info(&format!(
        "Cognito refresh response: status={}, body={}",
        status,
        &response_text[..response_text.len().min(500)]
    ));
    
    if !status.is_success() {
        let error_msg = format!(
            "Cognito token refresh failed with status {}: {}",
            status,
            &response_text[..response_text.len().min(200)]
        );
        crate::modules::logger::log_error(&error_msg);
        return Err(error_msg);
    }
    
    // Parse response
    let token_response: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| {
            let error_msg = format!("Failed to parse Cognito response: {}", e);
            crate::modules::logger::log_error(&error_msg);
            error_msg
        })?;
    
    let new_access_token = token_response
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            let error_msg = "No access_token in Cognito response".to_string();
            crate::modules::logger::log_error(&error_msg);
            error_msg
        })?
        .to_string();
    
    let expires_in = token_response
        .get("expires_in")
        .and_then(|v| v.as_i64())
        .unwrap_or(3600);
    
    // New refresh token (if provided, otherwise use the old one)
    let new_refresh_token = token_response
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| refresh_token.to_string());
    
    crate::modules::logger::log_info(&format!(
        "✅ Token refreshed successfully: access_token={}..., expires_in={}s",
        &new_access_token[..new_access_token.len().min(20)],
        expires_in
    ));
    
    // Use a default profile ARN for Kiro (we don't need to fetch user info just for refresh)
    let profile_arn = "arn:aws:codewhisperer:us-east-1:699475941385:profile/KIRO".to_string();
    
    Ok(KiroTokenResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        expires_in,
        profile_arn,
    })
}
