# 🔐 Инструкция по авторизации Google в drovity v0.1.2

## Как работает новый метод авторизации

Вместо устаревшего OOB flow используется **localhost redirect с ручным копированием кода**.

### Процесс авторизации:

```bash
drovity
# → 1. Accounts
# → 1. Add New Account
```

Вы увидите:

```
╔══════════════════════════════════════════════════════════════╗
║                 🔑 Google Authorization                      ║
╚══════════════════════════════════════════════════════════════╝

📱 Step 1: Open this URL:

   https://accounts.google.com/o/oauth2/v2/auth?client_id=...

🔐 Step 2: Authorize with your Google account

📋 Step 3: After authorization, Google will try to redirect to:

   http://localhost:8087/oauth/callback?code=...

   The page won't load (that's OK!), but the URL will contain a CODE.
   Look for ?code=

✂️  Step 4: Copy EVERYTHING after '?code=' from the URL

Paste the authorization code here: _
```

### Пошаговая инструкция:

1. **Скопируйте ссылку** из терминала
2. **Откройте её в браузере** (можно на любом устройстве - телефон, домашний ПК)
3. **Войдите в Google** и разрешите доступ
4. **Google попытается перенаправить** на `http://localhost:8087/...`
5. **Страница не загрузится** - это нормально!
6. **Посмотрите на URL** в адресной строке браузера
7. Найдите часть `?code=` и **скопируйте ВСЁ** после неё
8. **Вставьте код** в терминал
9. Готово! ✅

### Пример кода в URL:

```
http://localhost:8087/oauth/callback?code=4/0AanRRrvxxxxxxxxxxx-xxxxxxxxxxxxxxx&scope=...
                                          ↑
                                   Копируйте отсюда
```

Нужно скопировать: `4/0AanRRrvxxxxxxxxxxx-xxxxxxxxxxxxxxx`

### Почему этот метод?

- ✅ Работает с существующими credentials от Antigravity
- ✅ Не требует специальной настройки в Google Cloud Console  
- ✅ Работает на headless-серверах (браузер можно открыть на другом устройстве)
- ✅ Официально поддерживается Google

### После успешной авторизации:

```
✅ Authorization successful!

👤 Authorized as: your.email@gmail.com
📝 Name: Your Name

╔══════════════════════════════════════════════════════════════╗
║              ✅ Account Added Successfully!                  ║
╚══════════════════════════════════════════════════════════════╝
```

### Если возникла ошибка:

1. Убедитесь что скопировали **ВЕСЬ** код после `?code=` (без `&scope=...`)
2. Код должен начинаться с `4/0` обычно
3. Попробуйте ещё раз - сгенерируйте новую ссылку

---

## Обновление на сервере

```bash
# Обновить drovity до v0.1.2
curl -fsSL https://raw.githubusercontent.com/MixasV/drovity/main/install.sh | bash

# Добавить Google аккаунт (новый метод!)
drovity
# → 1. Accounts → 1. Add New Account
```

Релиз собирается: https://github.com/MixasV/drovity/actions

Через 5-10 минут будет готов!
