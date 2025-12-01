#!/usr/bin/env python3
"""
Legacy telemetry CSV generator - Python rewrite
Generates random telemetry data and inserts it into PostgreSQL database
"""

import os
import sys
import time
import random
import csv
from datetime import datetime
from pathlib import Path
import psycopg2
from psycopg2 import sql

def get_env(name: str, default: str = "") -> str:
    """Get environment variable with default value"""
    return os.environ.get(name, default)

def generate_csv_row() -> dict:
    """Generate a single row of telemetry data"""
    return {
        'recorded_at': datetime.now().strftime('%Y-%m-%d %H:%M:%S'),
        'voltage': round(random.uniform(3.2, 12.6), 2),
        'temp': round(random.uniform(-50.0, 80.0), 2),
    }

def generate_and_insert():
    """Generate CSV file and insert data into PostgreSQL"""
    # Configuration from environment
    csv_out_dir = get_env('CSV_OUT_DIR', '/data/csv')
    pg_host = get_env('PGHOST', 'db')
    pg_port = get_env('PGPORT', '5432')
    pg_user = get_env('PGUSER', 'monouser')
    pg_password = get_env('PGPASSWORD', 'monopass')
    pg_database = get_env('PGDATABASE', 'monolith')
    
    # Ensure output directory exists
    Path(csv_out_dir).mkdir(parents=True, exist_ok=True)
    
    # Generate timestamp and filename
    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
    filename = f'telemetry_{timestamp}.csv'
    filepath = os.path.join(csv_out_dir, filename)
    
    # Generate telemetry data
    data = generate_csv_row()
    data['source_file'] = filename
    
    # Write CSV file
    try:
        with open(filepath, 'w', newline='') as f:
            writer = csv.DictWriter(f, fieldnames=['recorded_at', 'voltage', 'temp', 'source_file'])
            writer.writeheader()
            writer.writerow(data)
        print(f"✓ CSV file created: {filepath}", file=sys.stdout)
    except Exception as e:
        print(f"✗ Error writing CSV: {e}", file=sys.stderr)
        return
    
    # Insert into PostgreSQL
    try:
        conn = psycopg2.connect(
            host=pg_host,
            port=pg_port,
            user=pg_user,
            password=pg_password,
            database=pg_database
        )
        
        with conn.cursor() as cursor:
            # Insert telemetry data
            cursor.execute(
                """
                INSERT INTO telemetry_legacy (recorded_at, voltage, temp, source_file)
                VALUES (%s, %s, %s, %s)
                """,
                (data['recorded_at'], data['voltage'], data['temp'], data['source_file'])
            )
            conn.commit()
        
        conn.close()
        print(f"✓ Data inserted into database: voltage={data['voltage']}V, temp={data['temp']}°C", file=sys.stdout)
    
    except psycopg2.Error as e:
        print(f"✗ Database error: {e}", file=sys.stderr)
    except Exception as e:
        print(f"✗ Unexpected error: {e}", file=sys.stderr)

def main():
    """Main loop - generate data periodically"""
    period_seconds = int(get_env('GEN_PERIOD_SEC', '300'))
    
    print(f"Starting telemetry generator (period: {period_seconds}s)", file=sys.stdout)
    print(f"Output directory: {get_env('CSV_OUT_DIR', '/data/csv')}", file=sys.stdout)
    print(f"Database: {get_env('PGHOST', 'db')}:{get_env('PGPORT', '5432')}/{get_env('PGDATABASE', 'monolith')}", file=sys.stdout)
    
    while True:
        try:
            generate_and_insert()
        except Exception as e:
            print(f"✗ Legacy error: {e}", file=sys.stderr)
        
        time.sleep(period_seconds)

if __name__ == '__main__':
    main()
