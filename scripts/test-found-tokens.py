#!/usr/bin/env python3
"""
Тест найденных токенов из браузера
"""

import requests
import json

def test_tokens(access_token, refresh_token, profile_arn=None):
    """Тест токенов на работоспособность"""
    print("🔍 ТЕСТИРОВАНИЕ НАЙДЕННЫХ ТОКЕНОВ")
    print("=" * 50)
    
    print(f"📋 Токены:")
    print(f"Access Token: {access_token[:50]}...")
    print(f"Refresh Token: {refresh_token[:50]}...")
    if profile_arn:
        print(f"Profile ARN: {profile_arn}")
    print()
    
    # Тест 1: Проверка токена через Kiro API
    print("=== ТЕСТ 1: ПРОВЕРКА ЧЕРЕЗ KIRO API ===")
    
    headers = {
        "Authorization": f"Bearer {access_token}",
        "Content-Type": "application/json",
        "User-Agent": "DroidGravity-Manager/2.0.0"
    }
    
    # Попробуем получить информацию о пользователе
    test_urls = [
        "https://app.kiro.dev/api/v1/GetUserInfo",
        "https://app.kiro.dev/service/KiroWebPortalService/operation/GetUserInfo",
        "https://app.kiro.dev/api/user",
        "https://app.kiro.dev/user"
    ]
    
    for url in test_urls:
        print(f"\nТестируем: {url}")
        try:
            response = requests.get(url, headers=headers, timeout=10)
            print(f"Status: {response.status_code}")
            print(f"Response: {response.text[:200]}")
            
            if response.status_code == 200 and "UnknownOperationException" not in response.text:
                print("🎉 SUCCESS! Токен работает!")
                try:
                    user_info = response.json()
                    print(f"User Info: {json.dumps(user_info, indent=2)}")
                except:
                    pass
                break
        except Exception as e:
            print(f"Error: {e}")
    
    # Тест 2: Проверка refresh токена
    print("\n=== ТЕСТ 2: ПРОВЕРКА REFRESH ТОКЕНА ===")
    
    refresh_data = {
        "refresh_token": refresh_token,
        "grant_type": "refresh_token"
    }
    
    refresh_urls = [
        "https://app.kiro.dev/api/v1/RefreshToken",
        "https://app.kiro.dev/service/KiroWebPortalService/operation/RefreshToken",
        "https://kiro-prod-us-east-1.auth.us-east-1.amazoncognito.com/oauth2/token"
    ]
    
    for url in refresh_urls:
        print(f"\nТестируем refresh: {url}")
        try:
            response = requests.post(url, json=refresh_data, headers=headers, timeout=10)
            print(f"Status: {response.status_code}")
            print(f"Response: {response.text[:200]}")
            
            if response.status_code == 200 and "access_token" in response.text.lower():
                print("🎉 SUCCESS! Refresh токен работает!")
                try:
                    new_tokens = response.json()
                    print(f"New Tokens: {json.dumps(new_tokens, indent=2)}")
                except:
                    pass
                break
        except Exception as e:
            print(f"Error: {e}")
    
    print("\n" + "=" * 50)
    print("✅ РЕЗУЛЬТАТ ТЕСТИРОВАНИЯ:")
    print("Если хотя бы один тест прошел успешно - токены рабочие!")
    print("Можно использовать их в DroidGravity Manager")

def main():
    print("🔐 ТЕСТ ТОКЕНОВ ИЗ БРАУЗЕРА")
    print("=" * 40)
    print()
    
    print("Введите токены, которые вы нашли в браузере:")
    print()
    
    access_token = input("Access Token: ").strip()
    if not access_token:
        print("❌ Access Token обязателен!")
        return
    
    refresh_token = input("Refresh Token: ").strip()
    if not refresh_token:
        print("❌ Refresh Token обязателен!")
        return
    
    profile_arn = input("Profile ARN (необязательно): ").strip()
    
    print()
    test_tokens(access_token, refresh_token, profile_arn if profile_arn else None)

if __name__ == "__main__":
    main()