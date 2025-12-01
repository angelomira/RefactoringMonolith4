FROM python:3.11-slim

WORKDIR /app

# Install PostgreSQL client
RUN apt-get update && apt-get install -y \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies
RUN pip install --no-cache-dir psycopg2-binary

# Copy application
COPY legacy.py /app/
COPY entrypoint.sh /app/

RUN chmod +x /app/legacy.py /app/entrypoint.sh

# Create data directory
RUN mkdir -p /data/csv

ENTRYPOINT ["/app/entrypoint.sh"]
