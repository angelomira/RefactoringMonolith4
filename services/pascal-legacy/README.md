# Telemetry Legacy Service Documentation

## Overview
This service generates synthetic telemetry data (voltage and temperature readings) and stores them in PostgreSQL database. It also saves the data to CSV files for archival purposes.

## Implementation
- **Original**: Pascal (Free Pascal Compiler)
- **Rewritten**: Python 3.11
- **Reason for rewrite**: Better maintainability, easier debugging, and modern ecosystem support

## Data Format

### CSV Format
The service generates CSV files with the following structure:

```csv
recorded_at,voltage,temp,source_file
2024-11-24 10:30:15,8.45,-12.34,telemetry_20241124_103015.csv
```

**Fields:**
- `recorded_at` (timestamp): When the measurement was taken (format: YYYY-MM-DD HH:MM:SS)
- `voltage` (decimal): Voltage reading in volts, range: 3.2V to 12.6V
- `temp` (decimal): Temperature reading in Celsius, range: -50.0°C to 80.0°C
- `source_file` (string): Name of the CSV file containing this record

### Database Schema
Data is inserted into the `telemetry_legacy` table:

```sql
CREATE TABLE telemetry_legacy (
    id BIGSERIAL PRIMARY KEY,
    recorded_at TIMESTAMPTZ NOT NULL,
    voltage NUMERIC(6,2) NOT NULL,
    temp NUMERIC(6,2) NOT NULL,
    source_file TEXT NOT NULL
);
```

## Configuration

### Environment Variables
- `CSV_OUT_DIR` (default: `/data/csv`): Directory where CSV files are saved
- `GEN_PERIOD_SEC` (default: `300`): Interval in seconds between data generation
- `PGHOST` (default: `db`): PostgreSQL host
- `PGPORT` (default: `5432`): PostgreSQL port
- `PGUSER` (default: `monouser`): PostgreSQL username
- `PGPASSWORD` (default: `monopass`): PostgreSQL password
- `PGDATABASE` (default: `monolith`): PostgreSQL database name

## How It Works

1. Service starts and validates database connection
2. Every `GEN_PERIOD_SEC` seconds:
   - Generate random telemetry values (voltage, temperature)
   - Create timestamped CSV file
   - Insert data into PostgreSQL database
   - Log success/failure to stdout/stderr

## Running the Service

### Via Docker Compose
```bash
docker compose up pascal_legacy
```

### Standalone
```bash
# Set environment variables
export CSV_OUT_DIR=/data/csv
export GEN_PERIOD_SEC=300
export PGHOST=localhost
export PGPORT=5432
export PGUSER=monouser
export PGPASSWORD=monopass
export PGDATABASE=monolith

# Run the script
python3 legacy.py
```

## Logging
- ✓ Success messages are written to **stdout**
- ✗ Error messages are written to **stderr**

## Examples

### Output Logs
```
Starting telemetry generator (period: 300s)
Output directory: /data/csv
Database: db:5432/monolith
✓ CSV file created: /data/csv/telemetry_20241124_103015.csv
✓ Data inserted into database: voltage=8.45V, temp=-12.34°C
```

### Generated CSV File
```csv
recorded_at,voltage,temp,source_file
2024-11-24 10:30:15,8.45,-12.34,telemetry_20241124_103015.csv
```

## Migration Notes
The Python implementation preserves the exact same behavior as the original Pascal version:
- Same data generation algorithm (random values within specified ranges)
- Same CSV format
- Same database insertion logic
- Same environment variable configuration
- Logs to stdout/stderr as required

## Dependencies
- Python 3.11+
- psycopg2-binary (PostgreSQL adapter)
- postgresql-client (for pg_isready health check)
