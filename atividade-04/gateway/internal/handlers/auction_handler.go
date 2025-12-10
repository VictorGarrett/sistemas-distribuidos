package handlers

import (
	"context"
	"gateway/internal/services"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	pb "gateway/proto-go/gateway"
)

type AuctionHandler struct {
	pb.UnimplementedAuctionServiceServer
	service *services.AuctionService
}

func NewAuctionHandler(svc *services.AuctionService) *AuctionHandler {
	return &AuctionHandler{service: svc}
}

func (h *AuctionHandler) GetActiveAuctions(ctx context.Context, req *pb.GetActiveAuctionsRequest) (*pb.GetActiveAuctionsResponse, error) {
	auctions, err := h.service.GetActiveAuctions()
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to get active auctions: %v", err)
	}

	// Convert service auctions to proto auctions
	pbAuctions := make([]*pb.Auction, 0, len(auctions))
	for _, auction := range auctions {
		pbAuctions = append(pbAuctions, &pb.Auction{
			Id:             auction.Id,
			Item:           auction.Item,
			StartTimestamp: auction.StartTimestamp,
			EndTimestamp:   auction.EndTimestamp,
			Status:         auction.Status,
		})
	}

	return &pb.GetActiveAuctionsResponse{
		Auctions: pbAuctions,
	}, nil
}

func (h *AuctionHandler) CreateAuction(ctx context.Context, req *pb.CreateAuctionRequest) (*pb.CreateAuctionResponse, error) {
	// Validate request
	if req.ItemName == "" {
		return nil, status.Error(codes.InvalidArgument, "item_name is required")
	}
	if req.StartTimestamp == 0 || req.EndTimestamp == 0 {
		return nil, status.Error(codes.InvalidArgument, "start_timestamp and end_timestamp are required")
	}
	if req.EndTimestamp <= req.StartTimestamp {
		return nil, status.Error(codes.InvalidArgument, "end_timestamp must be after start_timestamp")
	}

	// Call service
	_, err := h.service.CreateAuction(
		req.ItemName,
		req.StartTimestamp,
		req.EndTimestamp,
	)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "failed to create auction: %v", err)
	}

	// Convert service response to proto response
	return &pb.CreateAuctionResponse{
		Success: true,
	}, nil
}
