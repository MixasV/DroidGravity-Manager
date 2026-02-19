#!/usr/bin/env python3
"""
Тестовый скрипт для проверки Kiro OAuth API
Помогает понять правильный формат запросов без пересборки проекта
"""

import requests
import json
import hashlib
import base64
import secrets
import string
from urllib.parse import urlencode, parse_qs, urlparse
import webbrowser
import time
from http.server import HTTPServer, BaseHTTPRequestHandler
import threading

# Kiro API Configuration
KIRO_API_URL = "https://app.kiro.dev"
REDIRECT_URI = "http://localhost:3128"

class OAuthCallbackHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path.startswith('/?'):
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
                    <p>You can close this window and return to the script.</p>
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
    # Generate code_verifier (128 random characters)
    alphabet = string.ascii_letters + string.digits
    verifier = ''.join(secrets.choice(alphabet) for _ in range(128))
    
    # Generate code_challenge = BASE64URL(SHA256(verifier))
    challenge = base64.urlsafe_b64encode(
        hashlib.sha256(verifier.encode()).digest()
    ).decode().rstrip('=')
    
    return verifier, challenge

def test_initiate_login():
    """Test InitiateLogin API with different formats"""
    code_verifier, code_challenge = generate_pkce()
    state = secrets.token_urlsafe(32)
    
    print("=== TESTING INITIATE LOGIN ===")
    print(f"Code Verifier: {code_verifier[:20]}...")
    print(f"Code Challenge: {code_challenge[:20]}...")
    print(f"State: {state[:20]}...")
    
    # Test different request formats
    test_cases = [
        {
            "name": "Format 1: Correct endpoint with operation",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/operation",
            "headers": {
                "Content-Type": "application/json",
                "X-Amz-Target": "KiroWebPortalService.InitiateLogin"
            },
            "data": {
                "idp": "Google",
                "redirectUri": f"{REDIRECT_URI}/oauth/callback?login_option=google",
                "state": state,
                "codeChallenge": code_challenge,
                "codeChallengeMethod": "S256"
            }
        },
        {
            "name": "Format 2: AWS JSON RPC 1.1",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/operation",
            "headers": {
                "Content-Type": "application/x-amz-json-1.1",
                "X-Amz-Target": "KiroWebPortalService.InitiateLogin"
            },
            "data": {
                "idp": "Google",
                "redirectUri": f"{REDIRECT_URI}/oauth/callback?login_option=google",
                "state": state,
                "codeChallenge": code_challenge,
                "codeChallengeMethod": "S256"
            }
        },
        {
            "name": "Format 3: Direct InitiateLogin endpoint",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/InitiateLogin",
            "headers": {"Content-Type": "application/json"},
            "data": {
                "idp": "Google",
                "redirectUri": f"{REDIRECT_URI}/oauth/callback?login_option=google",
                "state": state,
                "codeChallenge": code_challenge,
                "codeChallengeMethod": "S256"
            }
        }
    ]
    
    for test_case in test_cases:
        print(f"\n--- {test_case['name']} ---")
        print(f"URL: {test_case['url']}")
        print(f"Headers: {test_case['headers']}")
        print(f"Data: {json.dumps(test_case['data'], indent=2)}")
        
        try:
            response = requests.post(
                test_case['url'],
                headers=test_case['headers'],
                json=test_case['data'],
                timeout=10
            )
            
            print(f"Status: {response.status_code}")
            print(f"Response Headers: {dict(response.headers)}")
            print(f"Response Body: {response.text}")
            
            if response.status_code == 200:
                try:
                    data = response.json()
                    if 'redirectUrl' in data:
                        print("SUCCESS! Found redirectUrl")
                        return data['redirectUrl'], code_verifier, state
                except:
                    pass
                    
        except Exception as e:
            print(f"ERROR: {e}")
    
    return None, code_verifier, state

