// Package clients provides HTTP clients for external APIs
package clients

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"time"
)

// HTTPClient is a wrapper around http.Client with common configuration
type HTTPClient struct {
	client *http.Client
}

// NewHTTPClient creates a new HTTP client with timeout
func NewHTTPClient() *HTTPClient {
	return &HTTPClient{
		client: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

// Get performs a GET request and returns the response body
func (c *HTTPClient) Get(url string) ([]byte, error) {
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return nil, err
	}
	req.Header.Set("User-Agent", "go-iss-service/1.0")

	resp, err := c.client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	if resp.StatusCode >= 400 {
		return nil, fmt.Errorf("HTTP %d: %s", resp.StatusCode, string(body))
	}

	return body, nil
}

// IssClient fetches ISS position data
type IssClient struct {
	http    *HTTPClient
	baseURL string
}

// NewIssClient creates a new ISS client
func NewIssClient(baseURL string) *IssClient {
	return &IssClient{
		http:    NewHTTPClient(),
		baseURL: baseURL,
	}
}

// BaseURL returns the base URL
func (c *IssClient) BaseURL() string {
	return c.baseURL
}

// FetchPosition fetches current ISS position
func (c *IssClient) FetchPosition() (json.RawMessage, error) {
	body, err := c.http.Get(c.baseURL)
	if err != nil {
		return nil, err
	}
	return json.RawMessage(body), nil
}

// OsdrClient fetches NASA OSDR datasets
type OsdrClient struct {
	http    *HTTPClient
	baseURL string
}

// NewOsdrClient creates a new OSDR client
func NewOsdrClient(baseURL string) *OsdrClient {
	return &OsdrClient{
		http:    NewHTTPClient(),
		baseURL: baseURL,
	}
}

// FetchDatasets fetches OSDR datasets
func (c *OsdrClient) FetchDatasets() ([]json.RawMessage, error) {
	body, err := c.http.Get(c.baseURL)
	if err != nil {
		return nil, err
	}

	// Try to parse as array first
	var items []json.RawMessage
	if err := json.Unmarshal(body, &items); err == nil {
		return items, nil
	}

	// Try to parse as object with items/results field
	var wrapper map[string]json.RawMessage
	if err := json.Unmarshal(body, &wrapper); err == nil {
		if arr, ok := wrapper["items"]; ok {
			var items []json.RawMessage
			if err := json.Unmarshal(arr, &items); err == nil {
				return items, nil
			}
		}
		if arr, ok := wrapper["results"]; ok {
			var items []json.RawMessage
			if err := json.Unmarshal(arr, &items); err == nil {
				return items, nil
			}
		}
	}

	// Return single item
	return []json.RawMessage{body}, nil
}

// NasaClient fetches data from NASA APIs
type NasaClient struct {
	http   *HTTPClient
	apiKey string
}

// NewNasaClient creates a new NASA API client
func NewNasaClient(apiKey string) *NasaClient {
	return &NasaClient{
		http:   NewHTTPClient(),
		apiKey: apiKey,
	}
}

// FetchAPOD fetches Astronomy Picture of the Day
func (c *NasaClient) FetchAPOD() (json.RawMessage, error) {
	u, _ := url.Parse("https://api.nasa.gov/planetary/apod")
	q := u.Query()
	q.Set("thumbs", "true")
	if c.apiKey != "" {
		q.Set("api_key", c.apiKey)
	}
	u.RawQuery = q.Encode()

	body, err := c.http.Get(u.String())
	if err != nil {
		return nil, err
	}
	return json.RawMessage(body), nil
}

// FetchNeoFeed fetches Near Earth Objects
func (c *NasaClient) FetchNeoFeed() (json.RawMessage, error) {
	today := time.Now().Format("2006-01-02")
	start := time.Now().AddDate(0, 0, -2).Format("2006-01-02")

	u, _ := url.Parse("https://api.nasa.gov/neo/rest/v1/feed")
	q := u.Query()
	q.Set("start_date", start)
	q.Set("end_date", today)
	if c.apiKey != "" {
		q.Set("api_key", c.apiKey)
	}
	u.RawQuery = q.Encode()

	body, err := c.http.Get(u.String())
	if err != nil {
		return nil, err
	}
	return json.RawMessage(body), nil
}

// FetchDonkiFLR fetches DONKI Solar Flares
func (c *NasaClient) FetchDonkiFLR() (json.RawMessage, error) {
	return c.fetchDonki("FLR")
}

// FetchDonkiCME fetches DONKI Coronal Mass Ejections
func (c *NasaClient) FetchDonkiCME() (json.RawMessage, error) {
	return c.fetchDonki("CME")
}

func (c *NasaClient) fetchDonki(endpoint string) (json.RawMessage, error) {
	today := time.Now().Format("2006-01-02")
	start := time.Now().AddDate(0, 0, -5).Format("2006-01-02")

	u, _ := url.Parse(fmt.Sprintf("https://api.nasa.gov/DONKI/%s", endpoint))
	q := u.Query()
	q.Set("startDate", start)
	q.Set("endDate", today)
	if c.apiKey != "" {
		q.Set("api_key", c.apiKey)
	}
	u.RawQuery = q.Encode()

	body, err := c.http.Get(u.String())
	if err != nil {
		return nil, err
	}
	return json.RawMessage(body), nil
}

// SpaceXClient fetches SpaceX launch data
type SpaceXClient struct {
	http *HTTPClient
}

// NewSpaceXClient creates a new SpaceX client
func NewSpaceXClient() *SpaceXClient {
	return &SpaceXClient{
		http: NewHTTPClient(),
	}
}

// FetchNextLaunch fetches the next SpaceX launch
func (c *SpaceXClient) FetchNextLaunch() (json.RawMessage, error) {
	body, err := c.http.Get("https://api.spacexdata.com/v4/launches/next")
	if err != nil {
		return nil, err
	}
	return json.RawMessage(body), nil
}
