package models

import "github.com/google/uuid"

type NewPaymentRequest struct {
	Amount   float32 `json:"amount"`
	Callback string  `json:"callback"`
}

type NewPaymentResponse struct {
	ID     string  `json:"id"`
	Amount float32 `json:"amount"`
	Link   string  `json:"link"`
}

type Payment struct {
	ID        uuid.UUID
	Amount    float32
	CreatedAt int64
	PaidAt    int64
	Callback  string
	Status    string
}

type PaymentStatus string

const (
	Pending PaymentStatus = "PENDING"
	Expired PaymentStatus = "EXPIRED"
	Paid    PaymentStatus = "PAID"
)

func (p *Payment) ToNewPaymentResponse(link string) *NewPaymentResponse {
	return &NewPaymentResponse{
		ID:     p.ID.String(),
		Amount: p.Amount,
		Link:   link,
	}
}
