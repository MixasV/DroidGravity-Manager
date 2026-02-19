#!/usr/bin/env python3
"""
Финальный тест полного Kiro OAuth flow
"""

import base64
import hashlib
import secrets
import urllib.parse
import webbrowser
import http.server
import socketserver
import threading
import time
from urllib.parse import urlparse, parse_qs

def generate_pkce():
    """Генерация PKCE параметров"""
    code_verifier = base64.urlsafe_b64encode(secrets.token_bytes(32)).decode('utf-8').rstrip('=')
    code_challenge = base64.urlsafe_b64encode(
        hashlib.sha256(code_verifier.encode('utf-8')).digest()
    ).decode('utf-8').rstrip('=')
    return code_verifier, code_challenge

class CallbackHandler(http.server.BaseHTTPRequestHandler):
    """Обработчик OAuth callback"""
    
    def do_GET(self):
        """Обработка GET запроса"""
        print(f"\n🔔 CALLBACK RECEIVED: {self.path}")
        
        # Парсим URL
        parsed_url = urlparse(self.path)
        query_params = parse_qs(parsed_url.query)
        
        print(f"📋 Parsed parameters:")
        for key, value in query_params.items():
            print(f"  {key}: {value[0] if value else 'None'}")
        
        # Проверяем наличие кода
        if 'code' in query_params:
            code = query_params['code'][0]
            state = query_params.get('state', [''])[0]
            
            print(f"\n🎉 SUCCESS! Authorization code received:")
            print(f"Code: {code}")
            print(f"State: {state}")
            
            # Сохраняем код в глобальную переменную
            global received_code, received_state
            received_code = code
            received_state = state
            
            # Отправляем успешный ответ
            self.send_response(200)
            self.send_header('Content-type', 'text/html')
            self.end_headers()
            
            success_html = """
            <!DOCTYPE html>
            <html>
            <head><title>Kiro Authorization Success</title></head>
            <body>
                <h1>✅ Authorization Successful!</h1>
                <p>You can close this window and return to DroidGravity Manager.</p>
                <p>Authorization code: <code>{}</code></p>
            </body>
            </html>
            """.format(code[:20] + "...")
            
            self.wfile.write(success_html.encode())
            
        elif 'error' in query_params:
            error = query_params['error'][0]
            error_description = query_params.get('error_description', [''])[0]
            
            print(f"\n❌ ERROR in callback:")
            print(f"Error: {error}")
            print(f"Description: {error_description}")
            
            # Отправляем ошибку
            self.send_response(400)
            self.send_header('Content-type', 'text/html')
            self.end_headers()
            
            error_html = f"""
            <!DOCTYPE html>
            <html>
            <head><title>Kiro Authorization Error</title></head>
            <body>
                <h1>❌ Authorization Error</h1>
                <p>Error: {error}</p>
                <p>Description: {error_description}</p>
            </body>
            </html>
            """
            
            self.wfile.write(error_html.encode())
        else:
            # Обычный запрос без параметров
            self.send_response(200)
            self.send_header('Content-type', 'text/plain')
            self.end_headers()
            self.wfile.write(b"OAuth callback server running...")
    
    def log_message(self, format, *args):
        """Отключаем стандартные логи сервера"""
        pass

def start_callback_server(port=3128):
    """Запуск локального сервера для callback"""
    global httpd
    
    try:
        httpd = socketserver.TCPServer(("", port), CallbackHandler)
        print(f"🌐 Callback server started on http://localhost:{port}")
        httpd.serve_forever()
    except OSError as e:
        if e.errno == 10048:  # Port already in use
            print(f"❌ Port {port} is already in use!")
            print("Please close any other applications using this port and try again.")
        else:
            print(f"❌ Failed to start server: {e}")

def test_complete_oauth_flow():
    """Тест полного OAuth flow"""
    print("🔍 ПОЛНЫЙ KIRO OAUTH FLOW TEST")
    print("=" * 50)
    
    # Генерируем параметры
    code_verifier, code_challenge = generate_pkce()
    state = secrets.token_urlsafe(32)
    
    # Создаем ссылку как в нашем коде
    signin_url = f"https://app.kiro.dev/signin?state={state}&code_challenge={code_challenge}&code_challenge_method=S256&redirect_uri=http%3A//localhost%3A3128&redirect_from=KiroIDE"
    
    print(f"📋 Generated parameters:")
    print(f"State: {state}")
    print(f"Code Verifier: {code_verifier}")
    print(f"Code Challenge: {code_challenge}")
    print()
    
    print(f"🔗 Authorization URL:")
    print(signin_url)
    print()
    
    # Запускаем callback сервер в отдельном потоке
    global received_code, received_state, httpd
    received_code = None
    received_state = None
    httpd = None
    
    server_thread = threading.Thread(target=start_callback_server, daemon=True)
    server_thread.start()
    
    # Даем серверу время запуститься
    time.sleep(1)
    
    print("📋 ИНСТРУКЦИИ:")
    print("1. Скопируйте ссылку выше")
    print("2. Откройте её в браузере")
    print("3. Выберите Google как метод авторизации")
    print("4. Завершите авторизацию")
    print("5. Вернитесь сюда и посмотрите результат")
    print()
    
    # Автоматически открываем браузер
    try:
        webbrowser.open(signin_url)
        print("🌐 Browser opened automatically")
    except:
        print("❌ Could not open browser automatically")
    
    print("\n⏳ Waiting for callback... (Press Ctrl+C to stop)")
    
    # Ждем callback
    try:
        start_time = time.time()
        while received_code is None and time.time() - start_time < 300:  # 5 минут
            time.sleep(1)
        
        if received_code:
            print(f"\n🎉 SUCCESS! Received authorization code:")
            print(f"Code: {received_code}")
            print(f"State: {received_state}")
            print(f"Expected State: {state}")
            print(f"State Match: {'✅' if received_state == state else '❌'}")
            print()
            
            print(f"📝 Next steps for implementation:")
            print(f"1. Use code '{received_code}' with verifier '{code_verifier}'")
            print(f"2. Exchange for tokens using proper endpoint")
            print(f"3. Save tokens to account storage")
            
            return received_code, code_verifier, state
        else:
            print("\n⏰ Timeout waiting for callback")
            print("💡 Make sure you completed authorization in browser")
            
    except KeyboardInterrupt:
        print("\n🛑 Stopped by user")
    
    finally:
        if httpd:
            httpd.shutdown()
            print("🔌 Callback server stopped")
    
    return None, code_verifier, state

def main():
    print("🚀 KIRO OAUTH FLOW TESTER")
    print("=" * 30)
    print()
    print("This test will:")
    print("1. Generate proper OAuth parameters")
    print("2. Create authorization URL like KiroIDE")
    print("3. Start local callback server")
    print("4. Open browser for authorization")
    print("5. Capture authorization code")
    print()
    
    input("Press Enter to start...")
    
    code, verifier, state = test_complete_oauth_flow()
    
    if code:
        print(f"\n✅ TEST SUCCESSFUL!")
        print(f"Authorization code: {code}")
        print(f"Code verifier: {verifier}")
        print(f"State: {state}")
        print()
        print("🔧 Implementation ready:")
        print("- OAuth URL generation works")
        print("- Callback handling works") 
        print("- PKCE parameters correct")
        print("- Ready for token exchange")
    else:
        print(f"\n❌ TEST INCOMPLETE")
        print("Authorization code not received")
        print("Check browser and try again")

if __name__ == "__main__":
    main()