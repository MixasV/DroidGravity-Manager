#!/usr/bin/env python3
"""
Тест GetToken API с реальным кодом авторизации
"""

import requests
import json

# Данные из вашей авторизации
KIRO_API_URL = "https://app.kiro.dev"
AUTH_CODE = "77227eb9-4cae-4f2a-b8bc-3882e6141bef"
STATE = "QzhhtM0CEt2lqVMQHXkjDVIGxmEUreSXH-5NVwcvH8I"
CODE_VERIFIER = "PpAJjiWjKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKvKXfvcbp0xaKv"  # Из скрипта
REDIRECT_URI = "http://localhost:3128"

def test_get_token_with_real_code():
    """Test GetToken API with real authorization code"""
    print("=== TESTING GETTOKEN WITH REAL CODE ===")
    print(f"Code: {AUTH_CODE}")
    print(f"State: {STATE}")
    print(f"Code Verifier: {CODE_VERIFIER[:20]}...")
    print(f"Redirect URI: {REDIRECT_URI}")
    
    # Test different request formats
    test_cases = [
        {
            "name": "Format 1: snake_case с X-Amz-Target",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/operation",
            "headers": {
                "Content-Type": "application/json",
                "X-Amz-Target": "KiroWebPortalService.GetToken"
            },
            "data": {
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            }
        },
        {
            "name": "Format 2: AWS JSON RPC 1.1 с snake_case",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/operation",
            "headers": {
                "Content-Type": "application/x-amz-json-1.1",
                "X-Amz-Target": "KiroWebPortalService.GetToken"
            },
            "data": {
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            }
        },
        {
            "name": "Format 3: Direct GetToken endpoint с snake_case",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/GetToken",
            "headers": {"Content-Type": "application/json"},
            "data": {
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            }
        },
        {
            "name": "Format 4: camelCase fields",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/GetToken",
            "headers": {"Content-Type": "application/json"},
            "data": {
                "code": AUTH_CODE,
                "codeVerifier": CODE_VERIFIER,
                "redirectUri": REDIRECT_URI
            }
        },
        {
            "name": "Format 5: Callback redirect_uri",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/GetToken",
            "headers": {"Content-Type": "application/json"},
            "data": {
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": f"{REDIRECT_URI}/oauth/callback?login_option=google"
            }
        }
    ]
    
    for i, test_case in enumerate(test_cases, 1):
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
                        print("🎉 SUCCESS! Found access token")
                        print(f"Full response: {json.dumps(data, indent=2)}")
                        return data
                    elif not data.get('Output', {}).get('__type', '').endswith('Exception'):
                        print("✅ SUCCESS! Valid response (no exception)")
                        print(f"Full response: {json.dumps(data, indent=2)}")
                        return data
                except Exception as e:
                    print(f"JSON parse error: {e}")
            elif response.status_code in [400, 401]:
                print("⚠️  Client error - endpoint exists but request invalid")
            elif response.status_code == 404:
                print("❌ Not found - endpoint doesn't exist")
            elif "UnknownOperationException" in response.text:
                print("❌ Unknown operation - API changed or wrong format")
                    
        except Exception as e:
            print(f"❌ ERROR: {e}")
    
    print("\n❌ All formats failed")
    return None

def main():
    print("KIRO GETTOKEN REAL CODE TESTER")
    print("=" * 50)
    
    result = test_get_token_with_real_code()
    
    if result:
        print("\n🎉 SUCCESS! OAuth flow working!")
        
        # Test GetUserInfo if we have access token
        access_token = result.get('accessToken') or result.get('access_token')
        if access_token:
            print(f"\nTesting GetUserInfo with access token...")
            # Add GetUserInfo test here if needed
    else:
        print("\n❌ GetToken failed - need to investigate API changes")
        print("\nPossible issues:")
        print("1. API endpoints have changed")
        print("2. Different authentication required")
        print("3. Code verifier mismatch")
        print("4. Redirect URI mismatch")

if __name__ == "__main__":
    main()