package api

import (
	"context"
	"errors"
	"payment-ext-srv/internal"
	"payment-ext-srv/internal/models"
	pes "payment-ext-srv/proto-go/payment-ext-srv"
)

type PaymentExternalServiceServer struct {
	pes.UnimplementedPaymentExternalServiceServer
	tm *internal.TransactionManager
}

func NewGRPCService(
	tm *internal.TransactionManager,
) *PaymentExternalServiceServer {
	return &PaymentExternalServiceServer{
		tm: tm,
	}
}

func (s *PaymentExternalServiceServer) CreateTransaction(
	ctx context.Context,
	req *pes.CreateTransactionRequest,
) (*pes.CreateTransactionResponse, error) {
	if req.Amount <= 0.0 {
		return nil, errors.New("AMOUNT_MUST_BE_POSITIVE")
	}

	newTransaction := models.NewTransactionRequest{
		Amount:   req.Amount,
		Callback: "",
	}

	res := s.tm.CreateNewTransaction(&newTransaction)

	return &pes.CreateTransactionResponse{
		TransactionId: res.ID,
		Amount:        res.Amount,
		PaymentLink:   res.Link,
	}, nil
}

func (s *PaymentExternalServiceServer) GetTransactions(
	ctx context.Context,
	req *pes.GetTransactionsRequest,
) (*pes.GetTransactionsResponse, error) {
	return nil, errors.New("NOT_IMPLEMENTED")
}

func (s *PaymentExternalServiceServer) PayTransaction(
	ctx context.Context,
	req *pes.PayTransactionRequest,
) (*pes.PayTransactionResponse, error) {
	return nil, errors.New("NOT_IMPLEMENTED")
}
