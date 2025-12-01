### Проверка установки Docker

```bash
docker --version
# Docker version 20.10.0 или выше

docker compose version
# Docker Compose version v2.0.0 или выше
```

### Шаг 1: Клонирование репозитория

```bash
git clone https://github.com/Falcion/Debug.git
cd Debug
git checkout copilot/refactor-codebase-for-practice-4
```

### Шаг 2: Сборка и запуск сервисов

####  Вариант A: Запуск с логами в консоли

```bash
docker compose up --build
```

*Этот вариант удобен для наблюдения за логами всех сервисов в реальном времени.*
#### Вариант B: Запуск в фоновом режиме

```bash
docker compose up --build -d
```

Для просмотра логов:

```bash
# Все сервисы
docker compose logs -f

# Только Go ISS-сервис (мигрирован с Rust)
docker compose logs -f go_iss

# Только Node.js веб-сервис
docker compose logs -f node_web

# Только Python legacy
docker compose logs -f python_legacy
```

#### Ожидаемый результат

Вы должны увидеть следующие сообщения:

```
[+] Running 4/4
 ✔ Container iss_db          Started
 ✔ Container go_iss          Started
 ✔ Container node_web        Started
 ✔ Container python_legacy   Started
```

**Время запуска:** 2-5 минут (в зависимости от скорости интернета)

### Шаг 3: Проверка работоспособности сервисов

#### 3.1 Проверка Node.js веб-сервиса (мигрирован с PHP/Laravel)

```bash
# Health check
curl http://localhost:8080/health
```

**Ожидаемый ответ:**
```json
{
  "status": "ok",
  "service": "node-web",
  "timestamp": "2024-11-27T13:47:51.596Z"
}
```

#### 3.2 Проверка Go ISS API (мигрирован с Rust)

```bash
# Health check
curl http://localhost:8081/health
```

**Ожидаемый ответ:**
```json
{
  "status": "ok",
  "now": "2024-11-24T10:30:15.123456Z"
}
```

```bash
# Получить последнюю позицию МКС
curl http://localhost:8081/last | jq
```

**Ожидаемый ответ:**
```json
{
  "ok": true,
  "id": 1,
  "fetched_at": "2024-11-24T10:30:00Z",
  "source_url": "https://api.wheretheiss.at/v1/satellites/25544",
  "payload": {
    "latitude": 45.123,
    "longitude": -12.456,
    "altitude": 408.5,
    "velocity": 27600.0
  }
}
```

```bash
# Получить тренд движения МКС
curl http://localhost:8081/iss/trend | jq
```

**Ожидаемый ответ:**
```json
{
  "ok": true,
  "movement": true,
  "delta_km": 120.5,
  "dt_sec": 120.0,
  "velocity_kmh": 27600.0,
  "from_time": "2024-11-24T10:28:00Z",
  "to_time": "2024-11-24T10:30:00Z",
  "from_lat": 44.5,
  "from_lon": -13.0,
  "to_lat": 45.123,
  "to_lon": -12.456
}
```

```bash
# Список OSDR датасетов
curl http://localhost:8081/osdr/list | jq
```

```bash
# Сводка всех космических данных
curl http://localhost:8081/space/summary | jq
```

#### 3.2 Проверка веб-интерфейса (PHP/Laravel)

Откройте в браузере:
```
http://localhost:8080/dashboard
```

**Что вы должны увидеть:**
- Карта с текущей позицией МКС
- Метрики (скорость, высота)
- Галерея изображений JWST (загружается асинхронно)
- Графики и данные о космических событиях

#### 3.3 Проверка базы данных

```bash
# Подключиться к PostgreSQL
docker compose exec db psql -U monouser -d monolith
```

В PostgreSQL выполните:

```sql
-- Проверить таблицы
\dt

-- Посмотреть последние записи МКС
SELECT id, fetched_at, source_url FROM iss_fetch_log ORDER BY id DESC LIMIT 5;

-- Посмотреть OSDR датасеты
SELECT id, dataset_id, title, inserted_at FROM osdr_items ORDER BY id DESC LIMIT 5;

-- Проверить телеметрию от legacy-сервиса
SELECT * FROM telemetry_legacy ORDER BY id DESC LIMIT 5;

-- Посмотреть кэш космических данных
SELECT id, source, fetched_at FROM space_cache ORDER BY id DESC LIMIT 10;

-- Выход
\q
```

**Ожидаемый результат:**
- Таблица `iss_fetch_log` должна содержать записи (обновляются каждые 2 минуты)
- Таблица `telemetry_legacy` должна содержать записи (обновляются каждые 5 минут)
- Таблицы `osdr_items` и `space_cache` могут быть пустыми сразу после старта (обновляются каждые 10-60 минут)

#### 3.4 Проверка Python Legacy-сервиса

```bash
# Просмотр логов legacy-сервиса
docker compose logs python_legacy
```

