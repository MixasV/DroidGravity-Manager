#!/usr/bin/env python3
"""
Ручное тестирование Kiro авторизации
Пользователь авторизуется в браузере и копирует токены
"""

import requests
import json
import hashlib
import base64
import secrets
import string
import webbrowser
from urllib.parse import urlencode

# Kiro API Configuration
KIRO_API_URL = "https://app.kiro.dev"

def generate_pkce():
    """Generate PKCE code_verifier and code_challenge"""
    alphabet = string.ascii_letters + string.digits
    verifier = ''.join(secrets.choice(alphabet) for _ in range(128))
    
    challenge = base64.urlsafe_b64encode(
        hashlib.sha256(verifier.encode()).digest()
    ).decode().rstrip('=')
    
    return verifier, challenge

def generate_signin_url():
    """Generate Kiro signin URL"""
    code_verifier, code_challenge = generate_pkce()
    state = secrets.token_urlsafe(32)
    
    # Build Kiro signin URL
    auth_url = f"https://app.kiro.dev/signin?state={state}&code_challenge={code_challenge}&code_challenge_method=S256&redirect_uri={urlencode({'': 'http://localhost:3128'})[1:]}&redirect_from=KiroIDE"
    
    return auth_url, code_verifier, state

def test_kiro_api_with_token(access_token):
    """Test Kiro API endpoints with access token"""
    print(f"\n=== TESTING KIRO API WITH TOKEN ===")
    print(f"Access Token: {access_token[:50]}...")
    
    # Test different Kiro API endpoints
    test_cases = [
        {
            "name": "GetUserInfo",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/GetUserInfo",
            "method": "POST",
            "headers": {
                "Content-Type": "application/json",
                "Authorization": f"Bearer {access_token}"
            },
            "data": {"origin": "KIRO_IDE"}
        },
        {
            "name": "Chat Completions (Claude format)",
            "url": f"{KIRO_API_URL}/v1/messages",
            "method": "POST",
            "headers": {
                "Content-Type": "application/json",
                "Authorization": f"Bearer {access_token}",
                "anthropic-version": "2023-06-01"
            },
            "data": {
                "model": "claude-3-haiku-20240307",
                "max_tokens": 100,
                "messages": [
                    {"role": "user", "content": "Hello, test message"}
                ]
            }
        },
        {
            "name": "Models List",
            "url": f"{KIRO_API_URL}/v1/models",
            "method": "GET",
            "headers": {
                "Authorization": f"Bearer {access_token}"
            }
        }
    ]
    
    results = []
    
    for test_case in test_cases:
        print(f"\n--- Testing {test_case['name']} ---")
        print(f"URL: {test_case['url']}")
        
        try:
            if test_case['method'] == 'GET':
                response = requests.get(
                    test_case['url'],
                    headers=test_case['headers'],
                    timeout=30
                )
            else:
                response = requests.post(
                    test_case['url'],
                    headers=test_case['headers'],
                    json=test_case.get('data'),
                    timeout=30
                )
            
            print(f"Status: {response.status_code}")
            print(f"Response Headers: {dict(response.headers)}")
            
            # Truncate long responses
            response_text = response.text
            if len(response_text) > 500:
                response_text = response_text[:500] + "... [truncated]"
            print(f"Response Body: {response_text}")
            
            result = {
                "endpoint": test_case['name'],
                "status": response.status_code,
                "success": response.status_code == 200,
                "response_length": len(response.text)
            }
            
            if response.status_code == 200:
                print("✅ SUCCESS!")
                try:
                    data = response.json()
                    if test_case['name'] == "GetUserInfo" and 'email' in data:
                        print(f"   User Email: {data.get('email')}")
                        print(f"   User ID: {data.get('userId', 'N/A')}")
                        print(f"   Status: {data.get('status', 'N/A')}")
                    elif test_case['name'] == "Models List" and isinstance(data, dict) and 'data' in data:
                        models = data.get('data', [])
                        print(f"   Found {len(models)} models")
                        for model in models[:5]:  # Show first 5
                            print(f"     - {model.get('id', 'unknown')}")
                    elif test_case['name'] == "Chat Completions":
                        print("   Chat API working!")
                except:
                    print("   Valid response but not JSON")
            elif response.status_code == 401:
                print("❌ UNAUTHORIZED - Token invalid or expired")
            elif response.status_code == 403:
                print("❌ FORBIDDEN - Token valid but no permission")
            elif response.status_code == 404:
                print("❌ NOT FOUND - Endpoint doesn't exist")
            else:
                print(f"⚠️  Status {response.status_code}")
            
            results.append(result)
                    
        except Exception as e:
            print(f"❌ ERROR: {e}")
            results.append({
                "endpoint": test_case['name'],
                "status": "error",
                "success": False,
                "error": str(e)
            })
    
    return results

def main():
    print("KIRO MANUAL AUTHENTICATION TESTER")
    print("=" * 50)
    print("This script helps you test Kiro integration manually")
    print()
    
    # Step 1: Generate signin URL
    print("STEP 1: Generate Kiro signin URL")
    auth_url, code_verifier, state = generate_signin_url()
    
    print(f"Generated signin URL:")
    print(auth_url)
    print()
    print("Opening browser...")
    webbrowser.open(auth_url)
    
    print("\n" + "=" * 50)
    print("STEP 2: Manual Authorization")
    print("=" * 50)
    print("1. Complete authorization in the browser")
    print("2. After successful auth, open browser DevTools (F12)")
    print("3. Go to Application/Storage tab")
    print("4. Look for cookies or localStorage with 'token' or 'access'")
    print("5. Or check Network tab for API calls with Authorization headers")
    print()
    
    # Step 2: Get token from user
    print("Please find the access token and paste it here:")
    access_token = input("Access Token: ").strip()
    
    if not access_token:
        print("❌ No token provided. Exiting.")
        return
    
    if len(access_token) < 20:
        print("⚠️  Token seems too short. Are you sure it's correct?")
        confirm = input("Continue anyway? (y/n): ").strip().lower()
        if confirm != 'y':
            return
    
    # Step 3: Test API with token
    print("\n" + "=" * 50)
    print("STEP 3: Testing Kiro API")
    print("=" * 50)
    
    results = test_kiro_api_with_token(access_token)
    
    # Summary
    print("\n" + "=" * 50)
    print("SUMMARY")
    print("=" * 50)
    
    successful = [r for r in results if r['success']]
    failed = [r for r in results if not r['success']]
    
    print(f"✅ Successful endpoints: {len(successful)}")
    for result in successful:
        print(f"   - {result['endpoint']}")
    
    print(f"❌ Failed endpoints: {len(failed)}")
    for result in failed:
        print(f"   - {result['endpoint']} (Status: {result['status']})")
    
    if successful:
        print("\n🎉 SUCCESS! Kiro API integration is possible!")
        print("\nNext steps:")
        print("1. Implement token extraction from browser")
        print("2. Add token refresh mechanism")
        print("3. Integrate working endpoints into DroidGravity Manager")
        
        # Save working token for further testing
        with open("kiro_working_token.txt", "w") as f:
            f.write(access_token)
        print(f"\n💾 Saved working token to 'kiro_working_token.txt'")
    else:
        print("\n❌ No endpoints worked. Possible issues:")
        print("1. Token is invalid or expired")
        print("2. Token format is incorrect")
        print("3. Kiro API has changed")
        print("4. Additional authentication required")

if __name__ == "__main__":
    main()