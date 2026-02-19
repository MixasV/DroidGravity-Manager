#!/usr/bin/env python3
"""
Тестирует manual token input в реальном приложении DroidGravity Manager
"""

import time
import subprocess
import sys
import os
from pathlib import Path

def check_app_running():
    """Проверяет, запущено ли приложение"""
    try:
        # Проверяем процессы Windows
        result = subprocess.run(['tasklist', '/FI', 'IMAGENAME eq DroidGravity Manager.exe'], 
                              capture_output=True, text=True, shell=True)
        return 'DroidGravity Manager.exe' in result.stdout
    except:
        return False

def get_data_dir():
    """Получает папку данных приложения"""
    if sys.platform == "win32":
        return Path.home() / "AppData" / "Roaming" / "com.droidgravity.manager"
    elif sys.platform == "darwin":
        return Path.home() / "Library" / "Application Support" / "com.droidgravity.manager"
    else:
        return Path.home() / ".local" / "share" / "com.droidgravity.manager"

def count_kiro_accounts():
    """Подсчитывает количество Kiro аккаунтов"""
    data_dir = get_data_dir()
    accounts_dir = data_dir / "accounts"
    
    if not accounts_dir.exists():
        return 0
    
    kiro_count = 0
    
    # Читаем индекс аккаунтов
    index_file = data_dir / "accounts.json"
    if index_file.exists():
        import json
        with open(index_file, 'r', encoding='utf-8') as f:
            index = json.load(f)
        
        for acc_summary in index.get('accounts', []):
            acc_file = accounts_dir / f"{acc_summary['id']}.json"
            if acc_file.exists():
                with open(acc_file, 'r', encoding='utf-8') as f:
                    acc_data = json.load(f)
                if acc_data.get('provider') == 'kiro':
                    kiro_count += 1
    
    return kiro_count

def main():
    print("🧪 ТЕСТ MANUAL TOKEN INPUT В РЕАЛЬНОМ ПРИЛОЖЕНИИ")
    print("=" * 60)
    
    # Проверяем, запущено ли приложение
    if not check_app_running():
        print("❌ Приложение DroidGravity Manager не запущено!")
        print("📋 ИНСТРУКЦИИ:")
        print("1. Запустите DroidGravity Manager")
        print("2. Перейдите на страницу Accounts")
        print("3. Нажмите 'Add Account'")
        print("4. Выберите 'Kiro' как провайдер")
        print("5. Перейдите на вкладку 'Manual Tokens'")
        print("6. Запустите этот скрипт снова")
        return
    
    print("✅ Приложение запущено")
    
    # Подсчитываем текущие Kiro аккаунты
    initial_count = count_kiro_accounts()
    print(f"📊 Текущее количество Kiro аккаунтов: {initial_count}")
    
    print("\n📋 ИНСТРУКЦИИ ДЛЯ ТЕСТИРОВАНИЯ:")
    print("1. В приложении перейдите: Accounts → Add Account → Kiro → Manual Tokens")
    print("2. Вставьте тестовые токены:")
    print("   Access Token: test-access-token-12345")
    print("   Refresh Token: test-refresh-token-67890")
    print("3. Нажмите 'Add Account'")
    print("4. Проверьте, появился ли новый аккаунт в списке")
    
    print(f"\n⏳ Мониторинг изменений (нажмите Ctrl+C для остановки)...")
    
    try:
        while True:
            time.sleep(2)
            current_count = count_kiro_accounts()
            
            if current_count > initial_count:
                print(f"\n🎉 УСПЕХ! Добавлен новый Kiro аккаунт!")
                print(f"📊 Было: {initial_count}, стало: {current_count}")
                
                # Показываем детали последнего аккаунта
                data_dir = get_data_dir()
                accounts_dir = data_dir / "accounts"
                index_file = data_dir / "accounts.json"
                
                if index_file.exists():
                    import json
                    with open(index_file, 'r', encoding='utf-8') as f:
                        index = json.load(f)
                    
                    # Найдем последний Kiro аккаунт
                    latest_kiro = None
                    latest_time = 0
                    
                    for acc_summary in index.get('accounts', []):
                        acc_file = accounts_dir / f"{acc_summary['id']}.json"
                        if acc_file.exists():
                            with open(acc_file, 'r', encoding='utf-8') as f:
                                acc_data = json.load(f)
                            if (acc_data.get('provider') == 'kiro' and 
                                acc_data.get('created_at', 0) > latest_time):
                                latest_kiro = acc_data
                                latest_time = acc_data.get('created_at', 0)
                    
                    if latest_kiro:
                        print(f"📧 Email: {latest_kiro.get('email', 'N/A')}")
                        print(f"🆔 ID: {latest_kiro.get('id', 'N/A')}")
                        print(f"🔑 Access Token: {latest_kiro.get('token', {}).get('access_token', 'N/A')[:30]}...")
                        print(f"🔄 Refresh Token: {latest_kiro.get('token', {}).get('refresh_token', 'N/A')[:30]}...")
                        print(f"⏰ Created: {latest_kiro.get('created_at', 'N/A')}")
                
                print(f"\n✅ ТЕСТ ПРОШЕЛ УСПЕШНО!")
                break
            elif current_count < initial_count:
                print(f"\n⚠️  Количество аккаунтов уменьшилось: {current_count}")
                initial_count = current_count
            
    except KeyboardInterrupt:
        print(f"\n⏹️  Мониторинг остановлен")
        final_count = count_kiro_accounts()
        if final_count > initial_count:
            print(f"✅ За время мониторинга добавлено аккаунтов: {final_count - initial_count}")
        else:
            print(f"❌ Новые аккаунты не обнаружены")

if __name__ == "__main__":
    main()