# Node.js Web Service

Space Dashboard web application, migrated from PHP/Laravel to Node.js/Express.

## Technology Stack

- **Runtime**: Node.js 18+
- **Framework**: Express.js 4.x
- **Templates**: EJS (Embedded JavaScript)
- **Database**: PostgreSQL (via `pg` package)
- **HTTP Client**: Axios

## Features

- Dashboard with ISS tracking map and charts
- JWST image gallery with filtering
- Astronomy events from AstronomyAPI
- OSDR datasets listing
- CMS pages from database
- API proxy to rust_iss service

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| PORT | Server port | 3000 |
| RUST_BASE | Rust ISS service URL | http://rust_iss:3000 |
| JWST_HOST | JWST API host | https://api.jwstapi.com |
| JWST_API_KEY | JWST API key | - |
| ASTRO_APP_ID | AstronomyAPI app ID | - |
| ASTRO_APP_SECRET | AstronomyAPI secret | - |
| DB_HOST | PostgreSQL host | db |
| DB_PORT | PostgreSQL port | 5432 |
| DB_DATABASE | Database name | monolith |
| DB_USERNAME | Database user | monouser |
| DB_PASSWORD | Database password | monopass |

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| GET / | Redirect to /dashboard |
| GET /dashboard | Main dashboard page |
| GET /osdr | OSDR datasets page |
| GET /page/:slug | CMS page by slug |
| GET /api/iss/last | Proxy to rust_iss /last |
| GET /api/iss/trend | Proxy to rust_iss /iss/trend |
| GET /api/jwst/feed | JWST image feed |
| GET /api/astro/events | Astronomy events |
| GET /health | Health check |

## Running Locally

```bash
# Install dependencies
npm install

# Start in development mode
npm run dev

# Start in production mode
npm start
```

## Docker

```bash
# Build image
docker build -t node-web .

# Run container
docker run -p 3000:3000 \
  -e RUST_BASE=http://rust_iss:3000 \
  -e DB_HOST=db \
  node-web
```

## Migration Notes

This service replaces the PHP/Laravel web application with equivalent functionality:

| PHP/Laravel | Node.js/Express |
|-------------|-----------------|
| Laravel routing | Express Router |
| Blade templates | EJS templates |
| Eloquent ORM | pg (raw SQL) |
| file_get_contents | axios |
| Controller classes | Route handlers |

All API endpoints and page routes maintain backward compatibility.
