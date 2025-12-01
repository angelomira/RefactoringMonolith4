// Main entry point for the Go ISS service
package main

import (
	"context"
	"log"
	"time"

	"go-iss/internal/clients"
	"go-iss/internal/config"
	"go-iss/internal/handlers"
	"go-iss/internal/repo"
	"go-iss/internal/services"

	"github.com/gin-gonic/gin"
	"github.com/jackc/pgx/v5/pgxpool"
)

func main() {
	// Load configuration
	cfg := config.LoadConfig()
	log.Println("Configuration loaded successfully")

	// Initialize database connection pool
	ctx := context.Background()
	pool, err := pgxpool.New(ctx, cfg.DatabaseURL)
	if err != nil {
		log.Fatalf("Unable to connect to database: %v", err)
	}
	defer pool.Close()
	log.Println("Database connection pool established")

	// Initialize database schema
	if err := repo.InitDB(ctx, pool); err != nil {
		log.Fatalf("Failed to initialize database: %v", err)
	}
	log.Println("Database schema initialized")

	// Initialize repositories
	issRepo := repo.NewIssRepo(pool)
	osdrRepo := repo.NewOsdrRepo(pool)
	cacheRepo := repo.NewCacheRepo(pool)

	// Initialize clients
	issClient := clients.NewIssClient(cfg.WhereIssURL)
	osdrClient := clients.NewOsdrClient(cfg.NasaAPIURL)
	nasaClient := clients.NewNasaClient(cfg.NasaAPIKey)
	spacexClient := clients.NewSpaceXClient()

	// Initialize services
	issService := services.NewIssService(issRepo, issClient)
	osdrService := services.NewOsdrService(osdrRepo, osdrClient)
	spaceService := services.NewSpaceService(cacheRepo, issRepo, osdrRepo, nasaClient, spacexClient)

	// Start background tasks
	startBackgroundTasks(cfg, issService, osdrService, spaceService)

	// Setup HTTP server
	gin.SetMode(gin.ReleaseMode)
	r := gin.New()
	r.Use(gin.Recovery())
	r.Use(gin.Logger())

	// Setup routes
	handler := handlers.NewHandler(issService, osdrService, spaceService)
	handlers.SetupRoutes(r, handler)

	// Start server
	log.Println("go_iss service listening on 0.0.0.0:3000")
	if err := r.Run(":3000"); err != nil {
		log.Fatalf("Failed to start server: %v", err)
	}
}

func startBackgroundTasks(cfg *config.AppConfig, iss *services.IssService, osdr *services.OsdrService, space *services.SpaceService) {
	intervals := cfg.FetchInterval

	// OSDR sync task
	go func() {
		log.Printf("Starting OSDR background task (interval: %ds)", intervals.OsdrSeconds)
		ticker := time.NewTicker(time.Duration(intervals.OsdrSeconds) * time.Second)
		defer ticker.Stop()
		for {
			ctx := context.Background()
			if _, err := osdr.Sync(ctx); err != nil {
				log.Printf("OSDR sync error: %v", err)
			}
			<-ticker.C
		}
	}()

	// ISS tracking task
	go func() {
		log.Printf("Starting ISS tracking task (interval: %ds)", intervals.IssSeconds)
		ticker := time.NewTicker(time.Duration(intervals.IssSeconds) * time.Second)
		defer ticker.Stop()
		for {
			ctx := context.Background()
			if err := iss.FetchAndStore(ctx); err != nil {
				log.Printf("ISS fetch error: %v", err)
			}
			<-ticker.C
		}
	}()

	// APOD task
	go func() {
		log.Printf("Starting APOD background task (interval: %ds)", intervals.ApodSeconds)
		ticker := time.NewTicker(time.Duration(intervals.ApodSeconds) * time.Second)
		defer ticker.Stop()
		for {
			ctx := context.Background()
			if err := space.FetchApod(ctx); err != nil {
				log.Printf("APOD fetch error: %v", err)
			}
			<-ticker.C
		}
	}()

	// NEO feed task
	go func() {
		log.Printf("Starting NEO feed task (interval: %ds)", intervals.NeoSeconds)
		ticker := time.NewTicker(time.Duration(intervals.NeoSeconds) * time.Second)
		defer ticker.Stop()
		for {
			ctx := context.Background()
			if err := space.FetchNeo(ctx); err != nil {
				log.Printf("NEO fetch error: %v", err)
			}
			<-ticker.C
		}
	}()

	// DONKI task
	go func() {
		log.Printf("Starting DONKI background task (interval: %ds)", intervals.DonkiSeconds)
		ticker := time.NewTicker(time.Duration(intervals.DonkiSeconds) * time.Second)
		defer ticker.Stop()
		for {
			ctx := context.Background()
			if err := space.FetchFlr(ctx); err != nil {
				log.Printf("DONKI FLR fetch error: %v", err)
			}
			if err := space.FetchCme(ctx); err != nil {
				log.Printf("DONKI CME fetch error: %v", err)
			}
			<-ticker.C
		}
	}()

	// SpaceX launches task
	go func() {
		log.Printf("Starting SpaceX launches task (interval: %ds)", intervals.SpacexSeconds)
		ticker := time.NewTicker(time.Duration(intervals.SpacexSeconds) * time.Second)
		defer ticker.Stop()
		for {
			ctx := context.Background()
			if err := space.FetchSpacex(ctx); err != nil {
				log.Printf("SpaceX fetch error: %v", err)
			}
			<-ticker.C
		}
	}()

	log.Println("All background tasks started successfully")
}
