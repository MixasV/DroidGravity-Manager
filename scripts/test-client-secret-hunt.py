#!/usr/bin/env python3
"""
Поиск client_secret для Kiro AWS Cognito клиента
"""

import requests
import json
from urllib.parse import urlencode
import hashlib
import base64

# Данные из вашей авторизации
AUTH_CODE = "c8fa952a-a0e1-417d-b838-de4ab4222db7"
CODE_VERIFIER = "sA_y-iGV0_37Ppav7vbaNSuTvdbdz7BoRrxgJROGtpwsA_y-iGV0_37Ppav7vbaNSuTvdbdz7BoRrxgJROGtpwsA_y-iGV0_37Ppav7vbaNSuTvdbdz7BoRrxgJROGtpw"
REDIRECT_URI = "http://localhost:3128"

# AWS Cognito данные
COGNITO_URL = "https://kiro-prod-us-east-1.auth.us-east-1.amazoncognito.com"
CLIENT_ID = "59bd15eh40ee7pc20h0bkcu7id"

def test_common_client_secrets():
    """Test common/predictable client secrets"""
    print("=== TESTING COMMON CLIENT SECRETS ===")
    
    # Common patterns for client secrets
    common_secrets = [
        "",  # Empty secret
        CLIENT_ID,  # Same as client_id
        "kiro",  # App name
        "kiro-secret",  # App name + secret
        "secret",  # Just "secret"
        "client_secret",  # Literal
        "59bd15eh40ee7pc20h0bkcu7id-secret",  # Client ID + secret
        hashlib.sha256(CLIENT_ID.encode()).hexdigest(),  # SHA256 of client_id
        base64.b64encode(CLIENT_ID.encode()).decode(),  # Base64 of client_id
        "kiro-prod-secret",  # Environment based
        "aws-cognito-secret",  # Service based
        "default",  # Default value
        "password",  # Common default
        "123456",  # Weak secret
        CLIENT_ID[::-1],  # Reversed client_id
    ]
    
    for i, secret in enumerate(common_secrets, 1):
        print(f"\n--- Attempt {i}: Testing secret '{secret}' ---")
        
        data = urlencode({
            "grant_type": "authorization_code",
            "client_id": CLIENT_ID,
            "client_secret": secret,
            "code": AUTH_CODE,
            "code_verifier": CODE_VERIFIER,
            "redirect_uri": REDIRECT_URI
        })
        
        try:
            response = requests.post(
                f"{COGNITO_URL}/oauth2/token",
                headers={"Content-Type": "application/x-www-form-urlencoded"},
                data=data,
                timeout=10
            )
            
            print(f"Status: {response.status_code}")
            print(f"Response: {response.text}")
            
            if response.status_code == 200:
                try:
                    tokens = response.json()
                    if 'access_token' in tokens:
                        print(f"🎉 SUCCESS! Found working client_secret: '{secret}'")
                        print(f"Access Token: {tokens['access_token'][:50]}...")
                        return secret, tokens
                except:
                    pass
            elif response.status_code == 400:
                error_data = response.json()
                error_type = error_data.get('error', 'unknown')
                if error_type != "invalid_client":
                    print(f"⚠️  Different error: {error_type} - might be progress!")
                    
        except Exception as e:
            print(f"❌ ERROR: {e}")
    
    print("\n❌ No common client secrets worked")
    return None, None

