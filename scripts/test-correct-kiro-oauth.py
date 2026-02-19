#!/usr/bin/env python3
"""
Правильный тест Kiro OAuth с исправленными параметрами
"""

import requests
import json
import hashlib
import base64
import secrets
import string
import webbrowser
import time
from urllib.parse import urlencode, parse_qs, urlparse
from http.server import HTTPServer, BaseHTTPRequestHandler
import threading

# Kiro API Configuration
KIRO_API_URL = "https://app.kiro.dev"
COGNITO_URL = "https://kiro-prod-us-east-1.auth.us-east-1.amazoncognito.com"
CLIENT_ID = "59bd15eh40ee7pc20h0bkcu7id"
REDIRECT_URI = "http://localhost:3128/oauth/callback?login_option=google"

class OAuthCallbackHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        # Parse callback parameters
        parsed = urlparse(self.path)
        params = parse_qs(parsed.query)
        
        if 'code' in params:
            self.server.auth_code = params['code'][0]
            self.server.state = params.get('state', [None])[0]
            
            # Send success response
            self.send_response(200)
            self.send_header('Content-type', 'text/html')
            self.end_headers()
            self.wfile.write(b'''
            <html><body style="font-family: sans-serif; text-align: center; padding: 50px;">
                <h1 style="color: green;">Authorization Successful!</h1>
                <p>Authorization code received. You can close this window.</p>
                <script>setTimeout(function() { window.close(); }, 2000);</script>
            </body></html>
            ''')
            return
        
        # Default response
        self.send_response(200)
        self.send_header('Content-type', 'text/plain')
        self.end_headers()
        self.wfile.write(b'OAuth callback server running...')
    
    def log_message(self, format, *args):
        pass  # Suppress default logging

def generate_pkce():
    """Generate PKCE code_verifier and code_challenge"""
    alphabet = string.ascii_letters + string.digits
    verifier = ''.join(secrets.choice(alphabet) for _ in range(128))
    
    challenge = base64.urlsafe_b64encode(
        hashlib.sha256(verifier.encode()).digest()
    ).decode().rstrip('=')
    
    return verifier, challenge

def start_callback_server():
    """Start local callback server"""
    server = HTTPServer(('localhost', 3128), OAuthCallbackHandler)
    server.auth_code = None
    server.state = None
    
    def run_server():
        print("Starting callback server on http://localhost:3128")
        server.serve_forever()
    
    thread = threading.Thread(target=run_server, daemon=True)
    thread.start()
    return server

def generate_correct_cognito_url():
    """Generate correct Cognito authorization URL"""
    code_verifier, code_challenge = generate_pkce()
    state = secrets.token_urlsafe(32)
    
    # Build correct Cognito URL with all required parameters
    params = {
        'client_id': CLIENT_ID,
        'response_type': 'code',
        'scope': 'email openid',
        'redirect_uri': REDIRECT_URI,
        'state': state,
        'code_challenge': code_challenge,
        'code_challenge_method': 'S256',
        'identity_provider': 'Google'  # This was missing!
    }
    
    auth_url = f"{COGNITO_URL}/oauth2/authorize?" + urlencode(params)
    
    return auth_url, code_verifier, state

def test_cognito_token_exchange(auth_code, code_verifier):
    """Test token exchange with Cognito using different client_secret approaches"""
    print(f"\n=== TESTING COGNITO TOKEN EXCHANGE ===")
    print(f"Auth Code: {auth_code}")
    print(f"Code Verifier: {code_verifier[:20]}...")
    
    # Try different approaches for client_secret
    test_cases = [
        {
            "name": "No client_secret (public client)",
            "data": {
                "grant_type": "authorization_code",
                "client_id": CLIENT_ID,
                "code": auth_code,
                "code_verifier": code_verifier,
                "redirect_uri": REDIRECT_URI
            }
        },
        {
            "name": "Empty client_secret",
            "data": {
                "grant_type": "authorization_code",
                "client_id": CLIENT_ID,
                "client_secret": "",
                "code": auth_code,
                "code_verifier": code_verifier,
                "redirect_uri": REDIRECT_URI
            }
        },
        {
            "name": "Client_secret = client_id",
            "data": {
                "grant_type": "authorization_code",
                "client_id": CLIENT_ID,
                "client_secret": CLIENT_ID,
                "code": auth_code,
                "code_verifier": code_verifier,
                "redirect_uri": REDIRECT_URI
            }
        }
    ]
    
    for test_case in test_cases:
        print(f"\n--- {test_case['name']} ---")
        
        try:
            response = requests.post(
                f"{COGNITO_URL}/oauth2/token",
                headers={"Content-Type": "application/x-www-form-urlencoded"},
                data=urlencode(test_case['data']),
                timeout=10
            )
            
            print(f"Status: {response.status_code}")
            print(f"Response: {response.text}")
            
            if response.status_code == 200:
                try:
                    tokens = response.json()
                    if 'access_token' in tokens:
                        print("🎉 SUCCESS! Got tokens from Cognito!")
                        return tokens
                except:
                    pass
            elif response.status_code == 400:
                error_data = response.json()
                error_type = error_data.get('error', 'unknown')
                print(f"Error: {error_type}")
                if error_type == "invalid_grant":
                    print("   → Authorization code expired or invalid")
                elif error_type == "invalid_client":
                    print("   → Client authentication failed")
                    
        except Exception as e:
            print(f"ERROR: {e}")
    
    return None

