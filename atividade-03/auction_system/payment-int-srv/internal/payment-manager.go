package internal

import (
	"errors"
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
}

func NewPaymentManager(url string) *PaymentManager {
	return &PaymentManager{
		Url:             url,
		pendingPayments: make(map[uuid.UUID]models.Payment),
		paidPayments:    make(map[uuid.UUID]models.Payment),
	}
}

func (pm *PaymentManager) CreateNewPayment(req *models.NewAuctionWinner) *models.Payment {
	paymentID := uuid.New()

	payment := models.Payment{
		ID:        paymentID,
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
	pm.mutex.Lock()
	defer pm.mutex.Unlock()
	if _, ok := pm.pendingPayments[id]; !ok {
		return errors.New("payment not found")
	}

	payment := pm.pendingPayments[id]
	delete(pm.pendingPayments, id)

	pm.paidPayments[payment.ID] = payment

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
