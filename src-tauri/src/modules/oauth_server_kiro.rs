use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use std::sync::{Mutex, OnceLock};
use std::sync::Arc;
use tauri::Emitter;
use crate::modules::oauth_kiro;

struct KiroOAuthFlowState {
    auth_url: String,
    code_verifier: String,
    state: String,
    redirect_uri: String,
    code_tx: Arc<tokio::sync::Mutex<Option<oneshot::Sender<Result<String, String>>>>>,
    code_rx: Option<oneshot::Receiver<Result<String, String>>>,
}

static KIRO_OAUTH_FLOW_STATE: OnceLock<Mutex<Option<KiroOAuthFlowState>>> = OnceLock::new();

fn get_kiro_oauth_flow_state() -> &'static Mutex<Option<KiroOAuthFlowState>> {
    KIRO_OAUTH_FLOW_STATE.get_or_init(|| Mutex::new(None))
}

fn oauth_success_html() -> &'static str {
    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n\
    <html>\
    <body style='font-family: sans-serif; text-align: center; padding: 50px;'>\
        <h1 style='color: green;'>✅ Kiro Authorization Successful!</h1>\
        <p>You can close this window and return to the app.</p>\
        <script>setTimeout(function() { window.close(); }, 2000);</script>\
    </body>\
    </html>"
}

fn oauth_fail_html() -> &'static str {
    "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\n\r\n\
    <html>\
    <body style='font-family: sans-serif; text-align: center; padding: 50px;'>\
        <h1 style='color: red;'>❌ Kiro Authorization Failed</h1>\
        <p>Could not get authorization code. Please return to the app and try again.</p>\
    </body>\
    </html>"
}

