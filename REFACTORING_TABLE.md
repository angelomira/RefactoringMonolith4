# Таблица рефакторинга проекта

## Технологический стек (после миграции)

| Сервис | Язык | Фреймворк | БД драйвер |
|--------|------|-----------|------------|
| **go_iss** | Go 1.23 | Gin | pgx |
| **node_web** | Node.js | Express | pg |
| **python_legacy** | Python 3.11 | - | psycopg2 |
## История миграций

| Старый стек | Новый стек | Причина миграции |
|-------------|------------|------------------|
| Rust + Axum | **Go + Gin** | Проще синтаксис, быстрее компиляция, широкое использование в DevOps |
| PHP + Laravel | **Node.js + Express** | Единый JS-стек, меньше контейнеров, проще деплой |
| Pascal | **Python 3.11** | Современный язык, широкая экосистема, простота отладки |
## Обзор изменений

| № | Модуль | Проблема | Решение | Применённый паттерн | Эффект |
|---|--------|----------|---------|---------------------|--------|
| 1 | go_iss/internal | Монолитный код без структуры | Разделение на слои: config, domain, repo, clients, services, handlers | **Clean Architecture** | Повышение тестируемости, читаемости, расширяемости |
| 2 | go_iss/handlers | Разнородные ошибки, нет единого формата | Унифицированный Response с ok/error/code/message, всегда HTTP 200 | **Error Response Pattern** | Предсказуемость API, упрощение обработки ошибок на клиенте |
| 3 | go_iss/repo | SQL в хендлерах, смешение слоёв | Выделены репозитории: IssRepo, OsdrRepo, CacheRepo. Upsert по бизнес-ключам | **Repository Pattern** | Изоляция логики доступа к данным, предотвращение дубликатов |
| 4 | go_iss/clients | Дублирование HTTP-клиента, нет таймаутов | Унифицированный HTTP клиент с таймаутами (30с), user-agent | **Client Facade** | Защита от блокировок API, единая обработка таймаутов |
| 5 | go_iss/services | Бизнес-логика в handlers | Выделены сервисы: IssService, OsdrService, SpaceService | **Service Layer Pattern** | Переиспользование логики, упрощение тестирования |
| 6 | go_iss/domain | Отсутствие доменных моделей | Созданы типы: IssLog, OsdrItem, SpaceCache, IssTrend, Health, SpaceSummary | **Domain Model** | Типобезопасность, документация через код |
| 7 | go_iss/types | Нет чёткости по типам timestamp | Везде time.Time для TIMESTAMPTZ. Строгая типизация | **Strong Typing** | Предотвращение ошибок с временными зонами |
| 8 | python_legacy | Pascal (устаревший язык), сложность поддержки | Переписан на Python 3.11, сохранена вся логика | **Technology Migration** | Современный стек, простота отладки |
| 9 | python_legacy/logs | Нет структурированных логов | Логи в stdout (✓) и stderr (✗) с понятными сообщениями | **Structured Logging** | Упрощение мониторинга и отладки |
| 10 | python_legacy/CSV | Формат не задокументирован | Полная документация формата CSV, схемы БД в README.md | **Documentation Pattern** | Понятность для новых разработчиков |
| 11 | node_web | PHP/Laravel (тяжёлый стек, nginx+php-fpm) | Миграция на Node.js/Express с EJS шаблонами | **Technology Migration** | Упрощение деплоя, единый JS-стек |
| 12 | node_web/templates | Blade шаблоны (PHP-специфичные) | EJS шаблоны (JS-based) | **Template Engine Migration** | Унификация технологий |
| 13 | node_web/routing | Laravel Router | Express Router | **Framework Migration** | Более легковесный стек |
| 14 | node_web/http | file_get_contents (синхронный) | Axios (async/await) | **HTTP Client Modernization** | Асинхронность, лучшая обработка ошибок |
| 15 | node_web/db | Eloquent ORM | pg (raw SQL с параметрами) | **ORM to Raw SQL** | Простота, контроль над запросами |
| 16 | go_iss/DB | Слепые INSERT, возможны дубликаты | Upsert с ON CONFLICT для osdr_items по dataset_id | **Upsert Pattern** | Предотвращение дубликатов, идемпотентность |
| 17 | Общее | Нет .gitignore, коммитятся артефакты | Создан .gitignore: target/, node_modules/, vendor/, build/ | **VCS Best Practices** | Чистая история репозитория |
| 18 | Общее | Секреты в коде | Все секреты в .env, параметризация через docker-compose | **Configuration Externalization** | Безопасность, разные окружения |
| 19 | Общее/Docs | Нет инструкций для запуска | README.md, RUNNING.md, примеры запросов | **Documentation as Code** | Быстрый onboarding |
| 20 | go_iss/config | Конфигурация разбросана по коду | Централизованный модуль config/ с AppConfig | **Configuration Object** | Единая точка конфигурации |
## Миграция Rust → Go

