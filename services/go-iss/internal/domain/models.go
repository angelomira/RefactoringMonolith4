// Package domain provides domain models for the application
package domain

import (
	"encoding/json"
	"time"
)

// IssLog represents an ISS position fetch log entry
type IssLog struct {
	ID        int64           `json:"id"`
	FetchedAt time.Time       `json:"fetched_at"`
	SourceURL string          `json:"source_url"`
	Payload   json.RawMessage `json:"payload"`
}

// OsdrItem represents a NASA OSDR dataset item
type OsdrItem struct {
	ID         int64           `json:"id"`
	DatasetID  *string         `json:"dataset_id"`
	Title      *string         `json:"title"`
	Status     *string         `json:"status"`
	UpdatedAt  *time.Time      `json:"updated_at"`
	InsertedAt time.Time       `json:"inserted_at"`
	Raw        json.RawMessage `json:"raw"`
}

// SpaceCache represents cached space data
type SpaceCache struct {
	ID        int64           `json:"id"`
	Source    string          `json:"source"`
	FetchedAt time.Time       `json:"fetched_at"`
	Payload   json.RawMessage `json:"payload"`
}

// IssTrend represents ISS movement analysis
type IssTrend struct {
	Movement    bool       `json:"movement"`
	DeltaKm     float64    `json:"delta_km"`
	DtSec       float64    `json:"dt_sec"`
	VelocityKmh *float64   `json:"velocity_kmh"`
	FromTime    *time.Time `json:"from_time"`
	ToTime      *time.Time `json:"to_time"`
	FromLat     *float64   `json:"from_lat"`
	FromLon     *float64   `json:"from_lon"`
	ToLat       *float64   `json:"to_lat"`
	ToLon       *float64   `json:"to_lon"`
}

// Health represents health check response
type Health struct {
	Status string    `json:"status"`
	Now    time.Time `json:"now"`
}

// SpaceSummary represents aggregated space data
type SpaceSummary struct {
	Apod      interface{} `json:"apod"`
	Neo       interface{} `json:"neo"`
	Flr       interface{} `json:"flr"`
	Cme       interface{} `json:"cme"`
	Spacex    interface{} `json:"spacex"`
	Iss       interface{} `json:"iss"`
	OsdrCount int64       `json:"osdr_count"`
}

// ApiResponse wraps API responses
type ApiResponse struct {
	Ok    bool        `json:"ok"`
	Data  interface{} `json:"data,omitempty"`
	Error *ApiError   `json:"error,omitempty"`
}

// ApiError represents an error response
type ApiError struct {
	Code    string `json:"code"`
	Message string `json:"message"`
}

// SuccessResponse creates a successful response
func SuccessResponse(data interface{}) ApiResponse {
	return ApiResponse{Ok: true, Data: data}
}

// ErrorResponse creates an error response
func ErrorResponse(code, message string) ApiResponse {
	return ApiResponse{Ok: false, Error: &ApiError{Code: code, Message: message}}
}
