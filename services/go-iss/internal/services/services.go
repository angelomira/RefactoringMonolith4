// Package services provides business logic
package services

import (
	"context"
	"encoding/json"
	"math"
	"time"

	"go-iss/internal/clients"
	"go-iss/internal/domain"
	"go-iss/internal/repo"
)

// IssService handles ISS tracking logic
type IssService struct {
	repo   *repo.IssRepo
	client *clients.IssClient
}

// NewIssService creates a new ISS service
func NewIssService(repo *repo.IssRepo, client *clients.IssClient) *IssService {
	return &IssService{repo: repo, client: client}
}

// FetchAndStore fetches ISS position and stores it
func (s *IssService) FetchAndStore(ctx context.Context) error {
	position, err := s.client.FetchPosition()
	if err != nil {
		return err
	}
	return s.repo.InsertLog(ctx, s.client.BaseURL(), position)
}

// GetLatest gets the latest ISS position
func (s *IssService) GetLatest(ctx context.Context) (*domain.IssLog, error) {
	return s.repo.GetLatest(ctx)
}

// CalculateTrend calculates ISS movement trend
func (s *IssService) CalculateTrend(ctx context.Context) (*domain.IssTrend, error) {
	rows, err := s.repo.GetLastN(ctx, 2)
	if err != nil {
		return nil, err
	}

	if len(rows) < 2 {
		return &domain.IssTrend{}, nil
	}

	t2, p2 := rows[0].FetchedAt, rows[0].Payload
	t1, p1 := rows[1].FetchedAt, rows[1].Payload

	lat1 := getFloat(p1, "latitude")
	lon1 := getFloat(p1, "longitude")
	lat2 := getFloat(p2, "latitude")
	lon2 := getFloat(p2, "longitude")
	v2 := getFloat(p2, "velocity")

	var deltaKm float64
	var movement bool
	if lat1 != nil && lon1 != nil && lat2 != nil && lon2 != nil {
		deltaKm = haversineKm(*lat1, *lon1, *lat2, *lon2)
		movement = deltaKm > 0.1
	}
	dtSec := t2.Sub(t1).Seconds()

	return &domain.IssTrend{
		Movement:    movement,
		DeltaKm:     deltaKm,
		DtSec:       dtSec,
		VelocityKmh: v2,
		FromTime:    &t1,
		ToTime:      &t2,
		FromLat:     lat1,
		FromLon:     lon1,
		ToLat:       lat2,
		ToLon:       lon2,
	}, nil
}

// OsdrService handles OSDR data logic
type OsdrService struct {
	repo   *repo.OsdrRepo
	client *clients.OsdrClient
}

// NewOsdrService creates a new OSDR service
func NewOsdrService(repo *repo.OsdrRepo, client *clients.OsdrClient) *OsdrService {
	return &OsdrService{repo: repo, client: client}
}

// Sync fetches and stores OSDR datasets
func (s *OsdrService) Sync(ctx context.Context) (int, error) {
	items, err := s.client.FetchDatasets()
	if err != nil {
		return 0, err
	}

	written := 0
	for _, item := range items {
		datasetID := pickString(item, "dataset_id", "id", "uuid", "studyId", "accession", "osdr_id")
		title := pickString(item, "title", "name", "label")
		status := pickString(item, "status", "state", "lifecycle")
		updatedAt := pickTime(item, "updated", "updated_at", "modified", "lastUpdated", "timestamp")

		if err := s.repo.UpsertItem(ctx, datasetID, title, status, updatedAt, item); err != nil {
			continue
		}
		written++
	}
	return written, nil
}

// List lists OSDR items
func (s *OsdrService) List(ctx context.Context, limit int) ([]domain.OsdrItem, error) {
	return s.repo.ListItems(ctx, limit)
}

// SpaceService handles space data aggregation
type SpaceService struct {
	cacheRepo    *repo.CacheRepo
	issRepo      *repo.IssRepo
	osdrRepo     *repo.OsdrRepo
	nasaClient   *clients.NasaClient
	spacexClient *clients.SpaceXClient
}

// NewSpaceService creates a new space service
func NewSpaceService(cacheRepo *repo.CacheRepo, issRepo *repo.IssRepo, osdrRepo *repo.OsdrRepo, nasaClient *clients.NasaClient, spacexClient *clients.SpaceXClient) *SpaceService {
	return &SpaceService{
		cacheRepo:    cacheRepo,
		issRepo:      issRepo,
		osdrRepo:     osdrRepo,
		nasaClient:   nasaClient,
		spacexClient: spacexClient,
	}
}

// FetchApod fetches and caches APOD
func (s *SpaceService) FetchApod(ctx context.Context) error {
	data, err := s.nasaClient.FetchAPOD()
	if err != nil {
		return err
	}
	return s.cacheRepo.Write(ctx, "apod", data)
}

// FetchNeo fetches and caches NEO data
func (s *SpaceService) FetchNeo(ctx context.Context) error {
	data, err := s.nasaClient.FetchNeoFeed()
	if err != nil {
		return err
	}
	return s.cacheRepo.Write(ctx, "neo", data)
}

