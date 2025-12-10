package handlers

import (
	"context"
	"gateway/internal/services"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	pb "gateway/proto-go/gateway"
)

type BidHandler struct {
	pb.UnimplementedBidServiceServer
	service *services.BidService
}

func NewBidHandler(svc *services.BidService) *BidHandler {
	return &BidHandler{service: svc}
}

func (h *BidHandler) CreateBid(ctx context.Context, req *pb.CreateBidRequest) (*pb.CreateBidResponse, error) {
	// Validate request
	if req.Value <= 0 {
		return nil, status.Error(codes.InvalidArgument, "value must be greater than 0")
	}
	if req.Signature == "" {
		return nil, status.Error(codes.InvalidArgument, "signature is required")
	}
	if req.PublicKey == "" {
		return nil, status.Error(codes.InvalidArgument, "public_key is required")
	}

	// Call service with all parameters
	_, err := h.service.CreateBid(
		req.AuctionId,
		req.ClientId,
		req.Value,
		req.Signature,
		req.PublicKey,
		req.Valid,
	)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to create bid: %v", err)
	}

	// Return simple success response
	return &pb.CreateBidResponse{
		Success: true,
	}, nil
}
