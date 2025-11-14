package services

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
)

type AuctionService struct {
	baseURL string
	client  *http.Client
}

func NewAuctionService(baseURL string) *AuctionService {
	return &AuctionService{
		baseURL: baseURL,
		client:  &http.Client{},
	}
}

func (s *AuctionService) GetActiveAuctions() ([]map[string]interface{}, error) {
	resp, err := s.client.Get(fmt.Sprintf("%s/auctions", s.baseURL))
	fmt.Printf("GET %s/auctions\n", s.baseURL)
	if err != nil {
		return nil, fmt.Errorf("failed to get active auctions: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("auction service error: %s", string(body))
	}

	var auctions []map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&auctions); err != nil {
		return nil, fmt.Errorf("failed to decode auctions: %w", err)
	}
	return auctions, nil
}

func (s *AuctionService) CreateAuction(payload []byte) ([]byte, error) {
	resp, err := s.client.Post(fmt.Sprintf("%s/auctions", s.baseURL), "application/json", bytes.NewBuffer(payload))
	fmt.Printf("POST %s/auctions\n", s.baseURL)
	fmt.Printf("Request Body: %s\n", string(payload))
	if err != nil {
		return nil, fmt.Errorf("failed to create auction: %w", err)
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	if resp.StatusCode != http.StatusCreated && resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("auction service returned %d: %s", resp.StatusCode, string(body))
	}

	return body, nil
}
