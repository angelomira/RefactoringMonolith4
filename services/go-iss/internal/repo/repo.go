// Package repo provides database repositories
package repo

import (
	"context"
	"encoding/json"
	"time"

	"go-iss/internal/domain"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
)

// IssRepo handles ISS data persistence
type IssRepo struct {
	pool *pgxpool.Pool
}

// NewIssRepo creates a new ISS repository
func NewIssRepo(pool *pgxpool.Pool) *IssRepo {
	return &IssRepo{pool: pool}
}

// InsertLog inserts a new ISS fetch log
func (r *IssRepo) InsertLog(ctx context.Context, sourceURL string, payload json.RawMessage) error {
	_, err := r.pool.Exec(ctx,
		"INSERT INTO iss_fetch_log (source_url, payload) VALUES ($1, $2)",
		sourceURL, payload)
	return err
}

// GetLatest retrieves the most recent ISS log
func (r *IssRepo) GetLatest(ctx context.Context) (*domain.IssLog, error) {
	row := r.pool.QueryRow(ctx,
		"SELECT id, fetched_at, source_url, payload FROM iss_fetch_log ORDER BY id DESC LIMIT 1")

	var log domain.IssLog
	err := row.Scan(&log.ID, &log.FetchedAt, &log.SourceURL, &log.Payload)
	if err == pgx.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &log, nil
}

// GetLastN retrieves the last N ISS logs
func (r *IssRepo) GetLastN(ctx context.Context, n int) ([]struct {
	FetchedAt time.Time
	Payload   json.RawMessage
}, error) {
	rows, err := r.pool.Query(ctx,
		"SELECT fetched_at, payload FROM iss_fetch_log ORDER BY id DESC LIMIT $1", n)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var results []struct {
		FetchedAt time.Time
		Payload   json.RawMessage
	}
	for rows.Next() {
		var item struct {
			FetchedAt time.Time
			Payload   json.RawMessage
		}
		if err := rows.Scan(&item.FetchedAt, &item.Payload); err != nil {
			return nil, err
		}
		results = append(results, item)
	}
	return results, nil
}

// OsdrRepo handles OSDR data persistence
type OsdrRepo struct {
	pool *pgxpool.Pool
}

// NewOsdrRepo creates a new OSDR repository
func NewOsdrRepo(pool *pgxpool.Pool) *OsdrRepo {
	return &OsdrRepo{pool: pool}
}

// UpsertItem upserts an OSDR item
func (r *OsdrRepo) UpsertItem(ctx context.Context, datasetID, title, status *string, updatedAt *time.Time, raw json.RawMessage) error {
	if datasetID != nil {
		_, err := r.pool.Exec(ctx, `
			INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw)
			VALUES($1,$2,$3,$4,$5)
			ON CONFLICT (dataset_id) DO UPDATE
			SET title=EXCLUDED.title, status=EXCLUDED.status,
			    updated_at=EXCLUDED.updated_at, raw=EXCLUDED.raw`,
			datasetID, title, status, updatedAt, raw)
		return err
	}
	_, err := r.pool.Exec(ctx,
		"INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw) VALUES($1,$2,$3,$4,$5)",
		nil, title, status, updatedAt, raw)
	return err
}

// ListItems lists OSDR items with limit
func (r *OsdrRepo) ListItems(ctx context.Context, limit int) ([]domain.OsdrItem, error) {
	rows, err := r.pool.Query(ctx,
		"SELECT id, dataset_id, title, status, updated_at, inserted_at, raw FROM osdr_items ORDER BY inserted_at DESC LIMIT $1",
		limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var items []domain.OsdrItem
	for rows.Next() {
		var item domain.OsdrItem
		if err := rows.Scan(&item.ID, &item.DatasetID, &item.Title, &item.Status, &item.UpdatedAt, &item.InsertedAt, &item.Raw); err != nil {
			return nil, err
		}
		items = append(items, item)
	}
	return items, nil
}

// CountItems counts total OSDR items
func (r *OsdrRepo) CountItems(ctx context.Context) (int64, error) {
	var count int64
	err := r.pool.QueryRow(ctx, "SELECT count(*) FROM osdr_items").Scan(&count)
	return count, err
}

// CacheRepo handles space cache persistence
type CacheRepo struct {
	pool *pgxpool.Pool
}

// NewCacheRepo creates a new cache repository
func NewCacheRepo(pool *pgxpool.Pool) *CacheRepo {
	return &CacheRepo{pool: pool}
}

// Write writes data to cache
func (r *CacheRepo) Write(ctx context.Context, source string, payload json.RawMessage) error {
	_, err := r.pool.Exec(ctx,
		"INSERT INTO space_cache(source, payload) VALUES ($1,$2)",
		source, payload)
	return err
}

// GetLatest gets latest cache entry for a source
func (r *CacheRepo) GetLatest(ctx context.Context, source string) (*domain.SpaceCache, error) {
	row := r.pool.QueryRow(ctx,
		"SELECT id, source, fetched_at, payload FROM space_cache WHERE source = $1 ORDER BY id DESC LIMIT 1",
		source)

	var cache domain.SpaceCache
	err := row.Scan(&cache.ID, &cache.Source, &cache.FetchedAt, &cache.Payload)
	if err == pgx.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	return &cache, nil
}

// InitDB initializes database tables
func InitDB(ctx context.Context, pool *pgxpool.Pool) error {
	queries := []string{
		`CREATE TABLE IF NOT EXISTS iss_fetch_log(
			id BIGSERIAL PRIMARY KEY,
			fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
			source_url TEXT NOT NULL,
			payload JSONB NOT NULL
		)`,
		`CREATE TABLE IF NOT EXISTS osdr_items(
			id BIGSERIAL PRIMARY KEY,
			dataset_id TEXT,
			title TEXT,
			status TEXT,
			updated_at TIMESTAMPTZ,
			inserted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
			raw JSONB NOT NULL
		)`,
		`CREATE UNIQUE INDEX IF NOT EXISTS ux_osdr_dataset_id
		 ON osdr_items(dataset_id) WHERE dataset_id IS NOT NULL`,
		`CREATE TABLE IF NOT EXISTS space_cache(
			id BIGSERIAL PRIMARY KEY,
			source TEXT NOT NULL,
			fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
			payload JSONB NOT NULL
		)`,
		`CREATE INDEX IF NOT EXISTS ix_space_cache_source
		 ON space_cache(source,fetched_at DESC)`,
	}

	for _, q := range queries {
		if _, err := pool.Exec(ctx, q); err != nil {
			return err
		}
	}
	return nil
}