**Ожидаемые логи:**
```
Starting telemetry generator (period: 300s)
Output directory: /data/csv
Database: db:5432/monolith
✓ CSV file created: /data/csv/telemetry_20241124_103015.csv
✓ Data inserted into database: voltage=8.45V, temp=-12.34°C
```

```bash
# Проверить созданные CSV файлы
docker compose exec python_legacy ls -lh /data/csv/
```
### Шаг 4: Проверка паттернов и архитектуры

#### 4.1 Проверка чистой архитектуры (Rust)

```bash
# Посмотреть структуру проекта
tree services/rust-iss/src/

services/rust-iss/src/
├── main.rs          # Entry point
├── config/          # Configuration layer
│   └── mod.rs
├── domain/          # Domain models
│   └── mod.rs
├── repo/            # Repository layer
│   └── mod.rs
├── clients/         # External API clients
│   └── mod.rs
├── services/        # Business logic
│   └── mod.rs
├── handlers/        # HTTP handlers
│   └── mod.rs
├── routes/          # Route definitions
│   └── mod.rs
├── errors/          # Error handling
│   └── mod.rs
└── utils.rs         # Utilities + tests
```

#### 4.2 Проверка унифицированного формата ошибок

```bash
# Попробовать запросить несуществующий источник данных
curl http://localhost:8081/space/invalid/latest | jq
```

**Ожидаемый ответ (HTTP 200 с ok=false):**
```json
{
  "ok": true,
  "source": "invalid",
  "message": "no data"
}
```

#### 4.3 Проверка Upsert по бизнес-ключам

```bash
# Синхронизировать OSDR два раза подряд
curl http://localhost:8081/osdr/sync
curl http://localhost:8081/osdr/sync

# Проверить, что нет дубликатов
docker compose exec db psql -U monouser -d monolith -c \
  "SELECT dataset_id, count(*) FROM osdr_items WHERE dataset_id IS NOT NULL GROUP BY dataset_id HAVING count(*) > 1;"
```

**Ожидаемый результат:** Пустой результат (нет дубликатов)
### Шаг 5: Запуск тестов

#### Тесты Go-сервиса

```bash
# Войти в контейнер
docker compose exec go_iss /bin/sh

# Запустить тесты
cargo test

# Выход
exit
```

**Ожидаемый результат:**
```
running 11 tests
test utils::tests::test_haversine_km_known_distance ... ok
test utils::tests::test_haversine_km_zero_distance ... ok
test utils::tests::test_num_from_float ... ok
test utils::tests::test_num_from_invalid ... ok
test utils::tests::test_num_from_string ... ok
test utils::tests::test_s_pick_finds_first ... ok
test utils::tests::test_s_pick_finds_second ... ok
test utils::tests::test_s_pick_not_found ... ok
test utils::tests::test_t_pick_from_unix_timestamp ... ok
test utils::tests::test_t_pick_iso_format ... ok
test utils::tests::test_t_pick_not_found ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured
```

### Шаг 6: Тестовые сценарии

#### Сценарий 1: Мониторинг МКС

1. Откройте http://localhost:8080/dashboard
2. Вы должны увидеть карту с позицией МКС
3. Подождите 2 минуты
4. Обновите страницу - позиция МКС должна измениться
#### Сценарий 2: Галерея JWST

1. На http://localhost:8080/dashboard прокрутите до раздела "JWST Gallery"
2. Должны загрузиться изображения телескопа James Webb
3. Изображения должны быть кликабельны

#### Сценарий 3: Космические события

1. Запросите: `curl http://localhost:8081/space/summary | jq`
2. Вы должны увидеть данные о:
   - APOD (Астрономическое фото дня)
   - NEO (Околоземные объекты)
   - Солнечные вспышки (FLR)
   - Корональные выбросы массы (CME)
   - Следующий запуск SpaceX
#### Сценарий 4: Телеметрия

1. Подождите 5 минут после запуска
2. Проверьте БД: должны появиться записи в `telemetry_legacy`
3. Проверьте файлы: `docker compose exec python_legacy ls /data/csv/`

### Шаг 7: Остановка и очистка

#### Остановка сервисов

```bash
# Остановить все сервисы
docker compose down

# Остановить и удалить volumes (очистка БД)
docker compose down -v
```

#### Полная очистка

```bash
# Удалить все контейнеры, образы и volumes
docker compose down -v --rmi all
```

### Решение проблем

#### Проблема: Порт уже занят

```bash
# Найти процесс на порту 8080
sudo lsof -i :8080

# Или использовать другие порты
# Отредактируйте docker-compose.yml:
# ports: - "9080:80"  # вместо 8080
```
#### Проблема: Сервис не запускается

```bash
# Просмотреть логи конкретного сервиса
docker compose logs [service_name]

# Примеры:
docker compose logs go_iss
docker compose logs php
docker compose logs db
```
#### Проблема: База данных не готова

```bash
# Проверить статус БД
docker compose ps db

# Переподключиться к БД
docker compose restart go_iss php python_legacy
```
