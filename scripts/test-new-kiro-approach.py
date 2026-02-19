#!/usr/bin/env python3
"""
Тест нового подхода с правильной ссылкой как в KiroIDE
"""

import base64
import hashlib
import secrets
import urllib.parse
import webbrowser

def generate_pkce():
    """Генерация PKCE параметров как в нашем коде"""
    code_verifier = base64.urlsafe_b64encode(secrets.token_bytes(32)).decode('utf-8').rstrip('=')
    code_challenge = base64.urlsafe_b64encode(
        hashlib.sha256(code_verifier.encode('utf-8')).digest()
    ).decode('utf-8').rstrip('=')
    return code_verifier, code_challenge

def generate_kiro_signin_url():
    """Генерация ссылки как в нашем обновленном коде"""
    code_verifier, code_challenge = generate_pkce()
    state = secrets.token_urlsafe(32)
    
    # Формат как в нашем обновленном коде
    kiro_signin_url = "https://app.kiro.dev/signin"
    redirect_uri = "http://localhost:3128"  # Простой, без /oauth/callback
    
    auth_url = f"{kiro_signin_url}?state={state}&code_challenge={code_challenge}&code_challenge_method=S256&redirect_uri={urllib.parse.quote(redirect_uri)}&redirect_from=KiroIDE"
    
    return auth_url, code_verifier, state

def main():
    print("🔍 ТЕСТ НОВОГО ПОДХОДА KIRO")
    print("=" * 40)
    
    # Генерируем ссылку как в нашем коде
    auth_url, code_verifier, state = generate_kiro_signin_url()
    
    print("📋 Сгенерированные параметры:")
    print(f"State: {state}")
    print(f"Code Verifier: {code_verifier}")
    print(f"Code Challenge: {code_verifier}")  # Показываем только начало
    print()
    
    print("🔗 Сгенерированная ссылка:")
    print(auth_url)
    print()
    
    print("📝 Сравнение с твоим примером:")
    your_example = "https://app.kiro.dev/signin?state=fcdd7bd1-bca2-41fb-9630-9abf6671e0aa&code_challenge=1KiyFO-M8bdCW8ztGRVgUCehm37gNdRDGI_L6RRrJhc&code_challenge_method=S256&redirect_uri=http://localhost:3128&redirect_from=KiroIDE"
    
    # Парсим оба URL для сравнения
    our_parsed = urllib.parse.urlparse(auth_url)
    our_params = urllib.parse.parse_qs(our_parsed.query)
    
    your_parsed = urllib.parse.urlparse(your_example)
    your_params = urllib.parse.parse_qs(your_parsed.query)
    
    print("Наши параметры:")
    for key, value in our_params.items():
        print(f"  {key}: {value[0][:50]}...")
    
    print("\nТвои параметры:")
    for key, value in your_params.items():
        print(f"  {key}: {value[0][:50]}...")
    
    print("\n✅ Проверка соответствия:")
    checks = [
        ("URL base", our_parsed.netloc == your_parsed.netloc and our_parsed.path == your_parsed.path),
        ("redirect_uri", our_params.get('redirect_uri', [''])[0] == your_params.get('redirect_uri', [''])[0]),
        ("code_challenge_method", our_params.get('code_challenge_method', [''])[0] == your_params.get('code_challenge_method', [''])[0]),
        ("redirect_from", our_params.get('redirect_from', [''])[0] == your_params.get('redirect_from', [''])[0]),
        ("state format", len(our_params.get('state', [''])[0]) > 0),
        ("code_challenge format", len(our_params.get('code_challenge', [''])[0]) > 0)
    ]
    
    for check_name, result in checks:
        status = "✅" if result else "❌"
        print(f"  {status} {check_name}")
    
    print("\n🎯 СЛЕДУЮЩИЕ ШАГИ:")
    print("1. Запустить DroidGravity Manager")
    print("2. Нажать 'Add Account' -> OAuth -> 'Start OAuth'")
    print("3. Проверить, что генерируется правильная ссылка")
    print("4. Открыть ссылку в браузере")
    print("5. Выбрать Google на сайте Kiro")
    print("6. Авторизоваться")
    print("7. Проверить, что callback приходит на http://localhost:3128")
    print("8. Если автоматический обмен не работает - использовать Manual Token Input")
    
    print(f"\n💡 Для тестирования можешь открыть эту ссылку:")
    print(f"   {auth_url}")
    print(f"\n   И посмотреть, ведет ли она на сайт Kiro с выбором Google")

if __name__ == "__main__":
    main()