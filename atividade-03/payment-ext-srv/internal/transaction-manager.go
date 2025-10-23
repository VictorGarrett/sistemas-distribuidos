package internal

import (
	"errors"
	"payment-ext-srv/internal/models"
	"sync"
	"time"

	"github.com/google/uuid"
)

type TransactionManager struct {
	Url                 string
	mutex               sync.Mutex
	pendingTransactions map[uuid.UUID]models.Transaction
	paidTransactions    map[uuid.UUID]models.Transaction
	expiredTransactions map[uuid.UUID]models.Transaction
}

func NewTransactionManager(url string) *TransactionManager {
	return &TransactionManager{
		Url:                 url,
		pendingTransactions: make(map[uuid.UUID]models.Transaction),
		paidTransactions:    make(map[uuid.UUID]models.Transaction),
		expiredTransactions: make(map[uuid.UUID]models.Transaction),
	}
}

func (pm *TransactionManager) CreateNewTransaction(req *models.NewTransactionRequest) *models.NewTransactionResponse {
	paymentID := uuid.New()

	payment := models.Transaction{
		ID:        paymentID,
		Amount:    req.Amount,
		CreatedAt: time.Now().Unix(),
		PaidAt:    -1,
		Callback:  req.Callback,
		Status:    string(models.Pending),
	}

	pm.mutex.Lock()
	pm.pendingTransactions[payment.ID] = payment
	pm.mutex.Unlock()

	return payment.ToNewTransactionResponse(pm.Url + "/pay/" + payment.ID.String())
}

func (pm *TransactionManager) SetPaid(id uuid.UUID) error {
	pm.mutex.Lock()
	defer pm.mutex.Unlock()
	if _, ok := pm.pendingTransactions[id]; !ok {
		return errors.New("payment not found")
	}

	payment := pm.pendingTransactions[id]
	delete(pm.pendingTransactions, id)
	payment.Status = string(models.Paid)

	pm.paidTransactions[payment.ID] = payment

	return nil
}

func (pm *TransactionManager) GetTransaction(id uuid.UUID) *models.Transaction {
	transaction, ok := pm.paidTransactions[id]
	if ok {
		return &transaction
	}

	transaction, ok = pm.pendingTransactions[id]
	if ok {
		return &transaction
	}

	transaction = pm.expiredTransactions[id]
	return &transaction
}