/// Prepare Kiro OAuth URL (without opening browser)
pub async fn prepare_kiro_oauth_url(app_handle: tauri::AppHandle) -> Result<String, String> {
    use tauri::Emitter;

    // Check if flow is already prepared
    if let Ok(state) = get_kiro_oauth_flow_state().lock() {
        if let Some(s) = state.as_ref() {
            return Ok(s.auth_url.clone());
        }
    }

    // Use fixed port 3128 like Kiro CLI with correct login_option
    let port = 3128u16;
    let redirect_uri = format!("http://localhost:{}/oauth/callback?login_option=google", port);

    // Start local server
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .map_err(|e| format!("Failed to bind to port {}: {}. Make sure port is not in use.", port, e))?;

    crate::modules::logger::log_info(&format!("Kiro OAuth callback server started on port {}", port));

    // Initiate OAuth flow with Kiro
    let (auth_url, code_verifier, state) = oauth_kiro::initiate_login(&redirect_uri).await?;

    // Create channels for code exchange
    let (code_tx, code_rx) = oneshot::channel();
    let code_tx = Arc::new(tokio::sync::Mutex::new(Some(code_tx)));

    // Store flow state
    {
        let mut flow_state = get_kiro_oauth_flow_state().lock().unwrap();
        *flow_state = Some(KiroOAuthFlowState {
            auth_url: auth_url.clone(),
            code_verifier: code_verifier.clone(),
            state: state.clone(),
            redirect_uri: redirect_uri.clone(),
            code_tx: code_tx.clone(),
            code_rx: Some(code_rx),
        });
    }

    // Start callback server task
    let app_handle_clone = app_handle.clone();
    let code_tx_clone = code_tx.clone();
    let redirect_uri_clone = redirect_uri.clone();
    let state_clone = state.clone();
    
    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let mut buffer = [0; 4096];
            if let Ok(n) = stream.read(&mut buffer).await {
                let request = String::from_utf8_lossy(&buffer[..n]);
                
                if let Some(line) = request.lines().next() {
                    if line.starts_with("GET /oauth/callback") {
                        // Parse URL to extract code
                        if let Some(url_part) = line.split_whitespace().nth(1) {
                            let full_url = format!("http://localhost{}", url_part);
                            
                            match url::Url::parse(&full_url) {
                                Ok(url) => {
                                    let mut code_opt = None;
                                    let mut state_opt = None;
                                    let mut error_opt = None;
                                    
                                    for (key, value) in url.query_pairs() {
                                        match key.as_ref() {
                                            "code" => code_opt = Some(value.to_string()),
                                            "state" => state_opt = Some(value.to_string()),
                                            "error" => error_opt = Some(value.to_string()),
                                            _ => {}
                                        }
                                    }
                                    
                                    // Validate state
                                    let state_valid = state_opt.as_ref() == Some(&state_clone);
                                    
                                    if let Some(error) = error_opt {
                                        let _ = stream.write_all(oauth_fail_html().as_bytes()).await;
                                        let _ = stream.flush().await;
                                        
                                        let mut tx_guard = code_tx_clone.lock().await;
                                        if let Some(tx) = tx_guard.take() {
                                            let _ = tx.send(Err(format!("OAuth error: {}", error)));
                                        }
                                    } else if let Some(code) = code_opt {
                                        if state_valid {
                                            let _ = stream.write_all(oauth_success_html().as_bytes()).await;
                                            let _ = stream.flush().await;
                                            
                                            // Send code through channel
                                            let mut tx_guard = code_tx_clone.lock().await;
                                            if let Some(tx) = tx_guard.take() {
                                                let _ = tx.send(Ok(code));
                                            }
                                            
                                            // Emit event for UI
                                            let _ = app_handle_clone.emit("oauth-callback-received", ());
                                            
                                            break; // Exit server loop
                                        } else {
                                            let _ = stream.write_all(oauth_fail_html().as_bytes()).await;
                                            let _ = stream.flush().await;
                                            
                                            let mut tx_guard = code_tx_clone.lock().await;
                                            if let Some(tx) = tx_guard.take() {
                                                let _ = tx.send(Err("Invalid state parameter".to_string()));
                                            }
                                        }
                                    } else {
                                        let _ = stream.write_all(oauth_fail_html().as_bytes()).await;
                                        let _ = stream.flush().await;
                                    }
                                }
                                Err(_) => {
                                    let _ = stream.write_all(oauth_fail_html().as_bytes()).await;
                                    let _ = stream.flush().await;
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    // Emit event with URL
    app_handle.emit("oauth-url-generated", &auth_url)
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    Ok(auth_url)
}

/// Start Kiro OAuth flow (prepare + open browser)
pub async fn start_kiro_oauth_flow(app_handle: tauri::AppHandle) -> Result<oauth_kiro::KiroTokenResponse, String> {
    // Prepare OAuth URL
    let auth_url = prepare_kiro_oauth_url(app_handle.clone()).await?;
    
    // Open browser
    if let Err(e) = open::that(&auth_url) {
        crate::modules::logger::log_warn(&format!("Failed to open browser: {}. Please open manually: {}", e, auth_url));
    }
    
    // Wait for callback
    complete_kiro_oauth_flow(app_handle).await
}

/// Complete Kiro OAuth flow (wait for callback and exchange code)
pub async fn complete_kiro_oauth_flow(_app_handle: tauri::AppHandle) -> Result<oauth_kiro::KiroTokenResponse, String> {
    // Get stored flow state
    let (code_verifier, redirect_uri, code_rx) = {
        let mut flow_state = get_kiro_oauth_flow_state().lock().unwrap();
        if let Some(state) = flow_state.as_mut() {
            let code_verifier = state.code_verifier.clone();
            let redirect_uri = state.redirect_uri.clone();
            let code_rx = state.code_rx.take().ok_or("OAuth flow not prepared")?;
            (code_verifier, redirect_uri, code_rx)
        } else {
            return Err("OAuth flow not prepared. Please call prepare_kiro_oauth_url first.".to_string());
        }
    };
    
    // Wait for authorization code
    let code = code_rx.await
        .map_err(|_| "OAuth callback timeout")?
        .map_err(|e| format!("OAuth callback error: {}", e))?;
    
    crate::modules::logger::log_info("Received authorization code, exchanging for tokens...");
    
    // Exchange code for tokens
    let tokens = oauth_kiro::exchange_code(&code, &code_verifier, &redirect_uri).await?;
    
    // Clear flow state
    {
        let mut flow_state = get_kiro_oauth_flow_state().lock().unwrap();
        *flow_state = None;
    }
    
    Ok(tokens)
}

/// Submit authorization code manually (for manual OAuth flow)
pub async fn submit_kiro_oauth_code(code: String, app_handle: tauri::AppHandle) -> Result<oauth_kiro::KiroTokenResponse, String> {
    // Get stored flow state
    let (code_verifier, redirect_uri) = {
        let flow_state = get_kiro_oauth_flow_state().lock().unwrap();
        if let Some(state) = flow_state.as_ref() {
            let code_verifier = state.code_verifier.clone();
            let redirect_uri = state.redirect_uri.clone();
            (code_verifier, redirect_uri)
        } else {
            return Err("OAuth flow not prepared. Please call prepare_kiro_oauth_url first.".to_string());
        }
    };
    
    crate::modules::logger::log_info("Exchanging manually submitted authorization code for tokens...");
    
    // Exchange code for tokens
    let tokens = oauth_kiro::exchange_code(&code, &code_verifier, &redirect_uri).await?;
    
    // Clear flow state
    {
        let mut flow_state = get_kiro_oauth_flow_state().lock().unwrap();
        *flow_state = None;
    }
    
    // Emit success event
    app_handle.emit("oauth-callback-received", ())
        .map_err(|e| format!("Failed to emit event: {}", e))?;
    
    Ok(tokens)
}

/// Cancel Kiro OAuth flow
pub fn cancel_kiro_oauth_flow() {
    let mut flow_state = get_kiro_oauth_flow_state().lock().unwrap();
    if let Some(state) = flow_state.take() {
        // Send cancellation through channel if still waiting
        tokio::spawn(async move {
            let mut tx_guard = state.code_tx.lock().await;
            if let Some(tx) = tx_guard.take() {
                let _ = tx.send(Err("OAuth flow cancelled by user".to_string()));
            }
        });
    }
}