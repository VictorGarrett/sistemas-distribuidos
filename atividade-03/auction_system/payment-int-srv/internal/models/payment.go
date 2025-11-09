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

func (p *Payment) ToPaymentLinkPublish(pl *PaymentLink) *PaymentLinkPublish {
	return &PaymentLinkPublish{
		PaymentID: p.ID.String(),
		ClientID:  p.ClientID,
		AuctionID: p.AuctionID,
		Amount:    p.Amount,
		Link:      pl.Link,
	}
}

type PaymentUpdate struct {
	PaymentID uuid.UUID
	Status    string
}

type PaymentLink struct {
	PaymentID uuid.UUID
	Link      string
}
