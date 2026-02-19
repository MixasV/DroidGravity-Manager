#!/usr/bin/env python3
"""
Тест InitiateLogin с существующими токенами из перехваченного трафика
"""

import requests
import json
import base64
import hashlib
import secrets
import urllib.parse

# Токены из перехваченного трафика
ACCESS_TOKEN = "aoaAAAAAGmV-ZEyqyeZ87VQwlMd6MAq8rmdtb-qiZ1vu2kQbpaUJ30JkYmkO5HJ8dVNThdKaFAqk_ZOGtJqUUU9ncBkc0:MGQCMFz43s+Fqy4A7hgJvTQI48n8KHE2sD+LPWxcHeR0/1mkgJi+MmN7jrQ+LqEzf2gg6AIwH0+jd0G8S3hqtWjRbm8BIeJ9GVsDvW/B2KId6J4ByYXdiwku0nDzN9pi5z9JS8aL"
REFRESH_TOKEN = "aorAAAAAGoMkoEXvu-yvg4l1jUz6QHxtX4szTtPuF-CX1lMTs_lHPYtw7x5GVCAqysLAoyws9rGPWjhLDgLH19A-ABkc0:MGQCMFOFYdKwEXlX7loRAZwxtx6HwtJqU34lH2FASU5zIgY7NLnuoI6wplVZ2Gv8HapVvgIwIGSRqmqJBZ0s2BaTd9dCfHb346qewv1wMgPmxjbRl0QUsaSAPIcmdVsB+foIHaGF"

def generate_pkce():
    """Генерация PKCE параметров"""
    code_verifier = base64.urlsafe_b64encode(secrets.token_bytes(32)).decode('utf-8').rstrip('=')
    code_challenge = base64.urlsafe_b64encode(
        hashlib.sha256(code_verifier.encode('utf-8')).digest()
    ).decode('utf-8').rstrip('=')
    return code_verifier, code_challenge

def test_initiate_login_with_auth():
    """Тест InitiateLogin с авторизацией через токены"""
    print("=== TESTING INITIATE LOGIN WITH TOKENS ===")
    
    # URL как в перехваченном трафике
    url = "https://app.kiro.dev/service/KiroWebPortalService/operation/InitiateLogin"
    
    code_verifier, code_challenge = generate_pkce()
    state = secrets.token_urlsafe(32)
    
    # Данные запроса
    data = {
        "idp": "Google",
        "redirectUri": "http://localhost:3128",
        "state": state,
        "codeChallenge": code_challenge,
        "codeChallengeMethod": "S256"
    }
    
    # Заголовки с авторизацией
    headers = {
        "Content-Type": "application/x-amz-json-1.1",
        "X-Amz-Target": "KiroWebPortalService.InitiateLogin",
        "Authorization": f"Bearer {ACCESS_TOKEN}",
        "User-Agent": "KiroIDE/1.0.0",
        "Origin": "https://app.kiro.dev",
        "Referer": "https://app.kiro.dev/"
    }
    
    print(f"📋 Request Data:")
    print(f"URL: {url}")
    print(f"Data: {json.dumps(data, indent=2)}")
    print(f"Access Token: {ACCESS_TOKEN[:50]}...")
    print()
    
    try:
        response = requests.post(url, json=data, headers=headers, timeout=10)
        print(f"Status: {response.status_code}")
        print(f"Headers: {dict(response.headers)}")
        print(f"Response: {response.text}")
        
        if response.status_code == 200:
            try:
                json_response = response.json()
                print(f"🎉 SUCCESS! JSON Response:")
                print(json.dumps(json_response, indent=2))
                
                if "redirectUrl" in json_response:
                    redirect_url = json_response["redirectUrl"]
                    print(f"\n🔗 Redirect URL: {redirect_url}")
                    return redirect_url, code_verifier
                    
            except json.JSONDecodeError:
                print("Response is not JSON")
        
    except Exception as e:
        print(f"Error: {e}")
    
    return None, code_verifier

