package models

type NewTransactionRequest struct {
	Amount   float32 `json:"amount"`
	Callback string  `json:"callback"`
}

type NewTransactionResponse struct {
	ID     string  `json:"id"`
	Amount float32 `json:"amount"`
	Link   string  `json:"link"`
}
