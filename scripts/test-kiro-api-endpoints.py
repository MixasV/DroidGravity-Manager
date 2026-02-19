#!/usr/bin/env python3
"""
Тестирование правильных Kiro API endpoints на основе перехваченного трафика
"""

import requests
import json
import sys

def test_initiate_login():
    """Тест InitiateLogin API"""
    print("=== TESTING INITIATE LOGIN ===")
    
    # Параметры из перехваченного трафика
    url = "https://app.kiro.dev/api/v1/InitiateLogin"
    
    data = {
        "idp": "Google",
        "redirectUri": "http://localhost:3128/oauth/callback?login_option=google",
        "state": "test-state-12345",
        "codeChallenge": "test-code-challenge-12345",
        "codeChallengeMethod": "S256"
    }
    
    headers = {
        "Content-Type": "application/json",
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
    }
    
    try:
        response = requests.post(url, json=data, headers=headers, timeout=10)
        print(f"Status: {response.status_code}")
        print(f"Headers: {dict(response.headers)}")
        print(f"Response: {response.text[:500]}")
        
        if response.status_code == 200:
            try:
                json_response = response.json()
                print(f"JSON Response: {json.dumps(json_response, indent=2)}")
                return json_response
            except:
                print("Response is not JSON")
        
    except Exception as e:
        print(f"Error: {e}")
    
    return None

def test_get_token(code="test-code", code_verifier="test-verifier"):
    """Тест GetToken API"""
    print("\n=== TESTING GET TOKEN ===")
    
    # Параметры из перехваченного трафика
    url = "https://app.kiro.dev/api/v1/GetToken"
    
    data = {
        "code": code,
        "code_verifier": code_verifier,
        "redirect_uri": "http://localhost:3128/oauth/callback?login_option=google"
    }
    
    headers = {
        "Content-Type": "application/json",
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
    }
    
    try:
        response = requests.post(url, json=data, headers=headers, timeout=10)
        print(f"Status: {response.status_code}")
        print(f"Headers: {dict(response.headers)}")
        print(f"Response: {response.text[:500]}")
        
        if response.status_code == 200:
            try:
                json_response = response.json()
                print(f"JSON Response: {json.dumps(json_response, indent=2)}")
                return json_response
            except:
                print("Response is not JSON")
        
    except Exception as e:
        print(f"Error: {e}")
    
    return None

def test_alternative_endpoints():
    """Тест альтернативных endpoints"""
    print("\n=== TESTING ALTERNATIVE ENDPOINTS ===")
    
    endpoints = [
        "https://app.kiro.dev/api/InitiateLogin",
        "https://app.kiro.dev/InitiateLogin", 
        "https://api.kiro.dev/v1/InitiateLogin",
        "https://kiro-prod-us-east-1.auth.us-east-1.amazoncognito.com/oauth2/token"
    ]
    
    for endpoint in endpoints:
        print(f"\nTesting: {endpoint}")
        try:
            response = requests.post(endpoint, json={"test": "data"}, timeout=5)
            print(f"Status: {response.status_code}")
            print(f"Response: {response.text[:200]}")
        except Exception as e:
            print(f"Error: {e}")

def main():
    print("🔍 Тестирование Kiro API endpoints")
    print("=" * 50)
    
    # Тест InitiateLogin
    initiate_result = test_initiate_login()
    
    # Тест GetToken
    get_token_result = test_get_token()
    
    # Тест альтернативных endpoints
    test_alternative_endpoints()
    
    print("\n" + "=" * 50)
    print("✅ Тестирование завершено")
    
    if len(sys.argv) > 1 and sys.argv[1] == "with-real-code":
        print("\n🔄 Тестирование с реальным кодом...")
        if len(sys.argv) > 2:
            real_code = sys.argv[2]
            real_verifier = sys.argv[3] if len(sys.argv) > 3 else "test-verifier"
            test_get_token(real_code, real_verifier)

if __name__ == "__main__":
    main()