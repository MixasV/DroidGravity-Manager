#!/usr/bin/env python3
"""
Тестирование различных X-Amz-Target для Kiro API
"""

import requests
import json

def test_service_target(target, operation_data):
    """Тест конкретного service target"""
    print(f"\n=== TESTING {target} ===")
    
    url = "https://app.kiro.dev/"
    
    headers = {
        "Content-Type": "application/x-amz-json-1.1",
        "X-Amz-Target": target,
        "User-Agent": "aws-sdk-js/2.1691.0 linux/v18.20.4 promise",
        "Accept": "application/json"
    }
    
    try:
        response = requests.post(url, json=operation_data, headers=headers, timeout=10)
        print(f"Status: {response.status_code}")
        print(f"Response: {response.text[:300]}")
        
        if response.status_code == 200 and "UnknownOperationException" not in response.text:
            print("🎉 SUCCESS! This target works!")
            return True
            
    except Exception as e:
        print(f"Error: {e}")
    
    return False

def main():
    print("🔍 Тестирование различных X-Amz-Target для Kiro API")
    print("=" * 60)
    
    # Данные для InitiateLogin
    initiate_data = {
        "idp": "Google",
        "redirectUri": "http://localhost:3128/oauth/callback?login_option=google",
        "state": "test-state-12345",
        "codeChallenge": "test-code-challenge-12345",
        "codeChallengeMethod": "S256"
    }
    
    # Данные для GetToken
    token_data = {
        "code": "test-code-12345",
        "code_verifier": "test-verifier-12345",
        "redirect_uri": "http://localhost:3128/oauth/callback?login_option=google"
    }
    
    # Различные варианты service targets
    targets_to_test = [
        ("KiroWebPortalService.InitiateLogin", initiate_data),
        ("KiroWebPortalService.GetToken", token_data),
        ("KiroService.InitiateLogin", initiate_data),
        ("KiroService.GetToken", token_data),
        ("InitiateLogin", initiate_data),
        ("GetToken", token_data),
        ("Kiro.InitiateLogin", initiate_data),
        ("Kiro.GetToken", token_data),
        ("WebPortalService.InitiateLogin", initiate_data),
        ("WebPortalService.GetToken", token_data),
        ("KiroWebPortal.InitiateLogin", initiate_data),
        ("KiroWebPortal.GetToken", token_data),
    ]
    
    successful_targets = []
    
    for target, data in targets_to_test:
        if test_service_target(target, data):
            successful_targets.append(target)
    
    print("\n" + "=" * 60)
    print("✅ Тестирование завершено")
    
    if successful_targets:
        print(f"\n🎉 Успешные targets: {successful_targets}")
    else:
        print("\n❌ Ни один target не сработал")
        print("\n💡 Возможные решения:")
        print("1. Использовать прямой Cognito с client_secret")
        print("2. Реализовать ручной ввод токенов")
        print("3. Найти правильный AWS подпись")

if __name__ == "__main__":
    main()