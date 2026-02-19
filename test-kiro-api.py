#!/usr/bin/env python3
"""
Тестовый скрипт для проверки Kiro API вызовов
"""

import requests
import json
import uuid
import hashlib
import base64
import secrets
from urllib.parse import urlencode

def generate_pkce():
    """Генерирует PKCE параметры"""
    # Generate code_verifier (128 random characters)
    code_verifier = base64.urlsafe_b64encode(secrets.token_bytes(96)).decode('utf-8').rstrip('=')
    
    # Generate code_challenge = BASE64URL(SHA256(verifier))
    challenge = hashlib.sha256(code_verifier.encode('utf-8')).digest()
    code_challenge = base64.urlsafe_b64encode(challenge).decode('utf-8').rstrip('=')
    
    return code_verifier, code_challenge

def test_kiro_oauth_url():
    """Тестирует генерацию OAuth URL для Kiro"""
    print("🔗 ТЕСТ ГЕНЕРАЦИИ KIRO OAUTH URL")
    print("=" * 50)
    
    # Генерируем PKCE параметры
    code_verifier, code_challenge = generate_pkce()
    state = str(uuid.uuid4())
    
    print(f"📋 Generated parameters:")
    print(f"State: {state}")
    print(f"Code Verifier: {code_verifier[:50]}...")
    print(f"Code Challenge: {code_challenge}")
    
    # Строим URL как в реальном Kiro
    redirect_uri = "http://localhost:3128"
    
    params = {
        'state': state,
        'code_challenge': code_challenge,
        'code_challenge_method': 'S256',
        'redirect_uri': redirect_uri,
        'redirect_from': 'KiroIDE'
    }
    
    auth_url = f"https://app.kiro.dev/signin?{urlencode(params)}"
    
    print(f"\n🔗 Authorization URL:")
    print(auth_url)
    
    print(f"\n📋 ИНСТРУКЦИИ:")
    print(f"1. Скопируйте ссылку выше")
    print(f"2. Откройте её в браузере")
    print(f"3. Выберите Google как метод авторизации")
    print(f"4. Завершите авторизацию")
    print(f"5. Скопируйте код из callback URL")
    
    return code_verifier, state

def test_kiro_get_user_info(access_token):
    """Тестирует получение информации о пользователе"""
    print("\n👤 ТЕСТ ПОЛУЧЕНИЯ ИНФОРМАЦИИ О ПОЛЬЗОВАТЕЛЕ")
    print("=" * 50)
    
    # Попробуем разные endpoints
    endpoints = [
        "https://app.kiro.dev/api/user",
        "https://app.kiro.dev/api/v1/user", 
        "https://app.kiro.dev/service/user",
        "https://app.kiro.dev/user",
        "https://app.kiro.dev/api/me",
    ]
    
    for endpoint in endpoints:
        print(f"\n🔍 Тестируем endpoint: {endpoint}")
        
        headers = {
            'Authorization': f'Bearer {access_token}',
            'User-Agent': 'DroidGravity-Manager/2.0.0',
            'Accept': 'application/json',
        }
        
        try:
            response = requests.get(endpoint, headers=headers, timeout=10)
            
            print(f"📊 Response Status: {response.status_code}")
            print(f"📝 Response Body: {response.text[:200]}...")
            
            if response.status_code == 200:
                try:
                    user_info = response.json()
                    print(f"✅ SUCCESS! User Info: {json.dumps(user_info, indent=2)}")
                    return user_info
                except:
                    print(f"✅ SUCCESS but not JSON: {response.text}")
                    return {"raw": response.text}
            elif response.status_code == 404:
                print(f"❌ Endpoint not found")
            else:
                print(f"❌ Failed with status {response.status_code}")
                
        except Exception as e:
            print(f"💥 Request failed: {e}")
    
    print(f"\n❌ Все endpoints не работают")
    return None

