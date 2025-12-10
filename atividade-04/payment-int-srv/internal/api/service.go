package api

import (
	"context"
	"payment-srv/internal"
	pb "payment-srv/proto-go/payment-int-srv"

	"github.com/google/uuid"
)

type PaymentInternalServiceServer struct {
	pb.UnimplementedPaymentInternalServiceServer
	pm *internal.PaymentManager
}

func NewPaymentInternalServiceServer(pm *internal.PaymentManager) *PaymentInternalServiceServer {
	return &PaymentInternalServiceServer{
		pm: pm,
	}
}

func (s *PaymentInternalServiceServer) UpdatePayment(
	ctx context.Context,
	req *pb.UpdatePaymentRequest,
) (*pb.UpdatePaymentResponse, error) {
	paymentUUID, err := uuid.Parse(req.PaymentId)
	if err != nil {
		return nil, err
	}

	err = s.pm.SetPaid(paymentUUID)
	if err != nil {
		return nil, err
	}

	return &pb.UpdatePaymentResponse{}, nil
}
