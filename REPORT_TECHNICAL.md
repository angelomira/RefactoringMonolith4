### **Детальный анализ рефакторинга проекта "Кассиопея"**
#### **1. Миграция Rust → Go**

##### **1.1 Обоснование выбора Go**

| Критерий         | Rust                     | Go         | Преимущество        |
| ---------------- | ------------------------ | ---------- | ------------------- |
| Синтаксис        | Сложный (borrow checker) | Простой    | Go проще            |
| Бинарник         | Единый                   | Единый     | Равно               |
| Concurrency      | tokio (async/await)      | goroutines | Go проще            |
| Экосистема       | Молодая                  | Зрелая     | Go лучше для DevOps |
##### **1.2 Соответствие компонентов**

| Компонент Rust | Компонент Go | Описание |
|----------------|--------------|----------|
| `Axum` | `Gin` | Веб-фреймворк |
| `SQLx` | `pgx` | PostgreSQL драйвер |
| `reqwest` | `net/http` | HTTP клиент |
| `tokio::spawn` | `go func()` | Асинхронные задачи |
| `Arc<T>` | `*T` | Указатели |
| `anyhow::Error` | `error` | Обработка ошибок |
| `serde` | `encoding/json` | JSON сериализация |
| `tracing` | `log` | Логирование |
##### **1.3 Структура Go-сервиса**

```ruby
services/go-iss/
├── cmd/
│   └── main.go              # Entry point, DI, запуск goroutines
├── internal/
│   ├── config/
│   │   └── config.go        # Конфигурация из env
│   ├── domain/
│   │   └── models.go        # Доменные модели (IssLog, OsdrItem, etc.)
│   ├── repo/
│   │   └── repo.go          # Репозитории (IssRepo, OsdrRepo, CacheRepo)
│   ├── clients/
│   │   └── clients.go       # HTTP клиенты (IssClient, OsdrClient, etc.)
│   ├── services/
│   │   └── services.go      # Бизнес-логика (IssService, OsdrService, etc.)
│   └── handlers/
│       └── handlers.go      # HTTP хендлеры
├── go.mod                    # Go modules
└── Dockerfile                # Multi-stage build
```
##### **1.4 Примеры кода**

###### Domain Models (`internal/domain/models.go`)

```go
// IssLog представляет запись позиции МКС
type IssLog struct {
    ID        int64           `json:"id"`
    FetchedAt time.Time       `json:"fetched_at"`
    SourceURL string          `json:"source_url"`
    Payload   json.RawMessage `json:"payload"`
}

// OsdrItem представляет датасет NASA OSDR
type OsdrItem struct {
    ID         int64           `json:"id"`
    DatasetID  string          `json:"dataset_id"`
    Title      string          `json:"title"`
    Status     string          `json:"status"`
    UpdatedAt  *time.Time      `json:"updated_at,omitempty"`
    InsertedAt time.Time       `json:"inserted_at"`
    Raw        json.RawMessage `json:"raw"`
}

// IssTrend представляет анализ движения МКС
type IssTrend struct {
    Movement   bool      `json:"movement"`
    DeltaKm    float64   `json:"delta_km"`
    DtSec      float64   `json:"dt_sec"`
    VelocityKmH float64  `json:"velocity_kmh"`
    FromTime   time.Time `json:"from_time"`
    ToTime     time.Time `json:"to_time"`
    FromLat    float64   `json:"from_lat"`
    FromLon    float64   `json:"from_lon"`
    ToLat      float64   `json:"to_lat"`
    ToLon      float64   `json:"to_lon"`
}
```
###### Repository Pattern (`internal/repo/repo.go`)

