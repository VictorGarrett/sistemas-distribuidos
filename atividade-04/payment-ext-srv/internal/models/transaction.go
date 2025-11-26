package models

import "github.com/google/uuid"

type NewTransactionRequest struct {
	Amount   float32 `json:"amount"`
	Callback string  `json:"callback"`
}

type NewTransactionResponse struct {
	ID     string  `json:"id"`
	Amount float32 `json:"amount"`
	Link   string  `json:"link"`
}

type Transaction struct {
	ID        uuid.UUID
	Amount    float32
	CreatedAt int64
	PaidAt    int64
	Callback  string
	Status    string
}

type TransactionStatus string

const (
	Pending TransactionStatus = "PENDING"
	Expired TransactionStatus = "EXPIRED"
	Paid    TransactionStatus = "PAID"
)

func (p *Transaction) ToNewTransactionResponse(link string) *NewTransactionResponse {
	return &NewTransactionResponse{
		ID:     p.ID.String(),
		Amount: p.Amount,
		Link:   link,
	}
}
