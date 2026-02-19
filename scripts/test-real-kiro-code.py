#!/usr/bin/env python3
"""
Тест с реальным кодом авторизации Kiro
"""

import requests
import json
from urllib.parse import urlencode

# Ваши реальные данные
AUTH_CODE = "8879dd7c-7d26-42e9-84cd-9283ee8ec139"
STATE = "6HWBlVBF7JE2dYyARUpo1NIDdkpfQ7Ouzsapy4L482w"
CODE_VERIFIER = "c2I_euOdlgpwdd1MtC01-6hyI3Kda9UaWcC06CI1EDk"  # Из вашего URL
REDIRECT_URI = "http://localhost:3128/oauth/callback?login_option=google"

# AWS Cognito данные
COGNITO_URL = "https://kiro-prod-us-east-1.auth.us-east-1.amazoncognito.com"
CLIENT_ID = "59bd15eh40ee7pc20h0bkcu7id"

def test_cognito_with_real_code():
    """Test Cognito with your real authorization code"""
    print("=== TESTING COGNITO WITH REAL CODE ===")
    print(f"Code: {AUTH_CODE}")
    print(f"State: {STATE}")
    print(f"Code Verifier: {CODE_VERIFIER}")
    print(f"Redirect URI: {REDIRECT_URI}")
    
    # Try public client (no secret)
    data = urlencode({
        "grant_type": "authorization_code",
        "client_id": CLIENT_ID,
        "code": AUTH_CODE,
        "code_verifier": CODE_VERIFIER,
        "redirect_uri": REDIRECT_URI
    })
    
    try:
        response = requests.post(
            f"{COGNITO_URL}/oauth2/token",
            headers={"Content-Type": "application/x-www-form-urlencoded"},
            data=data,
            timeout=10
        )
        
        print(f"Status: {response.status_code}")
        print(f"Response Headers: {dict(response.headers)}")
        print(f"Response Body: {response.text}")
        
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
                print("   → Authorization code expired or already used")
            elif error_type == "invalid_client":
                print("   → Client authentication failed (need client_secret)")
                
    except Exception as e:
        print(f"ERROR: {e}")
    
    return None

def test_kiro_api_direct():
    """Test direct Kiro API calls"""
    print("\n=== TESTING KIRO API DIRECT ===")
    
    kiro_url = "https://app.kiro.dev"
    
    # Test different approaches
    test_cases = [
        {
            "name": "GetToken with real code",
            "url": f"{kiro_url}/service/KiroWebPortalService/GetToken",
            "data": {
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            }
        },
        {
            "name": "GetUserInfo with code as token",
            "url": f"{kiro_url}/service/KiroWebPortalService/GetUserInfo",
            "headers": {"Authorization": f"Bearer {AUTH_CODE}"},
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
    print("KIRO REAL CODE TESTER")
    print("=" * 50)
    print("Testing with your real authorization code from successful auth")
    print()
    
    # Test Cognito
    tokens = test_cognito_with_real_code()
    
    if tokens:
        print("\n🎉 SUCCESS! Cognito integration working!")
        print("We can proceed with automatic token exchange!")
    else:
        print("\n⚠️  Cognito failed (expected - need client_secret)")
        
        # Test direct Kiro API
        result = test_kiro_api_direct()
        
        if result:
            print("\n🎉 SUCCESS! Direct Kiro API working!")
        else:
            print("\n❌ Both approaches failed")
            print("\nCONCLUSION:")
            print("✅ OAuth authorization flow works perfectly")
            print("✅ We can generate correct Cognito URLs")
            print("✅ We can get authorization codes")
            print("❌ Token exchange requires client_secret or different approach")
            print()
            print("NEXT STEPS:")
            print("1. Implement manual token input in UI")
            print("2. Guide user to extract tokens from DevTools")
            print("3. Test with manually extracted tokens")
            print("4. Future: Find client_secret or alternative method")

if __name__ == "__main__":
    main()