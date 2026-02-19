#!/usr/bin/env python3
"""
Руководство по ручной интеграции Kiro в DroidGravity Manager
"""

import webbrowser
import hashlib
import base64
import secrets
import string
from urllib.parse import urlencode

def generate_pkce():
    """Generate PKCE code_verifier and code_challenge"""
    alphabet = string.ascii_letters + string.digits
    verifier = ''.join(secrets.choice(alphabet) for _ in range(128))
    
    challenge = base64.urlsafe_b64encode(
        hashlib.sha256(verifier.encode()).digest()
    ).decode().rstrip('=')
    
    return verifier, challenge

def generate_kiro_auth_url():
    """Generate correct Kiro authorization URL"""
    code_verifier, code_challenge = generate_pkce()
    state = secrets.token_urlsafe(32)
    
    # Correct Cognito URL with all required parameters
    cognito_url = "https://kiro-prod-us-east-1.auth.us-east-1.amazoncognito.com"
    client_id = "59bd15eh40ee7pc20h0bkcu7id"
    redirect_uri = "http://localhost:3128/oauth/callback?login_option=google"
    
    params = {
        'client_id': client_id,
        'response_type': 'code',
        'scope': 'email openid',
        'redirect_uri': redirect_uri,
        'state': state,
        'code_challenge': code_challenge,
        'code_challenge_method': 'S256',
        'identity_provider': 'Google'
    }
    
    auth_url = f"{cognito_url}/oauth2/authorize?" + urlencode(params)
    
    return auth_url, code_verifier, state

def main():
    print("🚀 KIRO MANUAL INTEGRATION GUIDE")
    print("=" * 60)
    print("Руководство по ручной интеграции Kiro в DroidGravity Manager")
    print()
    
    print("📋 ПРОБЛЕМА:")
    print("- Kiro использует AWS Cognito с private client")
    print("- Требуется client_secret, который мы не можем получить")
    print("- Kiro API endpoints возвращают UnknownOperationException")
    print()
    
    print("✅ ЧТО РАБОТАЕТ:")
    print("- Генерация правильного Cognito authorization URL")
    print("- Авторизация через Google")
    print("- Получение authorization code")
    print()
    
    print("🔧 РЕШЕНИЕ:")
    print("Ручное получение токенов через браузер DevTools")
    print()
    
    # Generate auth URL
    auth_url, code_verifier, state = generate_kiro_auth_url()
    
    print("ШАГИ ДЛЯ РУЧНОЙ ИНТЕГРАЦИИ:")
    print("=" * 40)
    
    print("\n1️⃣  ОТКРОЙТЕ AUTHORIZATION URL:")
    print(f"   {auth_url}")
    print()
    
    print("2️⃣  ОТКРОЙТЕ DEVTOOLS (F12) В БРАУЗЕРЕ")
    print("   - Перейдите на вкладку Network")
    print("   - Включите запись сетевого трафика")
    print()
    
    print("3️⃣  ЗАВЕРШИТЕ АВТОРИЗАЦИЮ")
    print("   - Нажмите кнопку авторизации")
    print("   - Выберите Google аккаунт")
    print("   - Подтвердите разрешения")
    print()
    
    print("4️⃣  НАЙДИТЕ ТОКЕНЫ В DEVTOOLS")
    print("   Ищите в Network tab запросы к:")
    print("   - cognito.com/oauth2/token")
    print("   - app.kiro.dev/service/...")
    print("   - Или в Response содержащие 'access_token'")
    print()
    
    print("5️⃣  АЛЬТЕРНАТИВНО - ПРОВЕРЬТЕ STORAGE")
    print("   - Application tab -> Local Storage")
    print("   - Application tab -> Session Storage")
    print("   - Ищите ключи: token, access_token, auth, kiro")
    print()
    
    print("6️⃣  СКОПИРУЙТЕ ТОКЕНЫ")
    print("   Нужны:")
    print("   - access_token (обязательно)")
    print("   - refresh_token (желательно)")
    print("   - expires_in (опционально)")
    print()
    
    print("7️⃣  ДОБАВЬТЕ В DROIDGRAVITY MANAGER")
    print("   - Откройте DroidGravity Manager")
    print("   - Add Account -> Kiro -> Token tab")
    print("   - Вставьте токены")
    print()
    
    print("🔗 ОТКРЫТЬ AUTHORIZATION URL?")
    open_browser = input("Открыть браузер с URL авторизации? (y/n): ").strip().lower()
    
    if open_browser == 'y':
        print("\n🌐 Открываю браузер...")
        webbrowser.open(auth_url)
        print()
        print("📝 СОХРАНИТЕ ЭТИ ДАННЫЕ ДЛЯ ОТЛАДКИ:")
        print(f"Code Verifier: {code_verifier}")
        print(f"State: {state}")
        print(f"Authorization URL: {auth_url}")
        
        # Save to file
        with open("kiro_auth_debug.txt", "w") as f:
            f.write(f"Kiro Authorization Debug Info\n")
            f.write(f"Generated: {__import__('datetime').datetime.now()}\n\n")
            f.write(f"Code Verifier: {code_verifier}\n")
            f.write(f"State: {state}\n")
            f.write(f"Authorization URL: {auth_url}\n")
        
        print(f"\n💾 Данные сохранены в 'kiro_auth_debug.txt'")
    
    print("\n" + "=" * 60)
    print("🎯 СЛЕДУЮЩИЕ ШАГИ РАЗРАБОТКИ:")
    print("=" * 60)
    
    print("\n1. ОБНОВИТЬ RUST КОД:")
    print("   ✅ Использовать правильный Cognito URL")
    print("   ✅ Добавить identity_provider=Google")
    print("   ✅ Обработать callback на /oauth/callback")
    print("   ⏳ Добавить ручной ввод токенов")
    print()
    
    print("2. ОБНОВИТЬ UI:")
    print("   ⏳ Добавить поле для ручного ввода токенов")
    print("   ⏳ Показать инструкции пользователю")
    print("   ⏳ Добавить кнопку 'Open DevTools Guide'")
    print()
    
    print("3. ТЕСТИРОВАНИЕ:")
    print("   ⏳ Протестировать с реальными токенами")
    print("   ⏳ Проверить Kiro API endpoints")
    print("   ⏳ Реализовать refresh токенов")
    print()
    
    print("4. БУДУЩИЕ УЛУЧШЕНИЯ:")
    print("   🔮 Найти способ получить client_secret")
    print("   🔮 Автоматизировать извлечение токенов")
    print("   🔮 Reverse engineer Kiro desktop app")
    print()
    
    print("✨ ИНТЕГРАЦИЯ ГОТОВА К ТЕСТИРОВАНИЮ!")
    print("Используйте ручной ввод токенов для проверки работоспособности.")

if __name__ == "__main__":
    main()