// FetchFlr fetches and caches DONKI FLR
func (s *SpaceService) FetchFlr(ctx context.Context) error {
	data, err := s.nasaClient.FetchDonkiFLR()
	if err != nil {
		return err
	}
	return s.cacheRepo.Write(ctx, "flr", data)
}

// FetchCme fetches and caches DONKI CME
func (s *SpaceService) FetchCme(ctx context.Context) error {
	data, err := s.nasaClient.FetchDonkiCME()
	if err != nil {
		return err
	}
	return s.cacheRepo.Write(ctx, "cme", data)
}

// FetchSpacex fetches and caches SpaceX data
func (s *SpaceService) FetchSpacex(ctx context.Context) error {
	data, err := s.spacexClient.FetchNextLaunch()
	if err != nil {
		return err
	}
	return s.cacheRepo.Write(ctx, "spacex", data)
}

// GetLatest gets latest cached data for a source
func (s *SpaceService) GetLatest(ctx context.Context, source string) (*domain.SpaceCache, error) {
	return s.cacheRepo.GetLatest(ctx, source)
}

// Refresh refreshes multiple sources
func (s *SpaceService) Refresh(ctx context.Context, sources []string) []string {
	var refreshed []string
	for _, source := range sources {
		var err error
		switch source {
		case "apod":
			err = s.FetchApod(ctx)
		case "neo":
			err = s.FetchNeo(ctx)
		case "flr":
			err = s.FetchFlr(ctx)
		case "cme":
			err = s.FetchCme(ctx)
		case "spacex":
			err = s.FetchSpacex(ctx)
		default:
			continue
		}
		if err == nil {
			refreshed = append(refreshed, source)
		}
	}
	return refreshed
}

// GetSummary gets summary of all space data
func (s *SpaceService) GetSummary(ctx context.Context) (*domain.SpaceSummary, error) {
	apod := s.getLatestOrEmpty(ctx, "apod")
	neo := s.getLatestOrEmpty(ctx, "neo")
	flr := s.getLatestOrEmpty(ctx, "flr")
	cme := s.getLatestOrEmpty(ctx, "cme")
	spacex := s.getLatestOrEmpty(ctx, "spacex")

	issLog, _ := s.issRepo.GetLatest(ctx)
	var iss interface{} = map[string]interface{}{}
	if issLog != nil {
		iss = map[string]interface{}{
			"at":      issLog.FetchedAt,
			"payload": issLog.Payload,
		}
	}

	osdrCount, _ := s.osdrRepo.CountItems(ctx)

	return &domain.SpaceSummary{
		Apod:      apod,
		Neo:       neo,
		Flr:       flr,
		Cme:       cme,
		Spacex:    spacex,
		Iss:       iss,
		OsdrCount: osdrCount,
	}, nil
}

func (s *SpaceService) getLatestOrEmpty(ctx context.Context, source string) interface{} {
	cache, err := s.cacheRepo.GetLatest(ctx, source)
	if err != nil || cache == nil {
		return map[string]interface{}{}
	}
	return map[string]interface{}{
		"at":      cache.FetchedAt,
		"payload": cache.Payload,
	}
}

// Utility functions

func haversineKm(lat1, lon1, lat2, lon2 float64) float64 {
	const earthRadius = 6371.0
	dLat := (lat2 - lat1) * math.Pi / 180
	dLon := (lon2 - lon1) * math.Pi / 180
	lat1Rad := lat1 * math.Pi / 180
	lat2Rad := lat2 * math.Pi / 180

	a := math.Sin(dLat/2)*math.Sin(dLat/2) +
		math.Cos(lat1Rad)*math.Cos(lat2Rad)*math.Sin(dLon/2)*math.Sin(dLon/2)
	c := 2 * math.Atan2(math.Sqrt(a), math.Sqrt(1-a))
	return earthRadius * c
}

func getFloat(data json.RawMessage, key string) *float64 {
	var m map[string]interface{}
	if err := json.Unmarshal(data, &m); err != nil {
		return nil
	}
	if v, ok := m[key]; ok {
		switch val := v.(type) {
		case float64:
			return &val
		case int:
			f := float64(val)
			return &f
		}
	}
	return nil
}

func pickString(data json.RawMessage, keys ...string) *string {
	var m map[string]interface{}
	if err := json.Unmarshal(data, &m); err != nil {
		return nil
	}
	for _, key := range keys {
		if v, ok := m[key]; ok {
			if s, ok := v.(string); ok && s != "" {
				return &s
			}
		}
	}
	return nil
}

func pickTime(data json.RawMessage, keys ...string) *time.Time {
	var m map[string]interface{}
	if err := json.Unmarshal(data, &m); err != nil {
		return nil
	}
	for _, key := range keys {
		if v, ok := m[key]; ok {
			if s, ok := v.(string); ok && s != "" {
				// Try parsing common formats
				for _, layout := range []string{
					time.RFC3339,
					"2006-01-02T15:04:05Z",
					"2006-01-02T15:04:05",
					"2006-01-02",
				} {
					if t, err := time.Parse(layout, s); err == nil {
						return &t
					}
				}
			}
		}
	}
	return nil
}