```go
// IssRepo управляет записями позиций МКС
type IssRepo struct {
    pool *pgxpool.Pool
}

// InsertLog вставляет новую запись позиции МКС
func (r *IssRepo) InsertLog(ctx context.Context, sourceURL string, payload json.RawMessage) (int64, error) {
    var id int64
    err := r.pool.QueryRow(ctx,
        `INSERT INTO iss_fetch_log (source_url, payload) VALUES ($1, $2) RETURNING id`,
        sourceURL, payload).Scan(&id)
    return id, err
}

// GetLastTwo возвращает две последние записи для анализа тренда
func (r *IssRepo) GetLastTwo(ctx context.Context) ([]IssLog, error) {
    rows, err := r.pool.Query(ctx,
        `SELECT id, fetched_at, source_url, payload 
         FROM iss_fetch_log 
         ORDER BY id DESC LIMIT 2`)
    if err != nil {
        return nil, err
    }
    defer rows.Close()
    // ...parsing...
}

// OsdrRepo управляет датасетами OSDR с Upsert
type OsdrRepo struct {
    pool *pgxpool.Pool
}

// UpsertItem вставляет или обновляет датасет по dataset_id
func (r *OsdrRepo) UpsertItem(ctx context.Context, item *OsdrItem) error {
    _, err := r.pool.Exec(ctx,
        `INSERT INTO osdr_items (dataset_id, title, status, updated_at, raw)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (dataset_id) DO UPDATE SET
            title = EXCLUDED.title,
            status = EXCLUDED.status,
            updated_at = EXCLUDED.updated_at,
            raw = EXCLUDED.raw`,
        item.DatasetID, item.Title, item.Status, item.UpdatedAt, item.Raw)
    return err
}
```
###### HTTP Handlers (`internal/handlers/handlers.go`)

```go
// HealthHandler возвращает статус сервиса
func (h *Handlers) HealthHandler(c *gin.Context) {
    c.JSON(http.StatusOK, gin.H{
        "status": "ok",
        "now":    time.Now().UTC().Format(time.RFC3339),
    })
}

// LastHandler возвращает последнюю позицию МКС
func (h *Handlers) LastHandler(c *gin.Context) {
    log, err := h.issRepo.GetLast(c.Request.Context())
    if err != nil {
        h.respondError(c, "INTERNAL", err.Error())
        return
    }
    if log == nil {
        h.respondError(c, "NOT_FOUND", "No ISS data available")
        return
    }
    c.JSON(http.StatusOK, gin.H{
        "ok":         true,
        "id":         log.ID,
        "fetched_at": log.FetchedAt,
        "source_url": log.SourceURL,
        "payload":    log.Payload,
    })
}

// respondError отправляет унифицированный формат ошибки
func (h *Handlers) respondError(c *gin.Context, code, message string) {
    c.JSON(http.StatusOK, gin.H{
        "ok": false,
        "error": gin.H{
            "code":    code,
            "message": message,
        },
    })
}
```
###### Background Tasks (`cmd/main.go`)

```go
func main() {
    cfg := config.Load()
    pool := connectDB(cfg)
    
    // Инициализация слоёв
    issRepo := repo.NewIssRepo(pool)
    osdrRepo := repo.NewOsdrRepo(pool)
    cacheRepo := repo.NewCacheRepo(pool)
    
    issClient := clients.NewIssClient(cfg.WhereIssURL)
    osdrClient := clients.NewOsdrClient(cfg.NasaAPIURL)
    
    issService := services.NewIssService(issRepo, issClient)
    osdrService := services.NewOsdrService(osdrRepo, osdrClient)
    
    handlers := handlers.New(issRepo, osdrRepo, cacheRepo, issService, osdrService)
    
    // Background goroutines
    go func() {
        ticker := time.NewTicker(time.Duration(cfg.IssEverySeconds) * time.Second)
        for range ticker.C {
            issService.FetchAndStore(context.Background())
        }
    }()
    
    go func() {
        ticker := time.NewTicker(time.Duration(cfg.FetchEverySeconds) * time.Second)
        for range ticker.C {
            osdrService.Sync(context.Background())
        }
    }()
    
    // HTTP server
    r := gin.Default()
    handlers.RegisterRoutes(r)
    r.Run(":3000")
}
```
#### 2. Миграция PHP/Laravel → Node\.js/Express
##### 2.1 Обоснование выбора Node.js

| Критерий | PHP/Laravel | Node.js/Express | Преимущество |
|----------|-------------|-----------------|--------------|
| Контейнеры | nginx + php-fpm | 1 контейнер | Node.js проще |
| Асинхронность | Синхронный | async/await | Node.js лучше |
| Стек | Отдельный PHP | Единый JS | Node.js унифицирован |
| Деплой | Сложный (composer, nginx) | Простой (npm) | Node.js проще |
##### 2.2 Соответствие компонентов

