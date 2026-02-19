#!/usr/bin/env python3
"""
Тест альтернативных endpoints для получения токенов
"""

import requests
import json
from urllib.parse import urlencode

# Данные из вашей авторизации
KIRO_API_URL = "https://app.kiro.dev"
AUTH_CODE = "77227eb9-4cae-4f2a-b8bc-3882e6141bef"
CODE_VERIFIER = "PpAJjiWjKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKv"
REDIRECT_URI = "http://localhost:3128"

def test_alternative_endpoints():
    """Test alternative token endpoints"""
    print("=== TESTING ALTERNATIVE TOKEN ENDPOINTS ===")
    
    # Test different endpoints that might exist
    test_cases = [
        # Standard OAuth2 endpoints
        {
            "name": "OAuth2 Token Endpoint",
            "url": f"{KIRO_API_URL}/oauth2/token",
            "method": "POST",
            "headers": {"Content-Type": "application/x-www-form-urlencoded"},
            "data": urlencode({
                "grant_type": "authorization_code",
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            })
        },
        {
            "name": "Auth Token Endpoint",
            "url": f"{KIRO_API_URL}/auth/token",
            "method": "POST",
            "headers": {"Content-Type": "application/json"},
            "data": {
                "grant_type": "authorization_code",
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            }
        },
        {
            "name": "API Token Endpoint",
            "url": f"{KIRO_API_URL}/api/token",
            "method": "POST",
            "headers": {"Content-Type": "application/json"},
            "data": {
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            }
        },
        {
            "name": "API Auth Token",
            "url": f"{KIRO_API_URL}/api/auth/token",
            "method": "POST",
            "headers": {"Content-Type": "application/json"},
            "data": {
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            }
        },
        # AWS Cognito direct endpoints (if Kiro uses Cognito)
        {
            "name": "Cognito Token Exchange",
            "url": "https://kiro-prod-us-east-1.auth.us-east-1.amazoncognito.com/oauth2/token",
            "method": "POST",
            "headers": {"Content-Type": "application/x-www-form-urlencoded"},
            "data": urlencode({
                "grant_type": "authorization_code",
                "client_id": "59bd15eh40ee7pc20h0bkcu7id",  # From documentation
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            })
        },
        # Try different service names
        {
            "name": "KiroAuthService GetToken",
            "url": f"{KIRO_API_URL}/service/KiroAuthService/GetToken",
            "method": "POST",
            "headers": {"Content-Type": "application/json"},
            "data": {
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            }
        },
        {
            "name": "KiroOAuthService GetToken",
            "url": f"{KIRO_API_URL}/service/KiroOAuthService/GetToken",
            "method": "POST",
            "headers": {"Content-Type": "application/json"},
            "data": {
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            }
        }
    ]
    
    for i, test_case in enumerate(test_cases, 1):
        print(f"\n--- {test_case['name']} ---")
        print(f"URL: {test_case['url']}")
        print(f"Headers: {test_case['headers']}")
        
        if isinstance(test_case['data'], dict):
            print(f"Data: {json.dumps(test_case['data'], indent=2)}")
        else:
            print(f"Data: {test_case['data']}")
        
        try:
            if isinstance(test_case['data'], dict):
                response = requests.post(
                    test_case['url'],
                    headers=test_case['headers'],
                    json=test_case['data'],
                    timeout=10
                )
            else:
                response = requests.post(
                    test_case['url'],
                    headers=test_case['headers'],
                    data=test_case['data'],
                    timeout=10
                )
            
            print(f"Status: {response.status_code}")
            print(f"Response Headers: {dict(response.headers)}")
            print(f"Response Body: {response.text}")
            
            # Check for success indicators
            if response.status_code == 200:
                try:
                    data = response.json()
                    if any(key in data for key in ['access_token', 'accessToken', 'token', 'id_token']):
                        print("🎉 SUCCESS! Found token in response!")
                        print(f"Full response: {json.dumps(data, indent=2)}")
                        return data
                    elif not response.text.strip().startswith('<') and 'Exception' not in response.text:
                        print("✅ SUCCESS! Valid JSON response (no exception)")
                        print(f"Full response: {json.dumps(data, indent=2)}")
                        return data
                except:
                    if not response.text.strip().startswith('<') and 'Exception' not in response.text:
                        print("✅ SUCCESS! Valid non-JSON response")
                        return {"raw_response": response.text}
            elif response.status_code in [400, 401]:
                print("⚠️  Client error - endpoint exists but request invalid")
                if "invalid" in response.text.lower() or "error" in response.text.lower():
                    print("   This might be the correct endpoint with wrong parameters")
            elif response.status_code == 404:
                print("❌ Not found - endpoint doesn't exist")
            elif "UnknownOperationException" in response.text:
                print("❌ Unknown operation")
            else:
                print(f"⚠️  Unexpected status: {response.status_code}")
                    
        except Exception as e:
            print(f"❌ ERROR: {e}")
    
    print("\n❌ All alternative endpoints failed")
    return None

def main():
    print("KIRO ALTERNATIVE ENDPOINTS TESTER")
    print("=" * 50)
    print("Testing various possible token endpoints...")
    print()
    
    result = test_alternative_endpoints()
    
    if result:
        print("\n🎉 SUCCESS! Found working endpoint!")
    else:
        print("\n❌ No working endpoints found")
        print("\nConclusions:")
        print("1. Kiro API has significantly changed")
        print("2. May require different authentication flow")
        print("3. Might need to reverse engineer current Kiro client")
        print("4. Could be using different service architecture")

if __name__ == "__main__":
    main()