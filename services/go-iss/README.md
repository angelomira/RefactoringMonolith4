# Go ISS Service

Space data monitoring API service, migrated from Rust to Go.

## Technology Stack

- **Language**: Go 1.21+
- **Web Framework**: Gin
- **Database**: PostgreSQL (pgx driver)
- **HTTP Client**: net/http

## Features

- ISS position tracking and trend analysis
- NASA OSDR dataset synchronization
- NASA APIs integration (APOD, NeoWs, DONKI)
- SpaceX launch data
- Background data fetching tasks
- Unified error response format

## Architecture

```
cmd/
  main.go           → Application entry point
internal/
  config/           → Environment configuration
  domain/           → Domain models
  clients/          → External API clients
  repo/             → Database repositories
  services/         → Business logic
  handlers/         → HTTP handlers
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| GET /health | Health check |
| GET /last | Latest ISS position |
| GET /fetch | Trigger ISS fetch |
| GET /iss/trend | ISS movement analysis |
| GET /osdr/sync | Sync OSDR datasets |
| GET /osdr/list | List OSDR datasets |
| GET /space/:src/latest | Latest space data by source |
| GET /space/refresh | Refresh space data cache |
| GET /space/summary | Summary of all space data |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| DATABASE_URL | PostgreSQL connection string | postgres://monouser:monopass@db:5432/monolith |
| WHERE_ISS_URL | ISS API URL | https://api.wheretheiss.at/v1/satellites/25544 |
| NASA_API_URL | OSDR API URL | https://visualization.osdr.nasa.gov/biodata/api/v2/datasets/?format=json |
| NASA_API_KEY | NASA API key | DEMO_KEY |
| FETCH_EVERY_SECONDS | OSDR sync interval | 600 |
| ISS_EVERY_SECONDS | ISS tracking interval | 120 |
| APOD_EVERY_SECONDS | APOD fetch interval | 43200 |
| NEO_EVERY_SECONDS | NEO feed interval | 7200 |
| DONKI_EVERY_SECONDS | DONKI fetch interval | 3600 |
| SPACEX_EVERY_SECONDS | SpaceX fetch interval | 3600 |

## Running Locally

```bash
# Build
go build -o go-iss ./cmd/main.go

# Run
./go-iss
```

## Docker

```bash
# Build image
docker build -t go-iss .

# Run container
docker run -p 3000:3000 \
  -e DATABASE_URL=postgres://user:pass@host:5432/db \
  go-iss
```

## Migration Notes

This service replaces the Rust implementation with equivalent Go code:

| Rust | Go |
|------|-----|
| Axum | Gin |
| SQLx | pgx |
| reqwest | net/http |
| tokio | goroutines |
| Arc<T> | pointers |
| anyhow | error |

All API endpoints maintain backward compatibility.