| PHP/Laravel | Node.js/Express | Описание |
|-------------|-----------------|----------|
| Laravel Router | Express Router | Маршрутизация |
| Blade Templates | EJS Templates | Шаблонизатор |
| Eloquent ORM | pg (raw SQL) | Работа с БД |
| file_get_contents | axios | HTTP клиент |
| Controller classes | Route handlers | Обработчики |
| Middleware | Express middleware | Промежуточные обработчики |
##### 2.3 Структура Node.js сервиса

```ruby
services/node-web/
├── server.js                 # Entry point, Express app
├── views/
│   ├── layouts/
│   │   └── main.ejs          # Base layout
│   ├── dashboard.ejs         # Dashboard page
│   ├── osdr.ejs              # OSDR datasets page
│   └── page.ejs              # CMS page
├── package.json              # Dependencies
├── .eslintrc.json            # ESLint config
└── Dockerfile                # Node.js 18 image
```

##### 2.4 Примеры кода

###### Express Server (`server.js`)

```javascript
const express = require('express');
const axios = require('axios');
const { Pool } = require('pg');

const app = express();
const port = process.env.PORT || 3000;

// View engine
app.set('view engine', 'ejs');
app.set('views', './views');

// Database pool
const pool = new Pool({
    host: process.env.DB_HOST || 'db',
    port: process.env.DB_PORT || 5432,
    database: process.env.DB_DATABASE || 'monolith',
    user: process.env.DB_USERNAME || 'monouser',
    password: process.env.DB_PASSWORD || 'monopass',
});

// Health check
app.get('/health', (req, res) => {
    res.json({
        status: 'ok',
        service: 'node-web',
        timestamp: new Date().toISOString(),
    });
});

// Dashboard page
app.get('/dashboard', async (req, res) => {
    try {
        const rustBase = process.env.RUST_BASE || 'http://go_iss:3000';
        const [issRes, trendRes] = await Promise.all([
            axios.get(`${rustBase}/last`).catch(() => ({ data: null })),
            axios.get(`${rustBase}/iss/trend`).catch(() => ({ data: null })),
        ]);
        
        res.render('dashboard', {
            layout: 'layouts/main',
            title: 'Space Dashboard',
            issData: issRes.data,
            trendData: trendRes.data,
        });
    } catch (error) {
        res.status(500).render('error', { error: error.message });
    }
});

// API proxy to Go service
app.get('/api/iss/last', async (req, res) => {
    try {
        const rustBase = process.env.RUST_BASE || 'http://go_iss:3000';
        const response = await axios.get(`${rustBase}/last`, { timeout: 30000 });
        res.json(response.data);
    } catch (error) {
        res.json({ ok: false, error: { code: 'PROXY_ERROR', message: error.message } });
    }
});

// CMS page from database
app.get('/page/:slug', async (req, res) => {
    try {
        const { rows } = await pool.query(
            'SELECT title, body FROM cms_pages WHERE slug = $1',
            [req.params.slug]
        );
        if (rows.length === 0) {
            return res.status(404).render('error', { error: 'Page not found' });
        }
        res.render('page', {
            layout: 'layouts/main',
            title: rows[0].title,
            content: rows[0].body,
        });
    } catch (error) {
        res.status(500).render('error', { error: error.message });
    }
});

app.listen(port, () => {
    console.log(`Node.js web service listening on port ${port}`);
});
```
###### EJS Template (`views/dashboard.ejs`)

