# AGENTS.md

## Правила

1. **Зависимости** - добавлять через `cargo add`, НЕ редактировать Cargo.toml вручную
2. **После изменений** - запускать `cargo check` и `cargo fmt`
3. **Генерация OpenAPI** - `just gen` или `cargo test generate_openapi`

### Критические правила

- **НЕ делать коммиты** - я НИКОГДА не делаю коммиты, даже если пользователь разрешит
- **НЕ выполнять git push** - я НИКОГДА не делаю git push
- **НЕ редактировать историю git** - я НЕ делаю reset, rebase, amend и т.д.
- **НЕ делать cog bump** - я НЕ делаю bump версии, даже если пользователь разрешит

## Команды

```bash
just gen     # сгенерировать openapi.json
cargo fmt   # форматировать код
cargo check # проверить компиляцию
```

## Эндпоинты

- Swagger UI: http://localhost:3000/docs
- ReDoc: http://localhost:3000/redoc
- OpenAPI JSON: http://localhost:3000/openapi.json