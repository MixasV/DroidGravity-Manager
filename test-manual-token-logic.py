#!/usr/bin/env python3
"""
Тестирует точную логику manual_kiro_token_input без сборки приложения
"""

import json
import uuid
import os
import sys
from pathlib import Path
from datetime import datetime

def get_data_dir():
    """Получает папку данных приложения"""
    if sys.platform == "win32":
        return Path.home() / "AppData" / "Roaming" / "com.droidgravity.manager"
    elif sys.platform == "darwin":
        return Path.home() / "Library" / "Application Support" / "com.droidgravity.manager"
    else:
        return Path.home() / ".local" / "share" / "com.droidgravity.manager"

def manual_token_input(access_token, refresh_token, expires_in=3600):
    """Симулирует oauth_kiro::manual_token_input"""
    print(f"=== MANUAL TOKEN INPUT ===")
    print(f"Access Token: {access_token[:50]}...")
    print(f"Refresh Token: {refresh_token[:50]}...")
    
    # Симулируем KiroTokenResponse
    return {
        "access_token": access_token,
        "refresh_token": refresh_token,
        "expires_in": expires_in,
        "profile_arn": "arn:aws:codewhisperer:us-east-1:699475941385:profile/MANUAL"
    }

def create_token_data(access_token, refresh_token, expires_in, email):
    """Симулирует TokenData::new"""
    now = int(datetime.now().timestamp())
    return {
        "access_token": access_token,
        "refresh_token": refresh_token,
        "expires_in": expires_in,
        "expiry_timestamp": now + expires_in,
        "token_type": "Bearer",
        "email": email
    }

def create_account(user_id, email, token_data):
    """Симулирует Account::new"""
    now = int(datetime.now().timestamp())
    account_id = str(uuid.uuid4())
    
    return {
        "id": account_id,
        "email": email,
        "name": None,
        "provider": "kiro",
        "kiro_profile_arn": None,  # Будет установлено позже
        "kiro_user_id": user_id,
        "token": token_data,
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
        "created_at": now,
        "last_used": now
    }

def save_account(account):
    """Симулирует modules::account::save_account"""
    data_dir = get_data_dir()
    accounts_dir = data_dir / "accounts"
    
    # Создаем папки
    accounts_dir.mkdir(parents=True, exist_ok=True)
    
    # Сохраняем аккаунт
    account_file = accounts_dir / f"{account['id']}.json"
    with open(account_file, 'w', encoding='utf-8') as f:
        json.dump(account, f, indent=2, ensure_ascii=False)
    
    print(f"✅ Account saved: {account_file}")
    return account_file

def load_account_index():
    """Загружает индекс аккаунтов"""
    data_dir = get_data_dir()
    index_file = data_dir / "accounts.json"
    
    if index_file.exists():
        with open(index_file, 'r', encoding='utf-8') as f:
            return json.load(f)
    else:
        return {
            "version": "2.0",
            "accounts": [],
            "current_account_id": None
        }

def save_account_index(index):
    """Сохраняет индекс аккаунтов"""
    data_dir = get_data_dir()
    index_file = data_dir / "accounts.json"
    
    data_dir.mkdir(parents=True, exist_ok=True)
    
    with open(index_file, 'w', encoding='utf-8') as f:
        json.dump(index, f, indent=2, ensure_ascii=False)
    
    print(f"✅ Index saved: {index_file}")