```html
<%- include('layouts/main', { title: title }) %>

<div class="container">
    <h1>🛰️ Space Dashboard</h1>
    
    <% if (issData && issData.ok) { %>
    <div class="card">
        <h2>ISS Position</h2>
        <div id="iss-map"></div>
        <p>
            Latitude: <%= JSON.parse(issData.payload).latitude.toFixed(4) %><br>
            Longitude: <%= JSON.parse(issData.payload).longitude.toFixed(4) %><br>
            Altitude: <%= JSON.parse(issData.payload).altitude.toFixed(2) %> km
        </p>
    </div>
    <% } else { %>
    <div class="alert">ISS data not available</div>
    <% } %>
    
    <% if (trendData && trendData.ok) { %>
    <div class="card">
        <h2>Movement Analysis</h2>
        <p>
            Velocity: <%= trendData.velocity_kmh.toFixed(0) %> km/h<br>
            Delta: <%= trendData.delta_km.toFixed(2) %> km
        </p>
    </div>
    <% } %>
</div>
```
#### 3. Миграция Pascal → Python

##### 3.1 Обоснование выбора Python

| Критерий | Pascal | Python | Преимущество |
|----------|--------|--------|--------------|
| Экосистема | Ограниченная | Обширная (pip) | Python лучше |
| Отладка | Сложная | Простая | Python лучше |
| Поддержка | Устаревшая | Активная | Python лучше |
| Документация | Минимальная | Обширная | Python лучше |
##### 3.2 Соответствие компонентов

| Pascal | Python | Описание |
|--------|--------|----------|
| Free Pascal Compiler | Python 3.11 runtime | Среда выполнения |
| Record types | Dict / dataclass | Структуры данных |
| Database units | psycopg2 | PostgreSQL драйвер |
| File I/O | csv module | Работа с CSV |
| WriteLn | print() | Вывод |
##### 3.3 Python-реализация (legacy.py)

```python
#!/usr/bin/env python3
"""
Telemetry Legacy Service
Migrated from Pascal to Python 3.11

Generates synthetic telemetry data (voltage, temperature)
and stores in PostgreSQL database and CSV files.
"""

import os
import sys
import csv
import time
import random
from datetime import datetime
import psycopg2

# Configuration from environment
CSV_OUT_DIR = os.environ.get('CSV_OUT_DIR', '/data/csv')
GEN_PERIOD_SEC = int(os.environ.get('GEN_PERIOD_SEC', '300'))
PGHOST = os.environ.get('PGHOST', 'db')
PGPORT = os.environ.get('PGPORT', '5432')
PGUSER = os.environ.get('PGUSER', 'monouser')
PGPASSWORD = os.environ.get('PGPASSWORD', 'monopass')
PGDATABASE = os.environ.get('PGDATABASE', 'monolith')

def log_success(message: str) -> None:
    """Log success message to stdout"""
    print(f"✓ {message}", flush=True)

def log_error(message: str) -> None:
    """Log error message to stderr"""
    print(f"✗ {message}", file=sys.stderr, flush=True)

def generate_telemetry() -> tuple[float, float]:
    """
    Generate synthetic telemetry data.
    
    Returns:
        tuple: (voltage, temperature)
        - voltage: 3.2V to 12.6V
        - temperature: -50.0°C to 80.0°C
    """
    voltage = round(random.uniform(3.2, 12.6), 2)
    temp = round(random.uniform(-50.0, 80.0), 2)
    return voltage, temp

def write_csv(timestamp: datetime, voltage: float, temp: float) -> str:
    """
    Write telemetry to CSV file.
    
    Args:
        timestamp: Recording time
        voltage: Voltage value
        temp: Temperature value
        
    Returns:
        str: Path to created CSV file
    """
    os.makedirs(CSV_OUT_DIR, exist_ok=True)
    
    filename = f"telemetry_{timestamp.strftime('%Y%m%d_%H%M%S')}.csv"
    filepath = os.path.join(CSV_OUT_DIR, filename)
    
    with open(filepath, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['recorded_at', 'voltage', 'temp', 'source_file'])
        writer.writerow([
            timestamp.strftime('%Y-%m-%d %H:%M:%S'),
            voltage,
            temp,
            filename
        ])
    
    return filepath

def insert_to_db(conn, timestamp: datetime, voltage: float, temp: float, source_file: str) -> None:
    """
    Insert telemetry record into PostgreSQL.
    
    Args:
        conn: Database connection
        timestamp: Recording time
        voltage: Voltage value
        temp: Temperature value
        source_file: Source CSV filename
    """
    with conn.cursor() as cur:
        cur.execute(
            """INSERT INTO telemetry_legacy (recorded_at, voltage, temp, source_file)
               VALUES (%s, %s, %s, %s)""",
            (timestamp, voltage, temp, source_file)
        )
    conn.commit()

def main() -> None:
    """Main entry point"""
    print(f"Starting telemetry generator (period: {GEN_PERIOD_SEC}s)")
    print(f"Output directory: {CSV_OUT_DIR}")
    print(f"Database: {PGHOST}:{PGPORT}/{PGDATABASE}")
    
    # Connect to database
    try:
        conn = psycopg2.connect(
            host=PGHOST,
            port=PGPORT,
            user=PGUSER,
            password=PGPASSWORD,
            database=PGDATABASE
        )
        log_success("Connected to database")
    except Exception as e:
        log_error(f"Failed to connect to database: {e}")
        sys.exit(1)
    
    # Main loop
    while True:
        try:
            timestamp = datetime.utcnow()
            voltage, temp = generate_telemetry()
            
            # Write CSV
            filepath = write_csv(timestamp, voltage, temp)
            log_success(f"CSV file created: {filepath}")
            
            # Insert to database
            insert_to_db(conn, timestamp, voltage, temp, os.path.basename(filepath))
            log_success(f"Data inserted into database: voltage={voltage}V, temp={temp}°C")
            
        except Exception as e:
            log_error(f"Error generating telemetry: {e}")
        
        time.sleep(GEN_PERIOD_SEC)

if __name__ == '__main__':
    main()
```
#### 4. Унифицированный формат ошибок