def test_alternative_auth_headers():
    """Тест с различными вариантами авторизации"""
    print("\n=== TESTING ALTERNATIVE AUTH HEADERS ===")
    
    url = "https://app.kiro.dev/service/KiroWebPortalService/operation/InitiateLogin"
    
    code_verifier, code_challenge = generate_pkce()
    state = secrets.token_urlsafe(32)
    
    data = {
        "idp": "Google",
        "redirectUri": "http://localhost:3128",
        "state": state,
        "codeChallenge": code_challenge,
        "codeChallengeMethod": "S256"
    }
    
    # Различные варианты заголовков авторизации
    auth_variants = [
        {"Authorization": f"Bearer {ACCESS_TOKEN}"},
        {"X-Amz-Access-Token": ACCESS_TOKEN},
        {"X-Access-Token": ACCESS_TOKEN},
        {"Cookie": f"access_token={ACCESS_TOKEN}"},
        {"Cookie": f"kiro_access_token={ACCESS_TOKEN}"},
        {"X-Refresh-Token": REFRESH_TOKEN, "X-Access-Token": ACCESS_TOKEN},
    ]
    
    for i, auth_headers in enumerate(auth_variants, 1):
        print(f"\n--- Variant {i}: {list(auth_headers.keys())} ---")
        
        headers = {
            "Content-Type": "application/x-amz-json-1.1",
            "X-Amz-Target": "KiroWebPortalService.InitiateLogin",
            "User-Agent": "KiroIDE/1.0.0",
            **auth_headers
        }
        
        try:
            response = requests.post(url, json=data, headers=headers, timeout=5)
            print(f"Status: {response.status_code}")
            print(f"Response: {response.text[:200]}")
            
            if response.status_code == 200 and "redirectUrl" in response.text:
                print("🎉 SUCCESS! This auth method works!")
                return response.json(), code_verifier
                
        except Exception as e:
            print(f"Error: {e}")
    
    return None, None

def test_direct_signin_with_tokens():
    """Тест прямого обращения к /signin с токенами"""
    print("\n=== TESTING DIRECT SIGNIN WITH TOKENS ===")
    
    code_verifier, code_challenge = generate_pkce()
    state = secrets.token_urlsafe(32)
    
    # URL как в KiroIDE
    signin_url = f"https://app.kiro.dev/signin?state={state}&code_challenge={code_challenge}&code_challenge_method=S256&redirect_uri=http%3A//localhost%3A3128&redirect_from=KiroIDE"
    
    headers = {
        "Authorization": f"Bearer {ACCESS_TOKEN}",
        "User-Agent": "KiroIDE/1.0.0",
        "Cookie": f"access_token={ACCESS_TOKEN}; refresh_token={REFRESH_TOKEN}"
    }
    
    print(f"URL: {signin_url}")
    print(f"Access Token: {ACCESS_TOKEN[:50]}...")
    
    try:
        response = requests.get(signin_url, headers=headers, timeout=10, allow_redirects=False)
        print(f"Status: {response.status_code}")
        print(f"Headers: {dict(response.headers)}")
        
        if 'location' in response.headers:
            location = response.headers['location']
            print(f"🔗 Redirect Location: {location}")
            
            # Проверим, есть ли код в redirect
            if 'code=' in location:
                parsed = urllib.parse.urlparse(location)
                query_params = urllib.parse.parse_qs(parsed.query)
                code = query_params.get('code', [None])[0]
                print(f"🎉 Found authorization code: {code}")
                return code, code_verifier
        
        print(f"Response: {response.text[:300]}")
        
    except Exception as e:
        print(f"Error: {e}")
    
    return None, code_verifier

def main():
    print("🔍 ТЕСТИРОВАНИЕ INITIATE LOGIN С ТОКЕНАМИ")
    print("=" * 60)
    
    print("📋 Используемые токены из перехваченного трафика:")
    print(f"Access Token: {ACCESS_TOKEN[:50]}...")
    print(f"Refresh Token: {REFRESH_TOKEN[:50]}...")
    print()
    
    # Тест 1: InitiateLogin с авторизацией
    redirect_url, verifier1 = test_initiate_login_with_auth()
    
    # Тест 2: Альтернативные заголовки авторизации
    alt_result, verifier2 = test_alternative_auth_headers()
    
    # Тест 3: Прямое обращение к signin с токенами
    code, verifier3 = test_direct_signin_with_tokens()
    
    print("\n" + "=" * 60)
    print("✅ РЕЗУЛЬТАТЫ:")
    
    if redirect_url:
        print(f"🎉 InitiateLogin сработал! Redirect URL: {redirect_url}")
    else:
        print("❌ InitiateLogin не сработал")
    
    if alt_result:
        print(f"🎉 Альтернативная авторизация сработала!")
    else:
        print("❌ Альтернативная авторизация не сработала")
    
    if code:
        print(f"🎉 Получен код из прямого signin: {code}")
        print(f"📝 Можно использовать для обмена на токены с verifier: {verifier3}")
    else:
        print("❌ Прямой signin не дал код")
    
    print("\n💡 СЛЕДУЮЩИЕ ШАГИ:")
    if redirect_url or code:
        print("1. Обновить наш OAuth модуль для использования токенов")
        print("2. Добавить правильные заголовки авторизации")
        print("3. Протестировать полный flow")
    else:
        print("1. Токены могли устареть - нужны свежие")
        print("2. Возможно, нужен другой формат авторизации")
        print("3. Проверить другие endpoints")

if __name__ == "__main__":
    main()