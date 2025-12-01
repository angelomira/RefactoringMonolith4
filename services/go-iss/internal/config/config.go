// Package config provides application configuration from environment variables
package config

import (
	"os"
	"strconv"
)

// AppConfig holds all application configuration
type AppConfig struct {
	DatabaseURL   string
	WhereIssURL   string
	NasaAPIURL    string
	NasaAPIKey    string
	FetchInterval FetchIntervals
}

// FetchIntervals defines background task intervals in seconds
type FetchIntervals struct {
	OsdrSeconds   int
	IssSeconds    int
	ApodSeconds   int
	NeoSeconds    int
	DonkiSeconds  int
	SpacexSeconds int
}

// LoadConfig loads configuration from environment variables
func LoadConfig() *AppConfig {
	return &AppConfig{
		DatabaseURL: getEnv("DATABASE_URL", "postgres://monouser:monopass@db:5432/monolith"),
		WhereIssURL: getEnv("WHERE_ISS_URL", "https://api.wheretheiss.at/v1/satellites/25544"),
		NasaAPIURL:  getEnv("NASA_API_URL", "https://visualization.osdr.nasa.gov/biodata/api/v2/datasets/?format=json"),
		NasaAPIKey:  getEnv("NASA_API_KEY", "DEMO_KEY"),
		FetchInterval: FetchIntervals{
			OsdrSeconds:   getEnvInt("FETCH_EVERY_SECONDS", 600),
			IssSeconds:    getEnvInt("ISS_EVERY_SECONDS", 120),
			ApodSeconds:   getEnvInt("APOD_EVERY_SECONDS", 43200),
			NeoSeconds:    getEnvInt("NEO_EVERY_SECONDS", 7200),
			DonkiSeconds:  getEnvInt("DONKI_EVERY_SECONDS", 3600),
			SpacexSeconds: getEnvInt("SPACEX_EVERY_SECONDS", 3600),
		},
	}
}

func getEnv(key, defaultVal string) string {
	if val := os.Getenv(key); val != "" {
		return val
	}
	return defaultVal
}

func getEnvInt(key string, defaultVal int) int {
	if val := os.Getenv(key); val != "" {
		if n, err := strconv.Atoi(val); err == nil {
			return n
		}
	}
	return defaultVal
}
