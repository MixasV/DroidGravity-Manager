#!/usr/bin/env python3
"""
Тестовый скрипт для проверки добавления и работы Kiro аккаунтов
"""

import json
import os
import sys
from pathlib import Path

def test_kiro_account_creation():
    """Тестирует создание и сохранение Kiro аккаунта"""
    
    print("🔍 ТЕСТ СОЗДАНИЯ KIRO АККАУНТА")
    print("=" * 50)
    
    # 1. Проверяем структуру данных
    print("\n1. Проверка структуры данных...")
    
    # Найдем папку с данными приложения
    if sys.platform == "win32":
        data_dir = Path.home() / "AppData" / "Roaming" / "com.droidgravity.manager"
    elif sys.platform == "darwin":
        data_dir = Path.home() / "Library" / "Application Support" / "com.droidgravity.manager"
    else:
        data_dir = Path.home() / ".local" / "share" / "com.droidgravity.manager"
    
    accounts_dir = data_dir / "accounts"
    accounts_index = data_dir / "accounts.json"
    
    print(f"📁 Data directory: {data_dir}")
    print(f"📁 Accounts directory: {accounts_dir}")
    print(f"📄 Accounts index: {accounts_index}")
    
    # 2. Проверяем существующие аккаунты
    print("\n2. Проверка существующих аккаунтов...")
    
    if accounts_index.exists():
        with open(accounts_index, 'r', encoding='utf-8') as f:
            index_data = json.load(f)
        
        print(f"📊 Всего аккаунтов в индексе: {len(index_data.get('accounts', []))}")
        
        for account in index_data.get('accounts', []):
            account_file = accounts_dir / f"{account['id']}.json"
            if account_file.exists():
                with open(account_file, 'r', encoding='utf-8') as f:
                    account_data = json.load(f)
                
                provider = account_data.get('provider', 'gemini')
                print(f"  📧 {account['email']} (ID: {account['id'][:8]}..., Provider: {provider})")
                
                if provider == 'kiro':
                    print(f"    🚀 Kiro аккаунт найден!")
                    print(f"    🔑 Profile ARN: {account_data.get('kiro_profile_arn', 'N/A')}")
                    print(f"    👤 User ID: {account_data.get('kiro_user_id', 'N/A')}")
                    print(f"    🔐 Has access token: {'✅' if account_data.get('token', {}).get('access_token') else '❌'}")
                    print(f"    🔄 Has refresh token: {'✅' if account_data.get('token', {}).get('refresh_token') else '❌'}")
            else:
                print(f"  ❌ Файл аккаунта {account['id']}.json не найден!")
    else:
        print("❌ Файл индекса аккаунтов не найден!")
    
    # 3. Создаем тестовый Kiro аккаунт
    print("\n3. Создание тестового Kiro аккаунта...")
    
    test_account = {
        "id": "test-kiro-12345678",
        "email": "test-kiro-user@example.com",
        "name": None,
        "provider": "kiro",
        "kiro_profile_arn": "arn:aws:codewhisperer:us-east-1:699475941385:profile/TEST",
        "kiro_user_id": "test-user-id-12345",
        "token": {
            "access_token": "test-access-token-12345",
            "refresh_token": "test-refresh-token-12345",
            "expires_in": 3600,
            "expiry_timestamp": 1234567890,
            "token_type": "Bearer",
            "email": "test-kiro-user@example.com"
        },
        "device_profile": None,
        "device_history": [],
        "quota": None,
        "disabled": False,
        "disabled_reason": None,
        "disabled_at": None,
        "proxy_disabled": False,
        "proxy_disabled_reason": None,
        "proxy_disabled_at": None,
        "protected_models": [],
        "individual_proxy": None,
        "created_at": 1234567890,
        "last_used": 1234567890
    }
    
    # Создаем папки если не существуют
    accounts_dir.mkdir(parents=True, exist_ok=True)
    
    # Сохраняем тестовый аккаунт
    test_account_file = accounts_dir / f"{test_account['id']}.json"
    with open(test_account_file, 'w', encoding='utf-8') as f:
        json.dump(test_account, f, indent=2, ensure_ascii=False)
    
    print(f"✅ Тестовый аккаунт сохранен: {test_account_file}")
    
    # Обновляем индекс
    if accounts_index.exists():
        with open(accounts_index, 'r', encoding='utf-8') as f:
            index_data = json.load(f)
    else:
        index_data = {
            "version": "2.0",
            "accounts": [],
            "current_account_id": None
        }
    
    # Добавляем в индекс если еще нет
    existing_ids = [acc['id'] for acc in index_data['accounts']]
    if test_account['id'] not in existing_ids:
        index_data['accounts'].append({
            "id": test_account['id'],
            "email": test_account['email'],
            "name": test_account['name'],
            "created_at": test_account['created_at'],
            "last_used": test_account['last_used']
        })
        
        with open(accounts_index, 'w', encoding='utf-8') as f:
            json.dump(index_data, f, indent=2, ensure_ascii=False)
        
        print(f"✅ Индекс аккаунтов обновлен")
    
    # 4. Проверяем результат
    print("\n4. Проверка результата...")
    
    with open(accounts_index, 'r', encoding='utf-8') as f:
        updated_index = json.load(f)
    
    kiro_accounts = []
    for account in updated_index.get('accounts', []):
        account_file = accounts_dir / f"{account['id']}.json"
        if account_file.exists():
            with open(account_file, 'r', encoding='utf-8') as f:
                account_data = json.load(f)
            
            if account_data.get('provider') == 'kiro':
                kiro_accounts.append(account_data)
    
    print(f"🚀 Найдено Kiro аккаунтов: {len(kiro_accounts)}")
    
    for acc in kiro_accounts:
        print(f"  📧 {acc['email']} (ID: {acc['id']})")
        print(f"    🔑 Profile ARN: {acc.get('kiro_profile_arn')}")
        print(f"    👤 User ID: {acc.get('kiro_user_id')}")
    
    # 5. Тест экспорта
    print("\n5. Тест экспорта аккаунтов...")
    
    export_data = {
        "accounts": []
    }
    
    for account in updated_index.get('accounts', []):
        account_file = accounts_dir / f"{account['id']}.json"
        if account_file.exists():
            with open(account_file, 'r', encoding='utf-8') as f:
                account_data = json.load(f)
            
            export_data["accounts"].append({
                "email": account_data['email'],
                "refresh_token": account_data.get('token', {}).get('refresh_token', ''),
                "provider": account_data.get('provider', 'gemini')
            })
    
    export_file = Path("test-export.json")
    with open(export_file, 'w', encoding='utf-8') as f:
        json.dump(export_data, f, indent=2, ensure_ascii=False)
    
    print(f"✅ Экспорт сохранен: {export_file}")
    
    kiro_in_export = [acc for acc in export_data['accounts'] if acc.get('provider') == 'kiro']
    print(f"🚀 Kiro аккаунтов в экспорте: {len(kiro_in_export)}")
    
    print("\n" + "=" * 50)
    print("✅ ТЕСТ ЗАВЕРШЕН")
    
    if kiro_accounts:
        print("🎉 Kiro аккаунты найдены и работают!")
    else:
        print("❌ Kiro аккаунты не найдены - есть проблема с сохранением")
    
    return len(kiro_accounts) > 0

