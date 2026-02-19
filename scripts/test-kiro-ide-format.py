#!/usr/bin/env python3
"""
Тестирование правильного формата ссылки как в KiroIDE
"""

import requests
import json
import base64
import hashlib
import secrets
import urllib.parse

def generate_pkce():
    """Генерация PKCE параметров"""
    code_verifier = base64.urlsafe_b64encode(secrets.token_bytes(32)).decode('utf-8').rstrip('=')
    code_challenge = base64.urlsafe_b64encode(
        hashlib.sha256(code_verifier.encode('utf-8')).digest()
    ).decode('utf-8').rstrip('=')
    return code_verifier, code_challenge

def test_kiro_ide_format():
    """Тест с форматом ссылки как в KiroIDE"""
    print("=== TESTING KIRO IDE FORMAT ===")
    
    # Генерируем PKCE как в KiroIDE
    code_verifier, code_challenge = generate_pkce()
    state = secrets.token_urlsafe(32)
    
    # Формат ссылки как в KiroIDE
    base_url = "https://app.kiro.dev/signin"
    params = {
        "state": state,
        "code_challenge": code_challenge,
        "code_challenge_method": "S256",
        "redirect_uri": "http://localhost:3128",  # БЕЗ /oauth/callback
        "redirect_from": "KiroIDE"  # Важный параметр!
    }
    
    kiro_ide_url = f"{base_url}?" + urllib.parse.urlencode(params)
    
    print(f"🔗 KiroIDE Format URL:")
    print(kiro_ide_url)
    print()
    
    print(f"📋 PKCE Parameters:")
    print(f"Code Verifier: {code_verifier}")
    print(f"Code Challenge: {code_challenge}")
    print(f"State: {state}")
    print()
    
    # Попробуем сделать запрос к этой ссылке
    try:
        response = requests.get(kiro_ide_url, timeout=10, allow_redirects=False)
        print(f"Status: {response.status_code}")
        print(f"Headers: {dict(response.headers)}")
        
        if 'location' in response.headers:
            location = response.headers['location']
            print(f"Redirect Location: {location}")
            
            # Проверим, есть ли в redirect токены или коды
            if 'code=' in location:
                parsed = urllib.parse.urlparse(location)
                query_params = urllib.parse.parse_qs(parsed.query)
                print(f"🎉 Found code in redirect: {query_params.get('code', ['N/A'])[0]}")
                return query_params.get('code', [None])[0], code_verifier
        
        print(f"Response: {response.text[:300]}")
        
    except Exception as e:
        print(f"Error: {e}")
    
    return None, code_verifier

def test_initiate_login_with_kiro_format():
    """Тест InitiateLogin с правильными параметрами"""
    print("\n=== TESTING INITIATE LOGIN WITH KIRO FORMAT ===")
    
    url = "https://app.kiro.dev/api/v1/InitiateLogin"
    
    code_verifier, code_challenge = generate_pkce()
    state = secrets.token_urlsafe(32)
    
    # Данные как в KiroIDE
    data = {
        "idp": "Google",
        "redirectUri": "http://localhost:3128",  # БЕЗ /oauth/callback
        "state": state,
        "codeChallenge": code_challenge,
        "codeChallengeMethod": "S256"
    }
    
    headers = {
        "Content-Type": "application/x-amz-json-1.1",
        "X-Amz-Target": "KiroWebPortalService.InitiateLogin",
        "User-Agent": "KiroIDE/1.0.0",
        "Origin": "https://app.kiro.dev",
        "Referer": "https://app.kiro.dev/"
    }
    
    try:
        response = requests.post(url, json=data, headers=headers, timeout=10)
        print(f"Status: {response.status_code}")
        print(f"Response: {response.text[:300]}")
        
        if response.status_code == 200 and "redirectUrl" in response.text:
            json_response = response.json()
            redirect_url = json_response.get("redirectUrl", "")
            print(f"🎉 SUCCESS! Got redirect URL: {redirect_url}")
            return redirect_url, code_verifier
            
    except Exception as e:
        print(f"Error: {e}")
    
    return None, code_verifier

def analyze_captured_urls():
    """Анализ URL из перехваченного трафика"""
    print("\n=== ANALYZING CAPTURED URLS ===")
    
    # URL из твоего примера
    example_url = "https://app.kiro.dev/signin?state=fcdd7bd1-bca2-41fb-9630-9abf6671e0aa&code_challenge=1KiyFO-M8bdCW8ztGRVgUCehm37gNdRDGI_L6RRrJhc&code_challenge_method=S256&redirect_uri=http://localhost:3128&redirect_from=KiroIDE"
    
    parsed = urllib.parse.urlparse(example_url)
    params = urllib.parse.parse_qs(parsed.query)
    
    print("📋 Параметры из твоего примера:")
    for key, value in params.items():
        print(f"  {key}: {value[0]}")
    
    print(f"\n🔍 Ключевые отличия от нашего подхода:")
    print("1. Используется app.kiro.dev/signin вместо Cognito")
    print("2. redirect_uri без /oauth/callback?login_option=google")
    print("3. Есть параметр redirect_from=KiroIDE")
    print("4. Нет client_id и identity_provider")

def main():
    print("🔍 Тестирование формата KiroIDE")
    print("=" * 50)
    
    # Анализ захваченных URL
    analyze_captured_urls()
    
    # Тест формата KiroIDE
    code, verifier = test_kiro_ide_format()
    
    # Тест InitiateLogin
    redirect_url, verifier2 = test_initiate_login_with_kiro_format()
    
    print("\n" + "=" * 50)
    print("✅ Результаты:")
    
    if code:
        print(f"🎉 Получен код из KiroIDE формата: {code}")
        print(f"📝 Можно использовать для обмена на токены")
    else:
        print("❌ Код не получен из KiroIDE формата")
    
    if redirect_url:
        print(f"🎉 Получен redirect URL из InitiateLogin: {redirect_url}")
    else:
        print("❌ InitiateLogin не сработал")
    
    print("\n💡 Следующие шаги:")
    print("1. Обновить наш OAuth модуль для использования app.kiro.dev/signin")
    print("2. Убрать /oauth/callback?login_option=google из redirect_uri")
    print("3. Добавить параметр redirect_from=KiroIDE")
    print("4. Протестировать с реальным кодом")

if __name__ == "__main__":
    main()