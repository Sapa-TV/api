# Api для сайта sapa-tv.ru

## Эндпоинты

- Swagger UI: http://localhost:3000/docs
- ReDoc: http://localhost:3000/redoc
- OpenAPI JSON: http://localhost:3000/openapi.json

## CI/CD Secrets

Для деплоя нужны следующие secrets в репозитории:

| Secret            | Описание                                   |
| ----------------- | ------------------------------------------ |
| `HOST`            | IP адрес VPS                               |
| `SSH_PORT`        | SSH порт (обычно 22)                       |
| `SSH_DEPLOY_KEY`  | Приватный SSH ключ для пользователя deploy |
| `APP_ID`          | ID GitHub App                              |
| `APP_PRIVATE_KEY` | Приватный ключ GitHub App (.pem)           |
