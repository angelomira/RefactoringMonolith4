// Package handlers provides HTTP request handlers
package handlers

import (
	"net/http"
	"strconv"
	"strings"
	"time"

	"go-iss/internal/domain"
	"go-iss/internal/services"

	"github.com/gin-gonic/gin"
)

// Handler holds all service dependencies
type Handler struct {
	IssService   *services.IssService
	OsdrService  *services.OsdrService
	SpaceService *services.SpaceService
}

// NewHandler creates a new handler with services
func NewHandler(iss *services.IssService, osdr *services.OsdrService, space *services.SpaceService) *Handler {
	return &Handler{
		IssService:   iss,
		OsdrService:  osdr,
		SpaceService: space,
	}
}

// Health handles health check requests
func (h *Handler) Health(c *gin.Context) {
	c.JSON(http.StatusOK, domain.Health{
		Status: "ok",
		Now:    time.Now().UTC(),
	})
}

// GetLastISS handles requests for the latest ISS position
func (h *Handler) GetLastISS(c *gin.Context) {
	log, err := h.IssService.GetLatest(c.Request.Context())
	if err != nil {
		c.JSON(http.StatusOK, domain.ErrorResponse("INTERNAL", err.Error()))
		return
	}

	if log == nil {
		c.JSON(http.StatusOK, domain.SuccessResponse(map[string]interface{}{
			"message": "no data",
		}))
		return
	}

	c.JSON(http.StatusOK, domain.SuccessResponse(map[string]interface{}{
		"id":         log.ID,
		"fetched_at": log.FetchedAt,
		"source_url": log.SourceURL,
		"payload":    log.Payload,
	}))
}

// TriggerISSFetch handles requests to trigger ISS fetch
func (h *Handler) TriggerISSFetch(c *gin.Context) {
	if err := h.IssService.FetchAndStore(c.Request.Context()); err != nil {
		c.JSON(http.StatusOK, domain.ErrorResponse("FETCH_ERROR", err.Error()))
		return
	}
	h.GetLastISS(c)
}

// GetISSTrend handles requests for ISS movement trend
func (h *Handler) GetISSTrend(c *gin.Context) {
	trend, err := h.IssService.CalculateTrend(c.Request.Context())
	if err != nil {
		c.JSON(http.StatusOK, domain.ErrorResponse("INTERNAL", err.Error()))
		return
	}
	c.JSON(http.StatusOK, domain.SuccessResponse(trend))
}

// SyncOSDR handles OSDR sync requests
func (h *Handler) SyncOSDR(c *gin.Context) {
	written, err := h.OsdrService.Sync(c.Request.Context())
	if err != nil {
		c.JSON(http.StatusOK, domain.ErrorResponse("SYNC_ERROR", err.Error()))
		return
	}
	c.JSON(http.StatusOK, domain.SuccessResponse(map[string]interface{}{
		"written": written,
	}))
}

// ListOSDR handles OSDR list requests
func (h *Handler) ListOSDR(c *gin.Context) {
	limitStr := c.DefaultQuery("limit", "20")
	limit, _ := strconv.Atoi(limitStr)
	if limit <= 0 {
		limit = 20
	}

	items, err := h.OsdrService.List(c.Request.Context(), limit)
	if err != nil {
		c.JSON(http.StatusOK, domain.ErrorResponse("INTERNAL", err.Error()))
		return
	}
	c.JSON(http.StatusOK, domain.SuccessResponse(map[string]interface{}{
		"items": items,
	}))
}

// GetSpaceLatest handles requests for latest space data
func (h *Handler) GetSpaceLatest(c *gin.Context) {
	source := c.Param("src")
	cache, err := h.SpaceService.GetLatest(c.Request.Context(), source)
	if err != nil {
		c.JSON(http.StatusOK, domain.ErrorResponse("INTERNAL", err.Error()))
		return
	}

	if cache == nil {
		c.JSON(http.StatusOK, domain.SuccessResponse(map[string]interface{}{
			"source":  source,
			"message": "no data",
		}))
		return
	}

	c.JSON(http.StatusOK, domain.SuccessResponse(map[string]interface{}{
		"source":     cache.Source,
		"fetched_at": cache.FetchedAt,
		"payload":    cache.Payload,
	}))
}

// RefreshSpace handles space data refresh requests
func (h *Handler) RefreshSpace(c *gin.Context) {
	sourcesStr := c.DefaultQuery("src", "apod,neo,flr,cme,spacex")
	sources := strings.Split(sourcesStr, ",")
	for i := range sources {
		sources[i] = strings.TrimSpace(sources[i])
	}

	refreshed := h.SpaceService.Refresh(c.Request.Context(), sources)
	c.JSON(http.StatusOK, domain.SuccessResponse(map[string]interface{}{
		"refreshed": refreshed,
	}))
}

// GetSpaceSummary handles space summary requests
func (h *Handler) GetSpaceSummary(c *gin.Context) {
	summary, err := h.SpaceService.GetSummary(c.Request.Context())
	if err != nil {
		c.JSON(http.StatusOK, domain.ErrorResponse("INTERNAL", err.Error()))
		return
	}
	c.JSON(http.StatusOK, domain.SuccessResponse(summary))
}

// SetupRoutes configures all routes
func SetupRoutes(r *gin.Engine, h *Handler) {
	// Health check
	r.GET("/health", h.Health)

	// ISS endpoints
	r.GET("/last", h.GetLastISS)
	r.GET("/fetch", h.TriggerISSFetch)
	r.GET("/iss/trend", h.GetISSTrend)

	// OSDR endpoints
	r.GET("/osdr/sync", h.SyncOSDR)
	r.GET("/osdr/list", h.ListOSDR)

	// Space cache endpoints
	r.GET("/space/:src/latest", h.GetSpaceLatest)
	r.GET("/space/refresh", h.RefreshSpace)
	r.GET("/space/summary", h.GetSpaceSummary)
}