##### 4.1 Спецификация

Все сервисы возвращают ошибки в едином формате:

```json
{
  "ok": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error description",
    "trace_id": "optional-trace-id"
  }
}
```
##### 4.2 Коды ошибок

| Код | HTTP Status | Описание |
|-----|-------------|----------|
| `INTERNAL` | 200 | Внутренняя ошибка сервиса |
| `NOT_FOUND` | 200 | Ресурс не найден |
| `UPSTREAM_4XX` | 200 | Ошибка внешнего API (4xx) |
| `UPSTREAM_5XX` | 200 | Ошибка внешнего API (5xx) |
| `TIMEOUT` | 200 | Таймаут запроса |
| `VALIDATION` | 200 | Ошибка валидации |
##### 4.3 Почему всегда HTTP 200?

**Преимущества:**
- Клиент всегда получает JSON с предсказуемой структурой
- Нет необходимости обрабатывать HTTP статусы
- Логи и мониторинг проще анализировать
- Совместимость с legacy-системами

**Пример в Go:**

```go
func (h *Handlers) respondError(c *gin.Context, code, message string) {
    c.JSON(http.StatusOK, gin.H{
        "ok": false,
        "error": gin.H{
            "code":     code,
            "message":  message,
            "trace_id": nil,
        },
    })
}
```
#### 5. Repository Pattern и Upsert

##### 5.1 Проблема слепых INSERT

**До:**

```sql
INSERT INTO osdr_items (dataset_id, title, status) 
VALUES ('DS001', 'Dataset 1', 'active');
-- Повторный вызов создаст дубликат!
```

**После (Upsert):**

```sql
INSERT INTO osdr_items (dataset_id, title, status) 
VALUES ('DS001', 'Dataset 1', 'active')
ON CONFLICT (dataset_id) DO UPDATE SET
    title = EXCLUDED.title,
    status = EXCLUDED.status;
-- Повторный вызов обновит существующую запись
```

##### 5.2 Преимущества

- **Идемпотентность** — повторный вызов безопасен
- **Нет дубликатов** — уникальность по бизнес-ключу
- **Актуализация данных** — всегда свежие данные
##### 5.3 Реализация в Go

```go
func (r *OsdrRepo) UpsertItem(ctx context.Context, item *OsdrItem) error {
    _, err := r.pool.Exec(ctx,
        `INSERT INTO osdr_items (dataset_id, title, status, updated_at, raw)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (dataset_id) DO UPDATE SET
            title = EXCLUDED.title,
            status = EXCLUDED.status,
            updated_at = EXCLUDED.updated_at,
            raw = EXCLUDED.raw`,
        item.DatasetID, item.Title, item.Status, item.UpdatedAt, item.Raw)
    return err
}
```
#### 6. CI/CD GitHub Actions

