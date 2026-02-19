#!/usr/bin/env python3
"""
Скрипт для проверки логов DroidGravity Manager на предмет Kiro операций
"""

import os
import sys
from pathlib import Path
import re
from datetime import datetime

def find_log_files():
    """Находит файлы логов приложения"""
    
    # Возможные пути к логам
    possible_paths = []
    
    if sys.platform == "win32":
        # Windows
        possible_paths.extend([
            Path.home() / "AppData" / "Roaming" / "com.droidgravity.manager" / "logs",
            Path.home() / "AppData" / "Local" / "com.droidgravity.manager" / "logs",
            Path("logs"),  # Текущая папка
        ])
    elif sys.platform == "darwin":
        # macOS
        possible_paths.extend([
            Path.home() / "Library" / "Logs" / "com.droidgravity.manager",
            Path.home() / "Library" / "Application Support" / "com.droidgravity.manager" / "logs",
            Path("logs"),
        ])
    else:
        # Linux
        possible_paths.extend([
            Path.home() / ".local" / "share" / "com.droidgravity.manager" / "logs",
            Path.home() / ".cache" / "com.droidgravity.manager" / "logs",
            Path("logs"),
        ])
    
    log_files = []
    
    for path in possible_paths:
        if path.exists() and path.is_dir():
            for log_file in path.glob("*.log"):
                log_files.append(log_file)
            for log_file in path.glob("*.txt"):
                log_files.append(log_file)
    
    return log_files

def analyze_kiro_logs(log_file):
    """Анализирует лог файл на предмет Kiro операций"""
    
    print(f"\n📄 Анализ файла: {log_file}")
    print("-" * 50)
    
    kiro_entries = []
    
    try:
        with open(log_file, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
        
        for i, line in enumerate(lines):
            line_lower = line.lower()
            
            # Ищем Kiro-связанные записи
            if any(keyword in line_lower for keyword in [
                'kiro', 'manual_kiro_token_input', 'complete_kiro_oauth_login',
                'submit_kiro_oauth_code', 'prepare_kiro_oauth_url',
                'kiro-account-added', 'kiro oauth', 'kiro api'
            ]):
                kiro_entries.append({
                    'line_num': i + 1,
                    'content': line.strip(),
                    'timestamp': extract_timestamp(line)
                })
    
    except Exception as e:
        print(f"❌ Ошибка чтения файла: {e}")
        return []
    
    if kiro_entries:
        print(f"🚀 Найдено {len(kiro_entries)} Kiro записей:")
        
        for entry in kiro_entries[-20:]:  # Показываем последние 20
            timestamp = entry['timestamp'] or 'N/A'
            print(f"  [{timestamp}] Line {entry['line_num']}: {entry['content'][:100]}...")
    else:
        print("❌ Kiro записи не найдены")
    
    return kiro_entries

def extract_timestamp(line):
    """Извлекает timestamp из строки лога"""
    
    # Паттерны для разных форматов времени
    patterns = [
        r'\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}',  # ISO format
        r'\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}',   # Standard format
        r'\d{2}:\d{2}:\d{2}',                      # Time only
    ]
    
    for pattern in patterns:
        match = re.search(pattern, line)
        if match:
            return match.group(0)
    
    return None

def check_recent_activity():
    """Проверяет недавнюю активность"""
    
    print("\n⏰ ПРОВЕРКА НЕДАВНЕЙ АКТИВНОСТИ")
    print("=" * 50)
    
    log_files = find_log_files()
    
    if not log_files:
        print("❌ Файлы логов не найдены!")
        print("\n📋 Возможные причины:")
        print("1. Приложение не запускалось")
        print("2. Логирование отключено")
        print("3. Логи в другой папке")
        return
    
    print(f"📁 Найдено файлов логов: {len(log_files)}")
    
    all_kiro_entries = []
    
    for log_file in log_files:
        entries = analyze_kiro_logs(log_file)
        all_kiro_entries.extend(entries)
    
    if all_kiro_entries:
        print(f"\n📊 СВОДКА ПО KIRO АКТИВНОСТИ")
        print("=" * 50)
        
        # Сортируем по времени (приблизительно)
        all_kiro_entries.sort(key=lambda x: x['line_num'])
        
        print(f"🚀 Всего Kiro записей: {len(all_kiro_entries)}")
        
        # Анализируем типы операций
        operations = {}
        for entry in all_kiro_entries:
            content = entry['content'].lower()
            
            if 'manual_kiro_token_input' in content:
                operations['Manual Token Input'] = operations.get('Manual Token Input', 0) + 1
            elif 'complete_kiro_oauth_login' in content:
                operations['Complete OAuth'] = operations.get('Complete OAuth', 0) + 1
            elif 'submit_kiro_oauth_code' in content:
                operations['Submit Code'] = operations.get('Submit Code', 0) + 1
            elif 'prepare_kiro_oauth_url' in content:
                operations['Prepare URL'] = operations.get('Prepare URL', 0) + 1
            elif 'error' in content or 'failed' in content:
                operations['Errors'] = operations.get('Errors', 0) + 1
            elif 'success' in content or 'added' in content:
                operations['Success'] = operations.get('Success', 0) + 1
        
        print(f"\n📋 Типы операций:")
        for op, count in operations.items():
            print(f"  {op}: {count}")
        
        # Показываем последние записи
        print(f"\n📝 Последние 10 записей:")
        for entry in all_kiro_entries[-10:]:
            timestamp = entry['timestamp'] or 'N/A'
            print(f"  [{timestamp}] {entry['content'][:80]}...")
    
    else:
        print(f"\n❌ Kiro активность не обнаружена")
        print(f"\n📋 Это может означать:")
        print(f"1. Kiro команды не вызывались")
        print(f"2. Ошибки происходят до логирования")
        print(f"3. Проблема с регистрацией команд")

def main():
    print("🔍 АНАЛИЗ ЛОГОВ KIRO ОПЕРАЦИЙ")
    print("=" * 60)
    
    try:
        check_recent_activity()
        
        print(f"\n" + "=" * 60)
        print("✅ АНАЛИЗ ЗАВЕРШЕН")
        
        print(f"\n💡 РЕКОМЕНДАЦИИ:")
        print(f"1. Запустите приложение с включенным логированием")
        print(f"2. Попробуйте добавить Kiro аккаунт")
        print(f"3. Запустите этот скрипт снова")
        print(f"4. Проверьте консоль разработчика в приложении")
        
    except Exception as e:
        print(f"\n💥 Ошибка во время анализа: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    main()