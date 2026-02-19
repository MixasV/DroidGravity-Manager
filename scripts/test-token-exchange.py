#!/usr/bin/env python3
"""
Тест обмена authorization code на токены
"""

import requests
import json
import urllib.parse

def test_cognito_token_exchange(code, code_verifier):
    """Тест обмена через Cognito"""
    print("=== TESTING COGNITO TOKEN EXCHANGE ===")
    
    cognito_url = "https://kiro-prod-us-east-1.auth.us-east-1.amazoncognito.com/oauth2/token"
    client_id = "59bd15eh40ee7pc20h0bkcu7id"
    redirect_uri = "http://localhost:3128/oauth/callback?login_option=google"
    
    data = {
        "grant_type": "authorization_code",
        "client_id": client_id,
        "code": code,
        "code_verifier": code_verifier,
        "redirect_uri": redirect_uri
    }
    
    headers = {
        "Content-Type": "application/x-www-form-urlencoded",
        "User-Agent": "DroidGravity-Manager/2.0.0"
    }
    
    print(f"📋 Request:")
    print(f"URL: {cognito_url}")
    print(f"Data: {data}")
    print()
    
    try:
        response = requests.post(cognito_url, data=urllib.parse.urlencode(data), headers=headers, timeout=10)
        print(f"Status: {response.status_code}")
        print(f"Headers: {dict(response.headers)}")
        print(f"Response: {response.text}")
        
        if response.status_code == 200:
            try:
                tokens = response.json()
                print(f"\n🎉 SUCCESS! Got tokens:")
                print(json.dumps(tokens, indent=2))
                return tokens
            except json.JSONDecodeError:
                print("Response is not JSON")
        
    except Exception as e:
        print(f"Error: {e}")
    
    return None

def test_kiro_api_token_exchange(code, code_verifier):
    """Тест обмена через Kiro API"""
    print("\n=== TESTING KIRO API TOKEN EXCHANGE ===")
    
    kiro_api_url = "https://app.kiro.dev/api/v1/GetToken"
    redirect_uri = "http://localhost:3128/oauth/callback?login_option=google"
    
    data = {
        "code": code,
        "code_verifier": code_verifier,
        "redirect_uri": redirect_uri
    }
    
    headers = {
        "Content-Type": "application/json",
        "User-Agent": "DroidGravity-Manager/2.0.0"
    }
    
    print(f"📋 Request:")
    print(f"URL: {kiro_api_url}")
    print(f"Data: {json.dumps(data, indent=2)}")
    print()
    
    try:
        response = requests.post(kiro_api_url, json=data, headers=headers, timeout=10)
        print(f"Status: {response.status_code}")
        print(f"Headers: {dict(response.headers)}")
        print(f"Response: {response.text}")
        
        if response.status_code == 200:
            try:
                tokens = response.json()
                print(f"\n🎉 SUCCESS! Got tokens:")
                print(json.dumps(tokens, indent=2))
                return tokens
            except json.JSONDecodeError:
                print("Response is not JSON")
        
    except Exception as e:
        print(f"Error: {e}")
    
    return None

def test_alternative_endpoints(code, code_verifier):
    """Тест альтернативных endpoints"""
    print("\n=== TESTING ALTERNATIVE ENDPOINTS ===")
    
    endpoints = [
        "https://app.kiro.dev/service/KiroWebPortalService/operation/GetToken",
        "https://app.kiro.dev/GetToken",
        "https://app.kiro.dev/token",
        "https://api.kiro.dev/token"
    ]
    
    redirect_uri = "http://localhost:3128/oauth/callback?login_option=google"
    
    data = {
        "code": code,
        "code_verifier": code_verifier,
        "redirect_uri": redirect_uri
    }
    
    for endpoint in endpoints:
        print(f"\n--- Testing: {endpoint} ---")
        
        headers = {
            "Content-Type": "application/json",
            "User-Agent": "DroidGravity-Manager/2.0.0"
        }
        
        try:
            response = requests.post(endpoint, json=data, headers=headers, timeout=5)
            print(f"Status: {response.status_code}")
            print(f"Response: {response.text[:200]}")
            
            if response.status_code == 200 and "access" in response.text.lower():
                print("🎉 SUCCESS! This endpoint works!")
                try:
                    tokens = response.json()
                    return tokens
                except:
                    pass
                    
        except Exception as e:
            print(f"Error: {e}")
    
    return None

def main():
    print("🔍 ТЕСТ ОБМЕНА AUTHORIZATION CODE НА ТОКЕНЫ")
    print("=" * 60)
    
    # Данные из успешного теста
    code = "6788ef1e-3289-43b9-a843-6ed88c38d76e"
    code_verifier = "ehENtc1j8v13J6GkUlNQhmKFnKgAtjadsQ4HsA5Spyo"
    
    print(f"📋 Using data from successful OAuth test:")
    print(f"Code: {code}")
    print(f"Code Verifier: {code_verifier}")
    print()
    
    # Тест 1: Cognito
    cognito_tokens = test_cognito_token_exchange(code, code_verifier)
    
    # Тест 2: Kiro API
    kiro_tokens = test_kiro_api_token_exchange(code, code_verifier)
    
    # Тест 3: Альтернативные endpoints
    alt_tokens = test_alternative_endpoints(code, code_verifier)
    
    print("\n" + "=" * 60)
    print("✅ РЕЗУЛЬТАТЫ:")
    
    if cognito_tokens:
        print("🎉 Cognito token exchange WORKS!")
        print(f"Access Token: {cognito_tokens.get('access_token', 'N/A')[:50]}...")
        print(f"Refresh Token: {cognito_tokens.get('refresh_token', 'N/A')[:50]}...")
    else:
        print("❌ Cognito token exchange failed")
    
    if kiro_tokens:
        print("🎉 Kiro API token exchange WORKS!")
        print(f"Access Token: {kiro_tokens.get('accessToken', 'N/A')[:50]}...")
        print(f"Refresh Token: {kiro_tokens.get('refreshToken', 'N/A')[:50]}...")
    else:
        print("❌ Kiro API token exchange failed")
    
    if alt_tokens:
        print("🎉 Alternative endpoint WORKS!")
    else:
        print("❌ Alternative endpoints failed")
    
    print("\n💡 ВЫВОДЫ:")
    if cognito_tokens or kiro_tokens or alt_tokens:
        print("✅ Token exchange is possible!")
        print("✅ Our OAuth flow is complete and working")
        print("✅ Ready to implement in DroidGravity Manager")
    else:
        print("❌ Token exchange failed - need manual token input")
        print("📋 But OAuth flow works - can get authorization codes")
        print("📋 Manual token input is still viable solution")

if __name__ == "__main__":
    main()