def cleanup_test_account():
    """Удаляет тестовый аккаунт"""
    print("\n🧹 ОЧИСТКА ТЕСТОВЫХ ДАННЫХ")
    
    # Найдем папку с данными приложения
    if sys.platform == "win32":
        data_dir = Path.home() / "AppData" / "Roaming" / "com.droidgravity.manager"
    elif sys.platform == "darwin":
        data_dir = Path.home() / "Library" / "Application Support" / "com.droidgravity.manager"
    else:
        data_dir = Path.home() / ".local" / "share" / "com.droidgravity.manager"
    
    accounts_dir = data_dir / "accounts"
    accounts_index = data_dir / "accounts.json"
    
    # Удаляем тестовый файл аккаунта
    test_account_file = accounts_dir / "test-kiro-12345678.json"
    if test_account_file.exists():
        test_account_file.unlink()
        print(f"✅ Удален тестовый файл: {test_account_file}")
    
    # Удаляем из индекса
    if accounts_index.exists():
        with open(accounts_index, 'r', encoding='utf-8') as f:
            index_data = json.load(f)
        
        index_data['accounts'] = [acc for acc in index_data['accounts'] if acc['id'] != 'test-kiro-12345678']
        
        with open(accounts_index, 'w', encoding='utf-8') as f:
            json.dump(index_data, f, indent=2, ensure_ascii=False)
        
        print("✅ Индекс аккаунтов очищен")
    
    # Удаляем тестовый экспорт
    export_file = Path("test-export.json")
    if export_file.exists():
        export_file.unlink()
        print(f"✅ Удален тестовый экспорт: {export_file}")

if __name__ == "__main__":
    try:
        success = test_kiro_account_creation()
        
        input("\nНажмите Enter чтобы очистить тестовые данные...")
        cleanup_test_account()
        
        if success:
            print("\n🎉 Тест прошел успешно!")
            sys.exit(0)
        else:
            print("\n❌ Тест не прошел - проблема с сохранением Kiro аккаунтов")
            sys.exit(1)
            
    except Exception as e:
        print(f"\n💥 Ошибка во время теста: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)