def test_kiro_api_direct(auth_code, code_verifier):
    """Test if we can use auth code directly with Kiro API"""
    print(f"\n=== TESTING KIRO API DIRECT ===")
    
    # Try to use auth code directly with Kiro
    test_cases = [
        {
            "name": "Direct code exchange",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/GetToken",
            "data": {
                "code": auth_code,
                "code_verifier": code_verifier,
                "redirect_uri": REDIRECT_URI
            }
        },
        {
            "name": "Auth code as bearer token",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/GetUserInfo",
            "headers": {"Authorization": f"Bearer {auth_code}"},
            "data": {"origin": "KIRO_IDE"}
        }
    ]
    
    for test_case in test_cases:
        print(f"\n--- {test_case['name']} ---")
        
        try:
            headers = test_case.get('headers', {})
            headers["Content-Type"] = "application/json"
            
            response = requests.post(
                test_case['url'],
                headers=headers,
                json=test_case['data'],
                timeout=10
            )
            
            print(f"Status: {response.status_code}")
            print(f"Response: {response.text}")
            
            if response.status_code == 200 and "UnknownOperationException" not in response.text:
                print("✅ SUCCESS! Kiro API responded positively")
                return response.json()
                    
        except Exception as e:
            print(f"ERROR: {e}")
    
    return None

def main():
    print("CORRECT KIRO OAUTH TESTER")
    print("=" * 50)
    print("Testing with correct Cognito parameters including identity_provider=Google")
    print()
    
    # Start callback server
    server = start_callback_server()
    time.sleep(1)
    
    # Generate correct Cognito URL
    auth_url, code_verifier, state = generate_correct_cognito_url()
    
    print("Generated correct Cognito authorization URL:")
    print(auth_url)
    print()
    print("Opening browser...")
    webbrowser.open(auth_url)
    
    # Wait for callback
    print("\nWaiting for OAuth callback...")
    print("Please complete authorization in your browser...")
    
    timeout = 300  # 5 minutes
    start_time = time.time()
    
    while server.auth_code is None and (time.time() - start_time) < timeout:
        time.sleep(1)
        elapsed = int(time.time() - start_time)
        if elapsed % 30 == 0 and elapsed > 0:
            print(f"Still waiting... ({elapsed}s elapsed)")
    
    if server.auth_code:
        print(f"✅ Received authorization code: {server.auth_code[:20]}...")
        
        # Test Cognito token exchange
        tokens = test_cognito_token_exchange(server.auth_code, code_verifier)
        
        if tokens:
            print("\n🎉 SUCCESS! Cognito integration working!")
            print("Access token obtained from Cognito")
            
            # Test Kiro API with Cognito token
            access_token = tokens['access_token']
            print(f"\nTesting Kiro API with Cognito token...")
            # Add Kiro API tests here
            
        else:
            print("\n⚠️  Cognito token exchange failed, trying direct Kiro API...")
            # Test direct Kiro API
            result = test_kiro_api_direct(server.auth_code, code_verifier)
            
            if result:
                print("\n🎉 SUCCESS! Direct Kiro API working!")
            else:
                print("\n❌ Both Cognito and direct Kiro API failed")
    else:
        print("\n⏰ Timeout waiting for authorization code")
        print("Manual testing:")
        print("1. Copy the authorization URL above")
        print("2. Open it in browser and complete auth")
        print("3. Copy the code from callback URL")
        
        manual_code = input("\nPaste authorization code manually (or press Enter to skip): ").strip()
        if manual_code:
            tokens = test_cognito_token_exchange(manual_code, code_verifier)
            if tokens:
                print("🎉 SUCCESS with manual code!")
    
    server.shutdown()
    print("\nTest completed!")

if __name__ == "__main__":
    main()