def test_derived_secrets():
    """Test secrets derived from known values"""
    print("\n=== TESTING DERIVED SECRETS ===")
    
    # Values we know about Kiro
    known_values = [
        "kiro-prod-us-east-1",  # Environment
        "59bd15eh40ee7pc20h0bkcu7id",  # Client ID
        "KiroWebPortalService",  # Service name
        "699475941385",  # AWS Account ID from profileArn
        "EHGA3GRVQMUK",  # Profile ID from profileArn
    ]
    
    derived_secrets = []
    
    # Generate derived secrets
    for value in known_values:
        derived_secrets.extend([
            hashlib.md5(value.encode()).hexdigest(),
            hashlib.sha1(value.encode()).hexdigest(),
            hashlib.sha256(value.encode()).hexdigest(),
            base64.b64encode(value.encode()).decode(),
            f"{value}-secret",
            f"secret-{value}",
            value.upper(),
            value.lower(),
        ])
    
    # Test combinations
    for i, secret in enumerate(derived_secrets[:20], 1):  # Limit to first 20
        print(f"\n--- Derived {i}: Testing '{secret[:50]}...' ---")
        
        data = urlencode({
            "grant_type": "authorization_code",
            "client_id": CLIENT_ID,
            "client_secret": secret,
            "code": AUTH_CODE,
            "code_verifier": CODE_VERIFIER,
            "redirect_uri": REDIRECT_URI
        })
        
        try:
            response = requests.post(
                f"{COGNITO_URL}/oauth2/token",
                headers={"Content-Type": "application/x-www-form-urlencoded"},
                data=data,
                timeout=10
            )
            
            if response.status_code == 200:
                try:
                    tokens = response.json()
                    if 'access_token' in tokens:
                        print(f"🎉 SUCCESS! Found working derived secret: '{secret}'")
                        return secret, tokens
                except:
                    pass
            elif response.status_code == 400:
                error_data = response.json()
                error_type = error_data.get('error', 'unknown')
                if error_type != "invalid_client":
                    print(f"⚠️  Different error: {error_type}")
                    
        except Exception as e:
            print(f"❌ ERROR: {e}")
    
    print("\n❌ No derived secrets worked")
    return None, None

def test_alternative_approaches():
    """Test alternative authentication approaches"""
    print("\n=== TESTING ALTERNATIVE APPROACHES ===")
    
    approaches = [
        {
            "name": "No client_secret (public client)",
            "data": {
                "grant_type": "authorization_code",
                "client_id": CLIENT_ID,
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            }
        },
        {
            "name": "Client credentials in Authorization header",
            "data": {
                "grant_type": "authorization_code",
                "code": AUTH_CODE,
                "code_verifier": CODE_VERIFIER,
                "redirect_uri": REDIRECT_URI
            },
            "headers": {
                "Authorization": f"Basic {base64.b64encode(f'{CLIENT_ID}:'.encode()).decode()}"
            }
        },
        {
            "name": "Different grant type",
            "data": {
                "grant_type": "client_credentials",
                "client_id": CLIENT_ID,
                "scope": "email openid"
            }
        }
    ]
    
    for approach in approaches:
        print(f"\n--- {approach['name']} ---")
        
        headers = approach.get('headers', {})
        headers["Content-Type"] = "application/x-www-form-urlencoded"
        
        try:
            response = requests.post(
                f"{COGNITO_URL}/oauth2/token",
                headers=headers,
                data=urlencode(approach['data']),
                timeout=10
            )
            
            print(f"Status: {response.status_code}")
            print(f"Response: {response.text}")
            
            if response.status_code == 200:
                try:
                    tokens = response.json()
                    if 'access_token' in tokens:
                        print(f"🎉 SUCCESS! Alternative approach worked!")
                        return approach['name'], tokens
                except:
                    pass
                    
        except Exception as e:
            print(f"❌ ERROR: {e}")
    
    print("\n❌ No alternative approaches worked")
    return None, None

def main():
    print("KIRO CLIENT SECRET HUNTER")
    print("=" * 50)
    print(f"Hunting for client_secret for client_id: {CLIENT_ID}")
    print(f"Using auth code: {AUTH_CODE}")
    print()
    
    # Try common secrets
    secret, tokens = test_common_client_secrets()
    if secret:
        print(f"\n🎉 FOUND WORKING CLIENT SECRET: '{secret}'")
        return
    
    # Try derived secrets
    secret, tokens = test_derived_secrets()
    if secret:
        print(f"\n🎉 FOUND WORKING DERIVED SECRET: '{secret}'")
        return
    
    # Try alternative approaches
    approach, tokens = test_alternative_approaches()
    if approach:
        print(f"\n🎉 FOUND WORKING APPROACH: {approach}")
        return
    
    print("\n❌ CLIENT SECRET HUNT FAILED")
    print("\nNext steps:")
    print("1. Reverse engineer Kiro desktop app")
    print("2. Check if Kiro uses different OAuth flow")
    print("3. Look for client_secret in Kiro installation files")
    print("4. Try intercepting HTTPS traffic from Kiro app")
    print("5. Check if Kiro bypasses Cognito entirely")

if __name__ == "__main__":
    main()