#!/usr/bin/env python3
"""
Тестирование Kiro API с различными заголовками и форматами
"""

import requests
import json
import base64
import hashlib
import hmac
import time
from urllib.parse import urlencode

def test_with_aws_headers():
    """Тест с AWS подписанными заголовками"""
    print("=== TESTING WITH AWS HEADERS ===")
    
    url = "https://app.kiro.dev/api/v1/InitiateLogin"
    
    # AWS подпись может потребоваться
    headers = {
        "Content-Type": "application/x-amz-json-1.1",
        "X-Amz-Target": "KiroWebPortalService.InitiateLogin",
        "User-Agent": "aws-sdk-js/2.1691.0 linux/v18.20.4 promise",
        "Authorization": "AWS4-HMAC-SHA256 Credential=test/20260219/us-east-1/execute-api/aws4_request, SignedHeaders=host;x-amz-date, Signature=test"
    }
    
    data = {
        "idp": "Google",
        "redirectUri": "http://localhost:3128/oauth/callback?login_option=google",
        "state": "test-state-12345",
        "codeChallenge": "test-code-challenge-12345",
        "codeChallengeMethod": "S256"
    }
    
    try:
        response = requests.post(url, json=data, headers=headers, timeout=10)
        print(f"Status: {response.status_code}")
        print(f"Response: {response.text[:300]}")
    except Exception as e:
        print(f"Error: {e}")

def test_with_kiro_headers():
    """Тест с заголовками как в Kiro IDE"""
    print("\n=== TESTING WITH KIRO IDE HEADERS ===")
    
    url = "https://app.kiro.dev/api/v1/InitiateLogin"
    
    headers = {
        "Content-Type": "application/x-amz-json-1.1",
        "X-Amz-Target": "KiroWebPortalService.InitiateLogin",
        "User-Agent": "Kiro/1.0.0",
        "Accept": "application/json",
        "Origin": "https://app.kiro.dev",
        "Referer": "https://app.kiro.dev/"
    }
    
    data = {
        "idp": "Google",
        "redirectUri": "http://localhost:3128/oauth/callback?login_option=google",
        "state": "test-state-12345",
        "codeChallenge": "test-code-challenge-12345",
        "codeChallengeMethod": "S256"
    }
    
    try:
        response = requests.post(url, json=data, headers=headers, timeout=10)
        print(f"Status: {response.status_code}")
        print(f"Response: {response.text[:300]}")
    except Exception as e:
        print(f"Error: {e}")

def test_direct_cognito():
    """Тест прямого обращения к Cognito"""
    print("\n=== TESTING DIRECT COGNITO ===")
    
    # Попробуем прямой вызов Cognito как в браузере
    url = "https://kiro-prod-us-east-1.auth.us-east-1.amazoncognito.com/oauth2/token"
    
    data = {
        "grant_type": "authorization_code",
        "client_id": "59bd15eh40ee7pc20h0bkcu7id",
        "code": "test-code-12345",
        "code_verifier": "test-verifier-12345",
        "redirect_uri": "http://localhost:3128/oauth/callback?login_option=google"
    }
    
    headers = {
        "Content-Type": "application/x-www-form-urlencoded",
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
    }
    
    try:
        response = requests.post(url, data=urlencode(data), headers=headers, timeout=10)
        print(f"Status: {response.status_code}")
        print(f"Response: {response.text[:300]}")
    except Exception as e:
        print(f"Error: {e}")

def test_binary_format():
    """Тест с бинарным форматом как в перехваченном трафике"""
    print("\n=== TESTING BINARY FORMAT ===")
    
    url = "https://app.kiro.dev/api/v1/InitiateLogin"
    
    # Попробуем формат как в перехваченных файлах
    headers = {
        "Content-Type": "application/x-amz-cbor-1.1",
        "X-Amz-Target": "KiroWebPortalService.InitiateLogin",
        "User-Agent": "aws-sdk-js/2.1691.0"
    }
    
    # Простые данные в JSON для начала
    data = {
        "idp": "Google",
        "redirectUri": "http://localhost:3128/oauth/callback?login_option=google",
        "state": "test-state-12345",
        "codeChallenge": "test-code-challenge-12345",
        "codeChallengeMethod": "S256"
    }
    
    try:
        response = requests.post(url, json=data, headers=headers, timeout=10)
        print(f"Status: {response.status_code}")
        print(f"Response: {response.text[:300]}")
    except Exception as e:
        print(f"Error: {e}")

def main():
    print("🔍 Тестирование Kiro API с различными заголовками")
    print("=" * 60)
    
    test_with_aws_headers()
    test_with_kiro_headers()
    test_direct_cognito()
    test_binary_format()
    
    print("\n" + "=" * 60)
    print("✅ Тестирование завершено")
    print("\n💡 Возможные причины UnknownOperationException:")
    print("1. Нужна AWS подпись запроса")
    print("2. Неправильный формат данных (CBOR вместо JSON)")
    print("3. Нужны специальные заголовки аутентификации")
    print("4. API доступен только из Kiro IDE")

if __name__ == "__main__":
    main()