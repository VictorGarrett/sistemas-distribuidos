package services

import (
	"context"
	"fmt"
	pis "payment-ext-srv/proto-go/payment-int-srv"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

type PaymentInternalService struct {
	client pis.PaymentInternalServiceClient
	conn   *grpc.ClientConn
}

func NewPaymentInternalService(url string) (*PaymentInternalService, error) {
	fmt.Println("Starting Payment internal service client")
	conn, err := grpc.NewClient(
		url,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		return nil, err
	}

	client := pis.NewPaymentInternalServiceClient(conn)

	return &PaymentInternalService{
		client: client,
		conn:   conn,
	}, nil
}

func (s *PaymentInternalService) UpdatePayment(transactionID string) error {
	body := pis.UpdatePaymentRequest{
		PaymentId: transactionID,
	}

	_, err := s.client.UpdatePayment(
		context.Background(),
		&body,
	)

	return err

}
