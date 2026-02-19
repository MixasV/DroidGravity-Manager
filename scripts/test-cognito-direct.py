#!/usr/bin/env python3
"""
Тест прямого обращения к AWS Cognito для получения токенов
"""

import requests
import json
from urllib.parse import urlencode

# Данные из вашей авторизации
AUTH_CODE = "c8fa952a-a0e1-417d-b838-de4ab4222db7"
CODE_VERIFIER = "PpAJjiWjKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKv"
REDIRECT_URI = "http://localhost:3128"

# AWS Cognito данные из документации
COGNITO_URL = "https://kiro-prod-us-east-1.auth.us-east-1.amazoncognito.com"
CLIENT_ID = "59bd15eh40ee7pc20h0bkcu7id"

def test_cognito_token_exchange():
    """Test AWS Cognito token exchange with different approaches"""
    print("=== TESTING AWS COGNITO TOKEN EXCHANGE ===")
    
    test_cases = [
        {
            "name": "Public Client (no client_secret)",
            "data": urlencode({
                "grant_type": "authorization_code",
                "client_id": CLIENT_ID,
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            })
        },
        {
            "name": "With empty client_secret",
            "data": urlencode({
                "grant_type": "authorization_code",
                "client_id": CLIENT_ID,
                "client_secret": "",
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            })
        },
        {
            "name": "PKCE only (no client_secret)",
            "data": urlencode({
                "grant_type": "authorization_code",
                "client_id": CLIENT_ID,
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            })
        },
        {
            "name": "Different redirect_uri format",
            "data": urlencode({
                "grant_type": "authorization_code",
                "client_id": CLIENT_ID,
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": f"{REDIRECT_URI}/oauth/callback?login_option=google"
            })
        },
        {
            "name": "With scope parameter",
            "data": urlencode({
                "grant_type": "authorization_code",
                "client_id": CLIENT_ID,
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI,
                "scope": "email openid"
            })
        }
    ]
    
    for i, test_case in enumerate(test_cases, 1):
        print(f"\n--- {test_case['name']} ---")
        print(f"URL: {COGNITO_URL}/oauth2/token")
        print(f"Data: {test_case['data']}")
        
        try:
            response = requests.post(
                f"{COGNITO_URL}/oauth2/token",
                headers={"Content-Type": "application/x-www-form-urlencoded"},
                data=test_case['data'],
                timeout=10
            )
            
            print(f"Status: {response.status_code}")
            print(f"Response Headers: {dict(response.headers)}")
            print(f"Response Body: {response.text}")
            
            if response.status_code == 200:
                try:
                    data = response.json()
                    if 'access_token' in data:
                        print("🎉 SUCCESS! Got access token from Cognito!")
                        print(f"Access Token: {data['access_token'][:50]}...")
                        if 'id_token' in data:
                            print(f"ID Token: {data['id_token'][:50]}...")
                        if 'refresh_token' in data:
                            print(f"Refresh Token: {data['refresh_token'][:50]}...")
                        return data
                except Exception as e:
                    print(f"JSON parse error: {e}")
            elif response.status_code == 400:
                try:
                    error_data = response.json()
                    error_type = error_data.get('error', 'unknown')
                    error_desc = error_data.get('error_description', 'no description')
                    print(f"⚠️  Error: {error_type} - {error_desc}")
                    
                    if error_type == "invalid_client":
                        print("   → Client configuration issue")
                    elif error_type == "invalid_grant":
                        print("   → Authorization code expired or invalid")
                    elif error_type == "invalid_request":
                        print("   → Missing or invalid parameters")
                    elif error_type == "unsupported_grant_type":
                        print("   → Grant type not supported")
                except:
                    print(f"⚠️  Bad request: {response.text}")
            else:
                print(f"⚠️  Unexpected status: {response.status_code}")
                    
        except Exception as e:
            print(f"❌ ERROR: {e}")
    
    print("\n❌ All Cognito attempts failed")
    return None

def test_user_info_endpoint():
    """Test if we can get user info directly from Cognito"""
    print("\n=== TESTING COGNITO USER INFO ===")
    
    # Try to get user info without token (should fail but tell us about endpoint)
    try:
        response = requests.get(
            f"{COGNITO_URL}/oauth2/userInfo",
            headers={"Authorization": "Bearer fake-token"},
            timeout=10
        )
        
        print(f"UserInfo endpoint status: {response.status_code}")
        print(f"UserInfo response: {response.text}")
        
        if response.status_code == 401:
            print("✅ UserInfo endpoint exists (returned 401 as expected)")
        elif response.status_code == 400:
            print("✅ UserInfo endpoint exists (returned 400)")
        else:
            print(f"⚠️  Unexpected status: {response.status_code}")
            
    except Exception as e:
        print(f"❌ UserInfo endpoint error: {e}")

def main():
    print("KIRO AWS COGNITO DIRECT TESTER")
    print("=" * 50)
    print("Testing direct AWS Cognito token exchange...")
    print(f"Client ID: {CLIENT_ID}")
    print(f"Auth Code: {AUTH_CODE}")
    print()
    
    # Test token exchange
    tokens = test_cognito_token_exchange()
    
    # Test user info endpoint
    test_user_info_endpoint()
    
    if tokens:
        print("\n🎉 SUCCESS! Cognito integration working!")
        print("\nNext steps:")
        print("1. Update Rust code to use Cognito directly")
        print("2. Skip Kiro's GetToken API entirely")
        print("3. Use Cognito tokens for Kiro API calls")
    else:
        print("\n❌ Cognito token exchange failed")
        print("\nPossible issues:")
        print("1. Authorization code expired (codes usually expire in 10 minutes)")
        print("2. Client requires secret (private client)")
        print("3. Redirect URI mismatch")
        print("4. PKCE code_verifier mismatch")
        print("\nRecommendations:")
        print("1. Try with fresh authorization code")
        print("2. Check if Kiro client is public or private")
        print("3. Investigate if client_secret can be extracted")

if __name__ == "__main__":
    main()