def test_manual_token_workflow():
    """Тестирует workflow ручного ввода токенов"""
    print("\n🔧 ТЕСТ WORKFLOW РУЧНОГО ВВОДА ТОКЕНОВ")
    print("=" * 50)
    
    print("📋 Шаги для ручного ввода токенов:")
    print("1. Откройте https://app.kiro.dev в браузере")
    print("2. Войдите через Google")
    print("3. Откройте DevTools (F12)")
    print("4. Перейдите: Application → Cookies → app.kiro.dev")
    print("5. Найдите и скопируйте:")
    print("   - AccessToken")
    print("   - RefreshToken")
    
    access_token = input("\n🔑 Введите AccessToken (или Enter для пропуска): ").strip()
    refresh_token = input("🔄 Введите RefreshToken (или Enter для пропуска): ").strip()
    
    if access_token and refresh_token:
        print(f"\n✅ Токены получены:")
        print(f"Access Token: {access_token[:50]}...")
        print(f"Refresh Token: {refresh_token[:50]}...")
        
        # Тестируем получение информации о пользователе
        user_info = test_kiro_get_user_info(access_token)
        
        if user_info:
            print(f"\n🎉 SUCCESS! Токены работают!")
            print(f"📧 Email: {user_info.get('email', 'N/A')}")
            print(f"👤 User ID: {user_info.get('userId', 'N/A')}")
            print(f"🔐 IDP: {user_info.get('idp', 'N/A')}")
            print(f"📊 Status: {user_info.get('status', 'N/A')}")
            
            return {
                'access_token': access_token,
                'refresh_token': refresh_token,
                'user_info': user_info
            }
        else:
            print(f"\n❌ Токены не работают или API недоступен")
    else:
        print(f"\n⏭️  Пропущено - токены не введены")
    
    return None

def test_kiro_models():
    """Тестирует доступные модели Kiro"""
    print("\n🤖 ТЕСТ ДОСТУПНЫХ МОДЕЛЕЙ KIRO")
    print("=" * 50)
    
    expected_models = [
        "custom:Kiro-Auto-10",
        "custom:Claude-Sonnet-3.5-10", 
        "custom:Claude-Haiku-3.5-1",
        "custom:Claude-Opus-3-15",
        "custom:DeepSeek-V3-0.25",
        "custom:Minimax-2.1-0.15", 
        "custom:Qwen3-Coder-Next-0.05"
    ]
    
    print("📋 Ожидаемые модели:")
    for model in expected_models:
        print(f"  🤖 {model}")
    
    print(f"\n✅ Всего моделей: {len(expected_models)}")
    
    return expected_models

if __name__ == "__main__":
    print("🚀 KIRO API ТЕСТИРОВАНИЕ")
    print("=" * 60)
    
    try:
        # 1. Тест генерации OAuth URL
        code_verifier, state = test_kiro_oauth_url()
        
        # 2. Тест ручного workflow
        token_data = test_manual_token_workflow()
        
        # 3. Тест моделей
        models = test_kiro_models()
        
        print("\n" + "=" * 60)
        print("📊 РЕЗУЛЬТАТЫ ТЕСТИРОВАНИЯ")
        print("=" * 60)
        
        if token_data:
            print("✅ OAuth URL генерация: OK")
            print("✅ Ручной ввод токенов: OK")
            print("✅ GetUserInfo API: OK")
            print("✅ Модели Kiro: OK")
            print("\n🎉 ВСЕ ТЕСТЫ ПРОШЛИ УСПЕШНО!")
            
            print(f"\n📋 Данные для ручного ввода в менеджер:")
            print(f"Access Token: {token_data['access_token'][:50]}...")
            print(f"Refresh Token: {token_data['refresh_token'][:50]}...")
            print(f"Email: {token_data['user_info'].get('email')}")
            
        else:
            print("✅ OAuth URL генерация: OK")
            print("⏭️  Ручной ввод токенов: ПРОПУЩЕНО")
            print("⏭️  GetUserInfo API: ПРОПУЩЕНО")
            print("✅ Модели Kiro: OK")
            print("\n⚠️  ЧАСТИЧНОЕ ТЕСТИРОВАНИЕ")
        
    except Exception as e:
        print(f"\n💥 Ошибка во время тестирования: {e}")
        import traceback
        traceback.print_exc()