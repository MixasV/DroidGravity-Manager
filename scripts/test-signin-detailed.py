#!/usr/bin/env python3
"""
Детальный тест signin endpoint с анализом HTML ответа
"""

import requests
import json
import base64
import hashlib
import secrets
import urllib.parse
import re

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

def analyze_html_response(html_content):
    """Анализ HTML ответа на предмет скрытых данных"""
    print("🔍 АНАЛИЗ HTML ОТВЕТА:")
    print("-" * 30)
    
    # Поиск meta тегов с данными
    meta_patterns = [
        r'<meta name="([^"]*)" content="([^"]*)"',
        r'<meta property="([^"]*)" content="([^"]*)"',
        r'window\.__([^_]+)__\s*=\s*"([^"]*)"',
        r'window\.([a-zA-Z_]+)\s*=\s*"([^"]*)"'
    ]
    
    found_data = {}
    
    for pattern in meta_patterns:
        matches = re.findall(pattern, html_content)
        for match in matches:
            key, value = match
            if any(keyword in key.lower() for keyword in ['token', 'auth', 'user', 'state', 'code', 'redirect']):
                found_data[key] = value
                print(f"  {key}: {value[:100]}...")
    
    # Поиск скриптов с данными
    script_pattern = r'<script[^>]*>(.*?)</script>'
    scripts = re.findall(script_pattern, html_content, re.DOTALL)
    
    for i, script in enumerate(scripts):
        if any(keyword in script.lower() for keyword in ['token', 'auth', 'redirect', 'code']):
            print(f"  Script {i+1}: {script[:200]}...")
    
    return found_data

def test_signin_with_follow_redirects():
    """Тест signin с отслеживанием редиректов"""
    print("=== TESTING SIGNIN WITH REDIRECT FOLLOWING ===")
    
    code_verifier, code_challenge = generate_pkce()
    state = secrets.token_urlsafe(32)
    
    signin_url = f"https://app.kiro.dev/signin?state={state}&code_challenge={code_challenge}&code_challenge_method=S256&redirect_uri=http%3A//localhost%3A3128&redirect_from=KiroIDE"
    
    headers = {
        "Authorization": f"Bearer {ACCESS_TOKEN}",
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36",
        "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
        "Accept-Language": "en-US,en;q=0.9,ru;q=0.8",
        "Cookie": f"access_token={ACCESS_TOKEN}; refresh_token={REFRESH_TOKEN}"
    }
    
    print(f"🔗 URL: {signin_url}")
    print(f"🔑 Access Token: {ACCESS_TOKEN[:50]}...")
    print()
    
    try:
        # Сначала без редиректов
        response = requests.get(signin_url, headers=headers, timeout=10, allow_redirects=False)
        print(f"📋 Initial Response:")
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
                print(f"🎉 FOUND AUTHORIZATION CODE: {code}")
                return code, code_verifier
        
        # Если нет редиректа, анализируем HTML
        if response.status_code == 200:
            html_content = response.text
            print(f"📄 HTML Content Length: {len(html_content)}")
            
            # Анализ HTML
            found_data = analyze_html_response(html_content)
            
            # Теперь попробуем с редиректами
            print("\n--- Following Redirects ---")
            response_with_redirects = requests.get(signin_url, headers=headers, timeout=10, allow_redirects=True)
            print(f"Final Status: {response_with_redirects.status_code}")
            print(f"Final URL: {response_with_redirects.url}")
            
            # Проверим финальный URL на код
            if 'code=' in response_with_redirects.url:
                parsed = urllib.parse.urlparse(response_with_redirects.url)
                query_params = urllib.parse.parse_qs(parsed.query)
                code = query_params.get('code', [None])[0]
                print(f"🎉 FOUND CODE IN FINAL URL: {code}")
                return code, code_verifier
        
    except Exception as e:
        print(f"Error: {e}")
    
    return None, code_verifier

def test_signin_without_tokens():
    """Тест signin без токенов (как новый пользователь)"""
    print("\n=== TESTING SIGNIN WITHOUT TOKENS ===")
    
    code_verifier, code_challenge = generate_pkce()
    state = secrets.token_urlsafe(32)
    
    signin_url = f"https://app.kiro.dev/signin?state={state}&code_challenge={code_challenge}&code_challenge_method=S256&redirect_uri=http%3A//localhost%3A3128&redirect_from=KiroIDE"
    
    headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36",
        "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
        "Accept-Language": "en-US,en;q=0.9,ru;q=0.8"
    }
    
    print(f"🔗 URL: {signin_url}")
    print("🔑 No tokens (new user)")
    
    try:
        response = requests.get(signin_url, headers=headers, timeout=10, allow_redirects=False)
        print(f"Status: {response.status_code}")
        
        if 'location' in response.headers:
            location = response.headers['location']
            print(f"🔗 Redirect Location: {location}")
            
            if 'code=' in location:
                parsed = urllib.parse.urlparse(location)
                query_params = urllib.parse.parse_qs(parsed.query)
                code = query_params.get('code', [None])[0]
                print(f"🎉 FOUND CODE WITHOUT TOKENS: {code}")
                return code, code_verifier
        
        if response.status_code == 200:
            print("📄 Got HTML page (probably login form)")
            # Анализ HTML для формы авторизации
            html_content = response.text
            if 'google' in html_content.lower():
                print("✅ Found Google auth option in HTML")
            if 'github' in html_content.lower():
                print("✅ Found GitHub auth option in HTML")
        
    except Exception as e:
        print(f"Error: {e}")
    
    return None, code_verifier

def main():
    print("🔍 ДЕТАЛЬНЫЙ ТЕСТ SIGNIN ENDPOINT")
    print("=" * 50)
    
    # Тест 1: С токенами и отслеживанием редиректов
    code1, verifier1 = test_signin_with_follow_redirects()
    
    # Тест 2: Без токенов (как новый пользователь)
    code2, verifier2 = test_signin_without_tokens()
    
    print("\n" + "=" * 50)
    print("✅ РЕЗУЛЬТАТЫ:")
    
    if code1:
        print(f"🎉 С токенами получен код: {code1}")
        print(f"📝 Code verifier: {verifier1}")
        print("💡 Можно использовать для автоматического обмена!")
    else:
        print("❌ С токенами код не получен")
    
    if code2:
        print(f"🎉 Без токенов получен код: {code2}")
        print(f"📝 Code verifier: {verifier2}")
    else:
        print("❌ Без токенов код не получен (ожидаемо - нужна авторизация)")
    
    print("\n💡 ВЫВОДЫ:")
    if code1:
        print("✅ Signin endpoint работает с существующими токенами!")
        print("✅ Можно реализовать автоматический flow для авторизованных пользователей")
        print("📋 Нужно обновить наш код для использования этого подхода")
    else:
        print("📋 Signin endpoint возвращает HTML форму")
        print("📋 Нужно симулировать выбор Google и авторизацию")
        print("📋 Или использовать ручной ввод токенов")

if __name__ == "__main__":
    main()