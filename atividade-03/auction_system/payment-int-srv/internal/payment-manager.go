package internal

import (
	"errors"
	"fmt"
	"payment-srv/internal/models"
	"sync"
	"time"

	"github.com/google/uuid"
)

type PaymentManager struct {
	Url             string
	mutex           sync.Mutex
	pendingPayments map[uuid.UUID]models.Payment
	paidPayments    map[uuid.UUID]models.Payment

	//channels
	updatesChannel chan models.PaymentUpdate
}

func NewPaymentManager(
	url string,
	updatesChannel chan models.PaymentUpdate,
) *PaymentManager {
	return &PaymentManager{
		Url:             url,
		pendingPayments: make(map[uuid.UUID]models.Payment),
		paidPayments:    make(map[uuid.UUID]models.Payment),
		updatesChannel:  updatesChannel,
	}
}

func (pm *PaymentManager) CreateNewPayment(req *models.NewAuctionWinner, id uuid.UUID) *models.Payment {
	payment := models.Payment{
		ID:        id,
		ClientID:  req.ClientID,
		AuctionID: req.AuctionID,
		Amount:    req.Amount,
		CreatedAt: time.Now().Unix(),
		PaidAt:    -1,
	}

	pm.mutex.Lock()
	pm.pendingPayments[payment.ID] = payment
	pm.mutex.Unlock()

	return &payment
}

func (pm *PaymentManager) SetPaid(id uuid.UUID) error {
	fmt.Printf("Set payment\n")
	pm.mutex.Lock()
	defer pm.mutex.Unlock()
	if _, ok := pm.pendingPayments[id]; !ok {
		return errors.New("payment not found")
	}

	payment := pm.pendingPayments[id]
	delete(pm.pendingPayments, id)

	pm.paidPayments[payment.ID] = payment

	pm.updatesChannel <- models.PaymentUpdate{
		PaymentID: id,
		Status:    "PAID",
	}

	return nil
}

func (pm *PaymentManager) GetPayment(id uuid.UUID) *models.Payment {
	Payment, ok := pm.paidPayments[id]
	if ok {
		return &Payment
	}

	Payment, ok = pm.pendingPayments[id]
	if ok {
		return &Payment
	}

	return &Payment
}