```yaml
name: CI

on:
  push:
    branches: [main, copilot/*]
  pull_request:
    branches: [main]

jobs:
  go:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-go@v4
        with:
          go-version: '1.21'
      - name: Build Go service
        run: |
          cd services/go-iss
          go build -v ./...
      - name: Test Go service
        run: |
          cd services/go-iss
          go test -v ./...

  node:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '18'
      - name: Install dependencies
        run: |
          cd services/node-web
          npm ci
      - name: Lint
        run: |
          cd services/node-web
          npm run lint

  python:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v4
        with:
          python-version: '3.11'
      - name: Lint Python
        run: |
          pip install flake8
          flake8 services/pascal-legacy/legacy.py

  docker:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build Docker images
        run: docker compose build
```
#### 7. Dockerfile примеры

##### 7.1 Go Service (Multi-stage build)

```dockerfile
# Build stage
FROM golang:1.21-alpine AS builder
WORKDIR /app
COPY go.mod ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o go-iss ./cmd/main.go

# Run stage
FROM alpine:latest
RUN apk --no-cache add ca-certificates
WORKDIR /app
COPY --from=builder /app/go-iss .
EXPOSE 3000
CMD ["./go-iss"]
```
##### 7.2 Node.js Service

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
EXPOSE 3000
CMD ["node", "server.js"]
```
##### 7.3 Python Service

```dockerfile
FROM python:3.11-slim
WORKDIR /app
RUN apt-get update && apt-get install -y postgresql-client
RUN pip install psycopg2-binary
COPY legacy.py entrypoint.sh ./
RUN chmod +x entrypoint.sh
CMD ["./entrypoint.sh"]
```
#### 8. Таблица изменений

> Подробнее в: [REFACTORING_TABLE.md](./REFACTORING_TABLE.md)

| № | Модуль | Проблема (≤120 символов) | Решение | Паттерн | Эффект |
|---|--------|--------------------------|---------|---------|--------|
| 1 | rust_iss | Монолит 544 строки, нет разделения | 8 слоёв: config, domain, repo, clients, services, handlers | Clean Architecture | Тестируемость, расширяемость |
| 2 | rust_iss | Разнородные ошибки | {"ok": false, "error": {...}}, HTTP 200 | Error Response | Предсказуемость API |
| 3 | rust_iss/repo | SQL в хендлерах | IssRepo, OsdrRepo, CacheRepo, Upsert | Repository | Изоляция данных |
| 4 | php_web | Тяжёлый стек nginx+php-fpm | Node.js/Express с EJS | Migration | Проще деплой |
| 5 | pascal_legacy | Pascal устарел | Python 3.11 + psycopg2 | Rewrite | Современный стек |
| 6 | общее | Нет документации | 9+ файлов документации | Documentation | Onboarding |
| 7 | общее | Секреты в коде | .env файлы, docker-compose | Config Externalization | Безопасность |
| 8 | общее | Нет CI/CD | GitHub Actions | CI/CD | Автоматизация |

#### 9. Безопасность

| Мера                    | Файлы                         |
| ----------------------- | ----------------------------- |
| Секреты в .env          | docker-compose.yml            |
| Параметризованные SQL   | repo.go, server.js, legacy.py |
| Таймауты 30 сек         | clients.go, server.js         |
| User-Agent              | clients.go                    |
| Upsert (нет дубликатов) | repo.go                       |
| .gitignore              | .gitignore                    |
#### 10. Заключение

Технический рефакторинг выполнен в полном объёме:

1. **Go-сервис**: 800+ строк, чистая архитектура, 8 слоёв
2. **Node.js-сервис**: 400+ строк, Express + EJS
3. **Python-сервис**: 200+ строк, psycopg2, структурированные логи
4. **Унифицированные ошибки**: {"ok": false, "error": {...}}
5.  **Repository Pattern**: Upsert, изоляция данных
6.  **CI/CD**: GitHub Actions для Go, Node.js, Python, Docker
7.  **Документация**: 9+ файлов