| Компонент Rust | Компонент Go      | Описание              |
| -------------- | ----------------- | --------------------- |
| Axum           | **Gin**           | Веб-фреймворк         |
| SQLx           | **pgx**           | PostgreSQL драйвер    |
| reqwest        | **net/http**      | HTTP клиент           |
| tokio::spawn   | **goroutines**    | Конкурентность        |
| Arc<T>         | **pointers**      | Разделяемое состояние |
| anyhow::Error  | **error**         | Обработка ошибок      |
| serde          | **encoding/json** | Сериализация          |
| tracing        | **log**           | Логирование           |

**Преимущества:**
- Проще синтаксис и быстрее компиляция
- Встроенный tooling (go fmt, go test, go vet)
- Goroutines проще tokio runtime
- Единый бинарник без зависимостей
- Широкое использование в DevOps/Cloud

>Номинально, Rust является хорошим и в некоторых случаях идеальным решением, особенно в сфере космической разработки или требованиях высокой безопасности. Но в контексте практической работы - мы не умеем работать с Rust.
## Миграция PHP/Laravel → Node\.js/Express

| Компонент PHP | Компонент Node.js | Описание |
|---------------|-------------------|----------|
| Laravel Routes | **Express Router** | Маршрутизация HTTP запросов |
| Blade Templates | **EJS Templates** | Server-side рендеринг HTML |
| Eloquent ORM | **pg (node-postgres)** | Работа с PostgreSQL |
| file_get_contents | **axios** | HTTP клиент |
| Controllers | **Route handlers** | Обработчики запросов |
| Middleware | **Express middleware** | Промежуточная обработка |

**Преимущества:**
- Единый JavaScript стек (Node.js + frontend JS)
- Меньше контейнеров (без nginx, php-fpm)
- Проще деплой и масштабирование
- Современный async/await вместо sync операций
- npm экосистема для зависимостей
## Миграция Pascal → Python

| Компонент Pascal | Компонент Python | Описание |
|------------------|------------------|----------|
| Free Pascal Compiler | **Python 3.11 runtime** | Исполняющая среда |
| Record types | **Dict/dataclass** | Структуры данных |
| Database units | **psycopg2** | PostgreSQL драйвер |
| File I/O | **csv module** | Работа с CSV |
| WriteLn | **print()** | Вывод в консоль |

**Преимущества:**
- Широкая экосистема (pip)
- Проще отладка и тестирование
- Лучшая документация и сообщество
- Совместимость с современными инструментами
- Pascal является артефактом прошлого и не является на том же уровне open-source, что и Python;
## Примеры кода после миграции

### 1). Repository Pattern (Go)

```go
// Через репозиторий
func (r *IssRepo) InsertLog(ctx context.Context, url string, payload []byte) error {
    _, err := r.pool.Exec(ctx,
        "INSERT INTO iss_fetch_log (source_url, payload) VALUES ($1, $2)",
        url, payload)
    return err
}
```

### 2). Upsert по бизнес-ключам (Go)

```go
// Upsert по dataset_id - нет дубликатов
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

### 3). Unified Error Response (Go)

```go
type Response struct {
    OK    bool        `json:"ok"`
    Data  interface{} `json:"data,omitempty"`
    Error *ErrorInfo  `json:"error,omitempty"`
}

type ErrorInfo struct {
    Code    string `json:"code"`
    Message string `json:"message"`
}

// Всегда HTTP 200, клиент проверяет ok поле
c.JSON(http.StatusOK, Response{
    OK: false,
    Error: &ErrorInfo{
        Code:    "INTERNAL",
        Message: "Database error",
    },
})
```

### 4). Service Layer (Go)

```go
// Разделение бизнес-логики
type IssService struct {
    repo   *IssRepo
    client *IssClient
}

func (s *IssService) FetchAndStore(ctx context.Context) (*IssLog, error) {
    data, err := s.client.FetchPosition(ctx)
    if err != nil {
        return nil, err
    }
    return s.repo.InsertLog(ctx, data.URL, data.Payload)
}
```

## Показатели улучшений

| Метрика               | До                  | После               |
| --------------------- | ------------------- | ------------------- |
| Технологический стек  | Rust, PHP, Pascal   | Go, Node.js, Python |
| Количество слоёв (Go) | 1                   | 6                   |
| Языков legacy         | Pascal              | Python              |
| Формат ошибок         | Разнородный         | Унифицированный     |
| Дубликаты в БД        | Возможны            | Исключены           |
| Контейнеров на сервис | 2-3 (nginx+php-fpm) | 1                   |
## Заключение

Рефакторинг привёл к:
- Полной миграции технологического стека (Rust$\to$Go, PHP$\to$Node.js, Pascal$\to$Python)
- Чистой архитектуре с разделением ответственности
- Унифицированным ошибкам и типам
- Тестируемости и поддерживаемости
- Полной документации для преподавателя
- Безопасности (секреты в `.env`, защита от SQL-инъекций)
