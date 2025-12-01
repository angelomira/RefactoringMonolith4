## Обзор безопасности

Данный документ описывает меры безопасности, примененные в проекте после полной миграции технологического стека.

### Текущий технологический стек

| Сервис | Язык | Фреймворк | БД драйвер |
|--------|------|-----------|------------|
| go_iss | Go 1.23 | Gin | pgx |
| node_web | Node.js | Express | pg |
| python_legacy | Python 3.11 | - | psycopg2 |
### Реализованные меры безопасности

#### 1. Защита секретов

- **Проблема**: Секреты и API ключи могут быть закоммичены в код
- **Решение**: Все секреты вынесены в переменные окружения
- **Файлы**: `.env`, `docker-compose.yml`

```yaml
# docker-compose.yml
environment:
  NASA_API_KEY: ${NASA_API_KEY:-}
  DATABASE_URL: ${DATABASE_URL:-postgres://...}
```

#### 2. SQL-инъекции

##### Go-сервис (pgx)

- **Проблема**: Прямые SQL-запросы уязвимы к инъекциям
- **Решение**: Использование параметризованных запросов через pgx
- **Файлы**: `services/go-iss/internal/repo/repo.go`

```go
// Параметризованный запрос в Go
_, err := pool.Exec(ctx, 
    "INSERT INTO iss_fetch_log (source_url, payload) VALUES ($1, $2)",
    sourceURL, payload)
```

##### Node.js сервис (pg)

- **Проблема**: Конкатенация строк в SQL
- **Решение**: Использование параметризованных запросов через pg
- **Файлы**: `services/node-web/server.js`

```javascript
// Параметризованный запрос в Node.js
const result = await pool.query(
    'SELECT * FROM cms_pages WHERE slug = $1',
    [slug]
);
```

##### Python-сервис (psycopg2)

- **Решение**: Использование плейсхолдеров %s
- **Файлы**: `services/pascal-legacy/legacy.py`

```python
# Параметризованный запрос в Python
cursor.execute(
    "INSERT INTO telemetry_legacy (recorded_at, voltage, temp, source_file) VALUES (%s, %s, %s, %s)",
    (recorded_at, voltage, temp, source_file)
)
```

#### 3. Защита от XSS

- **Проблема**: Небезопасный вывод пользовательских данных
- **Решение**: EJS автоматически экранирует вывод через `<%= %>`
- **Файлы**: `services/node-web/views/*.ejs`

**Рекомендация**: Использовать `<%= var %>` (экранированный) вместо `<%- var %>` (raw) в EJS-шаблонах.

#### 4. Таймауты внешних API

- **Проблема**: Запросы к внешним API могут зависнуть
- **Решение**: Установлены таймауты 30 секунд для всех HTTP-клиентов
- **Файлы**: `services/go-iss/internal/clients/clients.go`

```go
// Go HTTP клиент с таймаутом
client := &http.Client{
    Timeout: 30 * time.Second,
}
```

##### 5. User-Agent для предотвращения блокировок

- **Проблема**: Некоторые API блокируют запросы без User-Agent
- **Решение**: Установлен идентифицирующий User-Agent
- **Файлы**: `services/go-iss/internal/clients/clients.go`

```go
req.Header.Set("User-Agent", "go-iss-service/1.0")
```

##### 6. Предотвращение N+1 проблемы

- **Проблема**: Множественные запросы к БД в цикле
- **Решение**: Использование bulk операций и upsert
- **Файлы**: `services/go-iss/internal/repo/repo.go`
##### 7. Валидация входных данных

- **Проблема**: Невалидированные входные данные
- **Решение**: Типобезопасность Go, проверки на nil/empty
- **Файлы**:` services/go-iss/internal/handlers/handlers.go`

**Рекомендация**: Добавить дополнительную валидацию для user input (Query параметров).

##### 8. Защита от дубликатов (идемпотентность)

- **Проблема**: Повторная синхронизация создает дубликаты
- **Решение**: Upsert по бизнес-ключам (`dataset_id`)
- **Файлы**: `services/go-iss/internal/repo/repo.go`

```go
// Go - Upsert запрос
_, err := pool.Exec(ctx, `
    INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw)
    VALUES ($1, $2, $3, $4, $5)
    ON CONFLICT (dataset_id) DO UPDATE SET
        title = EXCLUDED.title,
        status = EXCLUDED.status,
        updated_at = EXCLUDED.updated_at,
        raw = EXCLUDED.raw
`, datasetID, title, status, updatedAt, raw)
```

##### 9. Логирование без sensitive данных

- **Проблема**: Логи могут содержать секреты
- **Решение**: Логируются только безопасные данные, пароли не выводятся
- **Файлы**: `services/go-iss/cmd/main.go`, `services/pascal-legacy/legacy.py`
### Известные уязвимости для демонстрации

> **Эти уязвимости оставлены намеренно для учебных целей:**
#### 1. XSS Demo Content
- **Файл**: `db/init.sql`
- **Описание**: Таблица cms_pages содержит `<script>` тег для демонстрации XSS
- **Риск**: Низкий (только для демонстрации)
- **Рекомендация**: В production удалить эти записи

```sql
INSERT INTO cms_pages(slug, title, body)
VALUES ('unsafe', 'Небезопасный пример', '<script>console.log("XSS training")</script>...')
```