def test_simple_signin():
    """Test simple signin URL generation"""
    code_verifier, code_challenge = generate_pkce()
    state = secrets.token_urlsafe(32)
    
    # Generate simple signin URL like in our current code
    auth_url = f"https://app.kiro.dev/signin?state={state}&code_challenge={code_challenge}&code_challenge_method=S256&redirect_uri={urlencode({'': REDIRECT_URI})[1:]}&redirect_from=KiroIDE"
    
    print("=== TESTING SIMPLE SIGNIN URL ===")
    print(f"Generated URL: {auth_url}")
    
    return auth_url, code_verifier, state

def test_get_token(code, code_verifier):
    """Test GetToken API with different formats"""
    print("\n=== TESTING GET TOKEN ===")
    print(f"Code: {code[:20]}...")
    print(f"Code Verifier: {code_verifier[:20]}...")
    
    # Test different request formats
    test_cases = [
        {
            "name": "Format 1: Correct endpoint with snake_case",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/operation",
            "headers": {
                "Content-Type": "application/json",
                "X-Amz-Target": "KiroWebPortalService.GetToken"
            },
            "data": {
                "code": code,
                "code_verifier": code_verifier,
                "redirect_uri": REDIRECT_URI
            }
        },
        {
            "name": "Format 2: AWS JSON RPC 1.1 with snake_case",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/operation",
            "headers": {
                "Content-Type": "application/x-amz-json-1.1",
                "X-Amz-Target": "KiroWebPortalService.GetToken"
            },
            "data": {
                "code": code,
                "code_verifier": code_verifier,
                "redirect_uri": REDIRECT_URI
            }
        },
        {
            "name": "Format 3: Direct GetToken endpoint with snake_case",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/GetToken",
            "headers": {"Content-Type": "application/json"},
            "data": {
                "code": code,
                "code_verifier": code_verifier,
                "redirect_uri": REDIRECT_URI
            }
        },
        {
            "name": "Format 4: camelCase fields",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/GetToken",
            "headers": {"Content-Type": "application/json"},
            "data": {
                "code": code,
                "codeVerifier": code_verifier,
                "redirectUri": REDIRECT_URI
            }
        }
    ]
    
    for test_case in test_cases:
        print(f"\n--- {test_case['name']} ---")
        print(f"URL: {test_case['url']}")
        print(f"Headers: {test_case['headers']}")
        print(f"Data: {json.dumps(test_case['data'], indent=2)}")
        
        try:
            response = requests.post(
                test_case['url'],
                headers=test_case['headers'],
                json=test_case['data'],
                timeout=10
            )
            
            print(f"Status: {response.status_code}")
            print(f"Response Headers: {dict(response.headers)}")
            print(f"Response Body: {response.text}")
            
            if response.status_code == 200:
                try:
                    data = response.json()
                    if 'accessToken' in data or 'access_token' in data:
                        print("SUCCESS! Found access token")
                        return data
                except:
                    pass
                    
        except Exception as e:
            print(f"ERROR: {e}")
    
    return None

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

