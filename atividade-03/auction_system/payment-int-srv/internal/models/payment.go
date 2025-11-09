package models

import "github.com/google/uuid"

type Payment struct {
	ID        uuid.UUID
	ClientID  int32
	AuctionID int32
	Amount    float32
	CreatedAt int64
	PaidAt    int64
}

func (p *Payment) ToPaymentUpdatePublish(update *PaymentUpdate) *PaymentUpdatePublish {
	return &PaymentUpdatePublish{
		PaymentID: p.ID.String(),
		ClientID:  p.ClientID,
		AuctionID: p.AuctionID,
		Status:    update.Status,
	}
}

type PaymentUpdate struct {
	PaymentID uuid.UUID
	Status    string
}
