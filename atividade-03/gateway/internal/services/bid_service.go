package services

import (
	"bytes"
	//"encoding/json"
	"fmt"
	"io"
	"net/http"
)

type BidService struct {
	baseURL string
	client  *http.Client
}

func NewBidService(baseURL string) *BidService {
	return &BidService{
		baseURL: baseURL,
		client:  &http.Client{},
	}
}

func (s *BidService) CreateBid(payload []byte) ([]byte, error) {
	resp, err := s.client.Post(fmt.Sprintf("%s/bid", s.baseURL), "application/json", bytes.NewBuffer(payload))
	fmt.Printf("POST %s/bid\n", s.baseURL)
	fmt.Printf("Request Body: %s\n", string(payload))
	if err != nil {
		return nil, fmt.Errorf("failed to create Bid: %w", err)
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	if resp.StatusCode != http.StatusCreated && resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("Bid service returned %d: %s", resp.StatusCode, string(body))
	}

	return body, nil
}