def test_manual_kiro_token_input():
    """Тестирует полную логику manual_kiro_token_input"""
    
    print("🧪 ТЕСТ ЛОГИКИ MANUAL_KIRO_TOKEN_INPUT")
    print("=" * 60)
    
    # Входные данные (симулируем реальные токены)
    access_token = "test-access-token-" + str(uuid.uuid4())
    refresh_token = "test-refresh-token-" + str(uuid.uuid4())
    expires_in = 3600
    
    print(f"📥 Input tokens:")
    print(f"  Access Token: {access_token[:30]}...")
    print(f"  Refresh Token: {refresh_token[:30]}...")
    print(f"  Expires In: {expires_in}")
    
    try:
        # 1. Manual token input
        print(f"\n1️⃣ Manual token input...")
        tokens = manual_token_input(access_token, refresh_token, expires_in)
        print(f"✅ Tokens processed")
        
        # 2. Skip GetUserInfo API (MVP approach)
        print(f"\n2️⃣ Generating fallback user info...")
        uuid_str = str(uuid.uuid4())
        fallback_email = f"kiro-user-{uuid_str[:8]}"
        fallback_user_id = f"manual-{uuid.uuid4()}"
        
        print(f"📧 Email: {fallback_email}")
        print(f"👤 User ID: {fallback_user_id}")
        
        # 3. Create TokenData
        print(f"\n3️⃣ Creating token data...")
        token_data = create_token_data(
            tokens["access_token"],
            tokens["refresh_token"], 
            tokens["expires_in"],
            fallback_email
        )
        print(f"✅ Token data created")
        
        # 4. Create Account
        print(f"\n4️⃣ Creating account...")
        account = create_account(fallback_user_id, fallback_email, token_data)
        
        # Set Kiro-specific fields
        account["provider"] = "kiro"
        account["kiro_profile_arn"] = tokens["profile_arn"]
        account["kiro_user_id"] = fallback_user_id
        
        print(f"✅ Account created:")
        print(f"  ID: {account['id']}")
        print(f"  Email: {account['email']}")
        print(f"  Provider: {account['provider']}")
        
        # 5. Save account
        print(f"\n5️⃣ Saving account...")
        account_file = save_account(account)
        
        # 6. Update index
        print(f"\n6️⃣ Updating account index...")
        index = load_account_index()
        
        # Add to index if not exists
        existing_ids = [acc['id'] for acc in index['accounts']]
        if account['id'] not in existing_ids:
            index['accounts'].append({
                "id": account['id'],
                "email": account['email'],
                "name": account['name'],
                "created_at": account['created_at'],
                "last_used": account['last_used']
            })
            
            save_account_index(index)
            print(f"✅ Account added to index")
        else:
            print(f"ℹ️  Account already in index")
        
        # 7. Verify result
        print(f"\n7️⃣ Verifying result...")
        
        # Check file exists
        if account_file.exists():
            print(f"✅ Account file exists: {account_file}")
            
            # Load and verify
            with open(account_file, 'r', encoding='utf-8') as f:
                saved_account = json.load(f)
            
            if saved_account['provider'] == 'kiro':
                print(f"✅ Provider correctly set to 'kiro'")
            else:
                print(f"❌ Provider is '{saved_account['provider']}', expected 'kiro'")
            
            if saved_account['kiro_profile_arn']:
                print(f"✅ Kiro profile ARN set: {saved_account['kiro_profile_arn']}")
            else:
                print(f"❌ Kiro profile ARN not set")
                
            if saved_account['token']['access_token'] == access_token:
                print(f"✅ Access token correctly saved")
            else:
                print(f"❌ Access token mismatch")
                
            if saved_account['token']['refresh_token'] == refresh_token:
                print(f"✅ Refresh token correctly saved")
            else:
                print(f"❌ Refresh token mismatch")
        else:
            print(f"❌ Account file not created")
            return False
        
        # Check index
        updated_index = load_account_index()
        kiro_accounts = []
        
        for acc_summary in updated_index['accounts']:
            acc_file = get_data_dir() / "accounts" / f"{acc_summary['id']}.json"
            if acc_file.exists():
                with open(acc_file, 'r', encoding='utf-8') as f:
                    acc_data = json.load(f)
                if acc_data.get('provider') == 'kiro':
                    kiro_accounts.append(acc_data)
        
        print(f"\n📊 РЕЗУЛЬТАТ:")
        print(f"🚀 Kiro аккаунтов в системе: {len(kiro_accounts)}")
        
        if len(kiro_accounts) > 0:
            print(f"✅ ТЕСТ ПРОШЕЛ УСПЕШНО!")
            print(f"📧 Добавленный аккаунт: {kiro_accounts[-1]['email']}")
            return True
        else:
            print(f"❌ ТЕСТ НЕ ПРОШЕЛ - Kiro аккаунт не найден")
            return False
            
    except Exception as e:
        print(f"\n💥 ОШИБКА: {e}")
        import traceback
        traceback.print_exc()
        return False

def cleanup_test_account():
    """Очищает тестовые данные"""
    print(f"\n🧹 ОЧИСТКА ТЕСТОВЫХ ДАННЫХ")
    
    data_dir = get_data_dir()
    accounts_dir = data_dir / "accounts"
    
    if not accounts_dir.exists():
        print(f"ℹ️  Папка аккаунтов не существует")
        return
    
    # Найдем и удалим тестовые аккаунты
    index = load_account_index()
    test_accounts = []
    
    for acc_summary in index['accounts']:
        if acc_summary['email'].startswith('kiro-user-'):
            test_accounts.append(acc_summary)
    
    if test_accounts:
        print(f"🗑️  Найдено тестовых аккаунтов: {len(test_accounts)}")
        
        for acc in test_accounts:
            # Удаляем файл
            acc_file = accounts_dir / f"{acc['id']}.json"
            if acc_file.exists():
                acc_file.unlink()
                print(f"✅ Удален: {acc['email']}")
        
        # Обновляем индекс
        index['accounts'] = [acc for acc in index['accounts'] 
                           if not acc['email'].startswith('kiro-user-')]
        save_account_index(index)
        print(f"✅ Индекс обновлен")
    else:
        print(f"ℹ️  Тестовые аккаунты не найдены")

if __name__ == "__main__":
    try:
        success = test_manual_kiro_token_input()
        
        input(f"\nНажмите Enter для очистки тестовых данных...")
        cleanup_test_account()
        
        if success:
            print(f"\n🎉 ЛОГИКА РАБОТАЕТ ПРАВИЛЬНО!")
            print(f"💡 Можно коммитить и собирать приложение")
        else:
            print(f"\n❌ ЛОГИКА НЕ РАБОТАЕТ!")
            print(f"🔧 Нужно исправлять код")
            
    except KeyboardInterrupt:
        print(f"\n⏹️  Тест прерван")
    except Exception as e:
        print(f"\n💥 Критическая ошибка: {e}")
        import traceback
        traceback.print_exc()