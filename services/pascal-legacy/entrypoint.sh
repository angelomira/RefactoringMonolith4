#!/bin/bash
set -e

echo "Legacy telemetry service starting..."
echo "Database: ${PGHOST}:${PGPORT}/${PGDATABASE}"
echo "CSV output: ${CSV_OUT_DIR}"
echo "Generation period: ${GEN_PERIOD_SEC}s"

# Wait for database to be ready
until pg_isready -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$PGDATABASE"; do
  echo "Waiting for PostgreSQL..."
  sleep 2
done

echo "Database is ready. Starting telemetry generator..."

# Run the Python script
exec python3 /app/legacy.py
