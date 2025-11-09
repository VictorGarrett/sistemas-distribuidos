package models

import "github.com/google/uuid"

type NewAuctionWinner struct {
	ClientID  int64   `json:"client_id"`
	AuctionID string  `json:"auction"`
	Amount    float32 `json:"amount"`
}

type Payment struct {
	ID        uuid.UUID
	ClientID  int64
	AuctionID string
	Amount    float32
	CreatedAt int64
	PaidAt    int64
}
