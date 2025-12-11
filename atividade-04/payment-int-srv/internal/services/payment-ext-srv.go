package services

import (
	"context"
	pes "payment-srv/proto-go/payment-ext-srv"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

type PaymentExternalService struct {
	client pes.PaymentExternalServiceClient
	conn   *grpc.ClientConn
}

func NewPaymentExternalService(url string) (*PaymentExternalService, error) {
	conn, err := grpc.NewClient(
		url,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)

	if err != nil {
		return nil, err
	}

	client := pes.NewPaymentExternalServiceClient(conn)

	return &PaymentExternalService{
			client: client,
			conn:   conn,
		},
		nil

}

func (s *PaymentExternalService) CreateTransaction(amount float32) (*pes.CreateTransactionResponse, error) {
	req := &pes.CreateTransactionRequest{
		Amount:   amount,
		Callback: "",
	}

	res, err := s.client.CreateTransaction(context.Background(), req)
	if err != nil {
		return nil, err
	}

	return res, nil
}
