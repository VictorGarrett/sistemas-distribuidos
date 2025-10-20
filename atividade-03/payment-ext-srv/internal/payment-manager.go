package internal

import (
	"errors"
	"payment-ext-srv/internal/models"
	"sync"
	"time"

	"github.com/google/uuid"
)

type PaymentManager struct {
	Url             string
	mutex           sync.Mutex
	pendingPayments map[uuid.UUID]models.Payment
	paidPayments    map[uuid.UUID]models.Payment
	expiredPayments map[uuid.UUID]models.Payment
}

func NewPaymentManager(url string) *PaymentManager {
	return &PaymentManager{
		Url:             url,
		pendingPayments: make(map[uuid.UUID]models.Payment),
		paidPayments:    make(map[uuid.UUID]models.Payment),
		expiredPayments: make(map[uuid.UUID]models.Payment),
	}
}

func (pm *PaymentManager) CreateNewPayment(req *models.NewPaymentRequest) *models.NewPaymentResponse {
	paymentID := uuid.New()

	payment := models.Payment{
		ID:        paymentID,
		Amount:    req.Amount,
		CreatedAt: time.Now().Unix(),
		PaidAt:    -1,
		Callback:  req.Callback,
		Status:    string(models.Pending),
	}

	pm.mutex.Lock()
	pm.pendingPayments[payment.ID] = payment
	pm.mutex.Unlock()

	return payment.ToNewPaymentResponse(pm.Url + "/pay/" + payment.ID.String())
}

func (pm *PaymentManager) SetPaid(id uuid.UUID) error {
	pm.mutex.Lock()
	defer pm.mutex.Unlock()
	if _, ok := pm.pendingPayments[id]; ok == false {
		return errors.New("Payment not found")
	}

	payment := pm.pendingPayments[id]
	delete(pm.pendingPayments, id)
	payment.Status = string(models.Paid)

	pm.paidPayments[payment.ID] = payment

	return nil
}
