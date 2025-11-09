package models

type NewAuctionWinner struct {
	ClientID  int32   `json:"client_id"`
	AuctionID int32   `json:"auction"`
	Amount    float32 `json:"amount"`
}

type PaymentUpdatePublish struct {
	PaymentID string `json:"payment_id"`
	ClientID  int32  `json:"client_id"`
	AuctionID int32  `json:"auction_id"`
	Status    string `json:"status"`
}

type PaymentLinkPublish struct {
	PaymentID string  `json:"payment_id"`
	ClientID  int32   `json:"client_id"`
	AuctionID int32   `json:"auction_id"`
	Amount    float32 `json:"amount"`
	Link      string  `json:"payment_link"`
}
