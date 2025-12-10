package services

import (

	//"encoding/json"
	"context"
	"fmt"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	bid_srv "gateway/proto-go/bid_srv"
)

type BidService struct {
	client bid_srv.BidServiceClient
	conn   *grpc.ClientConn
}

func NewBidService(baseURL string) *BidService {
	conn, err := grpc.Dial(
		baseURL,
		grpc.WithTransportCredentials(insecure.NewCredentials()), // remove if using TLS
	)
	if err != nil {
		fmt.Errorf("failed to connect to gRPC server: %w", err)
		return nil
	}

	client := bid_srv.NewBidServiceClient(conn)

	return &BidService{
		client: client,
		conn:   conn,
	}
}

func (s *BidService) CreateBid(auction_id uint32, client_id uint32, value float64, signature string, public_key string, valid bool) (*bid_srv.CreateBidResponse, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	fmt.Printf("gRPC sending: auction=%d client=%d val=%f\n",
		auction_id, client_id, value)

	req := &bid_srv.CreateBidRequest{
		AuctionId: auction_id,
		ClientId:  client_id,
		Value:     value,
		Signature: signature,
		PublicKey: public_key,
		Valid:     valid,
	}

	fmt.Println("gRPC: CreateBid")
	fmt.Printf("Request: %+v\n", req)

	resp, err := s.client.CreateBid(ctx, req)
	if err != nil {
		return nil, fmt.Errorf("grpc CreateBid failed: %w", err)
	}

	return resp, nil
}
