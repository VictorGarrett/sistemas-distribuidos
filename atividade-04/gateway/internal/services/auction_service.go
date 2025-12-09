package services

import (
    "context"
    "fmt"
    "time"

    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials/insecure"

)

import auction_srv "gateway/proto-go/auction_srv"

type AuctionService struct {
    client auction_srv.AuctionServiceClient
    conn   *grpc.ClientConn
}

func NewAuctionService(baseURL string) (*AuctionService, error) {
    // baseURL should look like "localhost:50051"
    conn, err := grpc.Dial(
        baseURL,
        grpc.WithTransportCredentials(insecure.NewCredentials()), // remove if using TLS
    )
    if err != nil {
        return nil, fmt.Errorf("failed to connect to gRPC server: %w", err)
    }

    client := auction_srv.NewAuctionServiceClient(conn)

    return &AuctionService{
        client: client,
        conn:   conn,
    }, nil
}

func (s *AuctionService) Close() {
    if s.conn != nil {
        s.conn.Close()
    }
}

func (s *AuctionService) GetActiveAuctions() ([]*auction_srv.Auction, error) {
    ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
    defer cancel()

    fmt.Println("gRPC: GetActiveAuctions")

    resp, err := s.client.GetActiveAuctions(ctx, &auction_srv.GetActiveAuctionsRequest{})
    if err != nil {
        return nil, fmt.Errorf("grpc GetActiveAuctions failed: %w", err)
    }

	fmt.Println("result", resp.Auctions)

    return resp.Auctions, nil
}

func (s *AuctionService) CreateAuction(itemName string, start, end uint64) (*auction_srv.Auction, error) {
    ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
    defer cancel()

	fmt.Printf("gRPC sending: item=%s start=%d end=%d\n",
    itemName, start, end)

    req := &auction_srv.CreateAuctionRequest{
        ItemName:        itemName,
        StartTimestamp:  start,
        EndTimestamp:    end,
    }

    fmt.Println("gRPC: CreateAuction")
    fmt.Printf("Request: %+v\n", req)

    resp, err := s.client.CreateAuction(ctx, req)
    if err != nil {
        return nil, fmt.Errorf("grpc CreateAuction failed: %w", err)
    }

    return resp, nil
}
