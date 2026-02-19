#!/usr/bin/env python3
"""
Упрощенный тестовый скрипт для проверки Kiro GetToken API
Тестирует разные форматы запросов без реальной авторизации
"""

import requests
import json
import hashlib
import base64
import secrets
import string

# Kiro API Configuration
KIRO_API_URL = "https://app.kiro.dev"

def generate_pkce():
    """Generate PKCE code_verifier and code_challenge"""
    # Generate code_verifier (128 random characters)
    alphabet = string.ascii_letters + string.digits
    verifier = ''.join(secrets.choice(alphabet) for _ in range(128))
    
    # Generate code_challenge = BASE64URL(SHA256(verifier))
    challenge = base64.urlsafe_b64encode(
        hashlib.sha256(verifier.encode()).digest()
    ).decode().rstrip('=')
    
    return verifier, challenge

def test_get_token_formats():
    """Test GetToken API with different formats using fake data"""
    print("=== TESTING GETTOKEN API FORMATS ===")
    
    # Generate fake but valid-looking data
    fake_code = "test-auth-code-12345"
    code_verifier, _ = generate_pkce()
    redirect_uri = "http://localhost:3128"
    
    print(f"Fake Code: {fake_code}")
    print(f"Code Verifier: {code_verifier[:20]}...")
    print(f"Redirect URI: {redirect_uri}")
    
    # Test different request formats
    test_cases = [
        {
            "name": "Format 1: snake_case с X-Amz-Target",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/operation",
            "headers": {
                "Content-Type": "application/json",
                "X-Amz-Target": "KiroWebPortalService.GetToken"
            },
            "data": {
                "code": fake_code,
                "code_verifier": code_verifier,
                "redirect_uri": redirect_uri
            }
        },
        {
            "name": "Format 2: AWS JSON RPC 1.1 с snake_case",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/operation",
            "headers": {
                "Content-Type": "application/x-amz-json-1.1",
                "X-Amz-Target": "KiroWebPortalService.GetToken"
            },
            "data": {
                "code": fake_code,
                "code_verifier": code_verifier,
                "redirect_uri": redirect_uri
            }
        },
        {
            "name": "Format 3: Direct GetToken endpoint с snake_case",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/GetToken",
            "headers": {"Content-Type": "application/json"},
            "data": {
                "code": fake_code,
                "code_verifier": code_verifier,
                "redirect_uri": redirect_uri
            }
        },
        {
            "name": "Format 4: camelCase fields",
            "url": f"{KIRO_API_URL}/service/KiroWebPortalService/GetToken",
            "headers": {"Content-Type": "application/json"},
            "data": {
                "code": fake_code,
                "codeVerifier": code_verifier,
                "redirectUri": redirect_uri
            }
        },
        {
            "name": "Format 5: Alternative endpoint",
            "url": f"{KIRO_API_URL}/api/auth/token",
            "headers": {"Content-Type": "application/json"},
            "data": {
                "code": fake_code,
                "code_verifier": code_verifier,
                "redirect_uri": redirect_uri
            }
        },
        {
            "name": "Format 6: OAuth2 standard format",
            "url": f"{KIRO_API_URL}/oauth2/token",
            "headers": {"Content-Type": "application/x-www-form-urlencoded"},
            "data": f"grant_type=authorization_code&code={fake_code}&code_verifier={code_verifier}&redirect_uri={redirect_uri}"
        }
    ]
    
    results = []
    
    for i, test_case in enumerate(test_cases, 1):
        print(f"\n--- {test_case['name']} ---")
        print(f"URL: {test_case['url']}")
        print(f"Headers: {test_case['headers']}")
        
        if isinstance(test_case['data'], dict):
            print(f"Data: {json.dumps(test_case['data'], indent=2)}")
        else:
            print(f"Data: {test_case['data']}")
        
        try:
            if test_case['name'] == "Format 6: OAuth2 standard format":
                # Form data request
                response = requests.post(
                    test_case['url'],
                    headers=test_case['headers'],
                    data=test_case['data'],
                    timeout=10
                )
            else:
                # JSON request
                response = requests.post(
                    test_case['url'],
                    headers=test_case['headers'],
                    json=test_case['data'],
                    timeout=10
                )
            
            print(f"Status: {response.status_code}")
            print(f"Response Headers: {dict(response.headers)}")
            print(f"Response Body: {response.text}")
            
            # Analyze response
            result = {
                "format": test_case['name'],
                "status": response.status_code,
                "success": False,
                "error_type": "unknown"
            }
            
            if response.status_code == 200:
                result["success"] = True
                result["error_type"] = "none"
            elif response.status_code == 400:
                result["error_type"] = "bad_request"
            elif response.status_code == 401:
                result["error_type"] = "unauthorized"
            elif response.status_code == 404:
                result["error_type"] = "not_found"
            elif "UnknownOperationException" in response.text:
                result["error_type"] = "unknown_operation"
            
            results.append(result)
                    
        except Exception as e:
            print(f"ERROR: {e}")
            results.append({
                "format": test_case['name'],
                "status": "error",
                "success": False,
                "error_type": "network_error",
                "error": str(e)
            })
    
    # Summary
    print("\n" + "=" * 60)
    print("SUMMARY OF RESULTS")
    print("=" * 60)
    
    for result in results:
        status_icon = "✓" if result["success"] else "✗"
        print(f"{status_icon} {result['format']}")
        print(f"   Status: {result['status']}, Error Type: {result['error_type']}")
        if 'error' in result:
            print(f"   Error: {result['error']}")
    
    # Recommendations
    print("\n" + "=" * 60)
    print("RECOMMENDATIONS")
    print("=" * 60)
    
    successful_formats = [r for r in results if r["success"]]
    if successful_formats:
        print("✓ Working formats found:")
        for result in successful_formats:
            print(f"  - {result['format']}")
    else:
        print("✗ No working formats found")
        
        # Analyze error patterns
        error_types = {}
        for result in results:
            error_type = result["error_type"]
            if error_type not in error_types:
                error_types[error_type] = []
            error_types[error_type].append(result["format"])
        
        print("\nError patterns:")
        for error_type, formats in error_types.items():
            print(f"  {error_type}: {len(formats)} formats")
            for fmt in formats:
                print(f"    - {fmt}")
        
        if "unknown_operation" in error_types:
            print("\n💡 Most endpoints return 'UnknownOperationException'")
            print("   This suggests the API has changed or requires authentication")
        
        if "not_found" in error_types:
            print("\n💡 Some endpoints return 404 Not Found")
            print("   These endpoints may not exist or have moved")
        
        if "bad_request" in error_types:
            print("\n💡 Some endpoints return 400 Bad Request")
            print("   These endpoints exist but expect different parameters")

def main():
    print("KIRO GETTOKEN API FORMAT TESTER")
    print("=" * 50)
    print("Testing different GetToken API formats with fake data")
    print("This helps identify the correct endpoint and parameter format")
    print()
    
    test_get_token_formats()
    
    print("\nTest completed!")
    print("\nNext steps:")
    print("1. If any format returns 400/401 instead of UnknownOperationException,")
    print("   that's likely the correct endpoint")
    print("2. Use the working format in the real OAuth implementation")
    print("3. Test with real authorization code from browser")

if __name__ == "__main__":
    main()