def main():
    print("KIRO OAUTH API TESTER")
    print("=" * 50)
    
    # Start callback server
    server = start_callback_server()
    time.sleep(1)  # Give server time to start
    
    # Test InitiateLogin API
    redirect_url, code_verifier, state = test_initiate_login()
    
    if not redirect_url:
        print("\nInitiateLogin failed, trying simple signin URL...")
        redirect_url, code_verifier, state = test_simple_signin()
    
    print(f"\nOpening browser with URL: {redirect_url}")
    webbrowser.open(redirect_url)
    
    # Wait for callback
    print("\nWaiting for OAuth callback...")
    print("Please complete authorization in your browser...")
    print("If browser doesn't open automatically, copy this URL:")
    print(redirect_url)
    
    timeout = 300  # 5 minutes timeout
    start_time = time.time()
    
    while server.auth_code is None and (time.time() - start_time) < timeout:
        time.sleep(1)
        # Show progress every 30 seconds
        elapsed = int(time.time() - start_time)
        if elapsed % 30 == 0 and elapsed > 0:
            print(f"Still waiting... ({elapsed}s elapsed)")
    
    if server.auth_code:
        print(f"Received authorization code: {server.auth_code[:20]}...")
        
        # Test GetToken
        tokens = test_get_token(server.auth_code, code_verifier)
        
        if tokens:
            print("\nSUCCESS! OAuth flow completed successfully!")
            print(f"Tokens: {json.dumps(tokens, indent=2)}")
            
            # Test GetUserInfo if we have access token
            if 'accessToken' in tokens or 'access_token' in tokens:
                access_token = tokens.get('accessToken') or tokens.get('access_token')
                print(f"\nTesting GetUserInfo with access token...")
                user_info = test_get_user_info(access_token)
                if user_info:
                    print(f"User Info: {json.dumps(user_info, indent=2)}")
        else:
            print("\nGetToken failed with all formats")
            print("Let's try manual code input...")
            
            # Manual code input fallback
            manual_code = input("\nPaste the authorization code manually (or press Enter to skip): ").strip()
            if manual_code:
                print(f"Testing with manual code: {manual_code[:20]}...")
                tokens = test_get_token(manual_code, code_verifier)
                if tokens:
                    print("\nSUCCESS with manual code!")
                    print(f"Tokens: {json.dumps(tokens, indent=2)}")
    else:
        print("\nTimeout waiting for authorization code")
        print("Let's try manual code input...")
        
        # Manual code input fallback
        manual_code = input("\nPaste the authorization code manually (or press Enter to skip): ").strip()
        if manual_code:
            print(f"Testing with manual code: {manual_code[:20]}...")
            tokens = test_get_token(manual_code, code_verifier)
            if tokens:
                print("\nSUCCESS with manual code!")
                print(f"Tokens: {json.dumps(tokens, indent=2)}")
    
    server.shutdown()
    print("\nTest completed!")

if __name__ == "__main__":
    main()

def test_get_user_info(access_token):
    """Test GetUserInfo API with access token"""
    print("\n=== TESTING GET USER INFO ===")
    print(f"Access Token: {access_token[:20]}...")
    
    # Test different request formats
    test_cases = [
        {
            "name": "Format 1: Direct GetUserInfo endpoint",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/GetUserInfo",
            "headers": {
                "Content-Type": "application/json",
                "Authorization": f"Bearer {access_token}"
            },
            "data": {
                "origin": "KIRO_IDE"
            }
        },
        {
            "name": "Format 2: Operation endpoint with X-Amz-Target",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/operation",
            "headers": {
                "Content-Type": "application/json",
                "X-Amz-Target": "KiroWebPortalService.GetUserInfo",
                "Authorization": f"Bearer {access_token}"
            },
            "data": {
                "origin": "KIRO_IDE"
            }
        },
        {
            "name": "Format 3: AWS JSON RPC 1.1",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/operation",
            "headers": {
                "Content-Type": "application/x-amz-json-1.1",
                "X-Amz-Target": "KiroWebPortalService.GetUserInfo",
                "Authorization": f"Bearer {access_token}"
            },
            "data": {
                "origin": "KIRO_IDE"
            }
        }
    ]
    
    for test_case in test_cases:
        print(f"\n--- {test_case['name']} ---")
        print(f"URL: {test_case['url']}")
        print(f"Headers: {test_case['headers']}")
        print(f"Data: {json.dumps(test_case['data'], indent=2)}")
        
        try:
            response = requests.post(
                test_case['url'],
                headers=test_case['headers'],
                json=test_case['data'],
                timeout=10
            )
            
            print(f"Status: {response.status_code}")
            print(f"Response Headers: {dict(response.headers)}")
            print(f"Response Body: {response.text}")
            
            if response.status_code == 200:
                try:
                    data = response.json()
                    if 'email' in data or 'userId' in data:
                        print("SUCCESS! Found user info")
                        return data
                except:
                    pass
                    
        except Exception as e:
            print(f"ERROR: {e}")
    
    return None