package main

import (
	"context"
	"flag"
	"gateway/internal/handlers"
	"gateway/internal/services"
	pb "gateway/proto-go/gateway"
	"log"
	"net"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/joho/godotenv"
	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"
)

func main() {
	dockerize := flag.Bool("docker", false, "Use if running the program in a docker container")
	flag.Parse()

	if !*dockerize {
		err := godotenv.Load(".env")
		if err != nil {
			log.Panicf("Failed to load environment file!")
		}
	}

	port := getEnv("PORT", "50051")
	auctionServiceURL := getEnv("AUCTION_SERVICE_URL", "localhost:8080")
	bidServiceURL := getEnv("BID_SERVICE_URL", "localhost:8081")

	// Initialize services
	auctionSvc, err := services.NewAuctionService(auctionServiceURL)
	if err != nil {
		log.Panicf("Failed to load auction service: %v", err)
	}

	bidSvc := services.NewBidService(bidServiceURL)

	// Create handlers
	auctionHandler := handlers.NewAuctionHandler(auctionSvc)
	bidHandler := handlers.NewBidHandler(bidSvc)

	// Create the broker for event streaming
	broker := handlers.NewGrpcBroker()

	// Create the gRPC server
	grpcServer := grpc.NewServer()

	// Register all services
	pb.RegisterAuctionServiceServer(grpcServer, auctionHandler)
	pb.RegisterBidServiceServer(grpcServer, bidHandler)

	// Create and register the event stream service
	eventStreamServer := handlers.NewEventStreamServer(broker)
	pb.RegisterEventStreamServer(grpcServer, eventStreamServer)

	// Register reflection service (useful for tools like grpcurl and debugging)
	reflection.Register(grpcServer)

	// Listen on TCP port
	listener, err := net.Listen("tcp", ":"+port)
	if err != nil {
		log.Fatalf("Failed to listen on port %s: %v", port, err)
	}

	log.Printf("gRPC server starting on :%s", port)

	// Start server in goroutine for graceful shutdown
	go func() {
		if err := grpcServer.Serve(listener); err != nil {
			log.Fatalf("Failed to serve: %v", err)
		}
	}()

	// Graceful shutdown
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit
	log.Println("Shutting down gRPC server...")

	// Graceful stop with timeout
	stopped := make(chan struct{})
	go func() {
		grpcServer.GracefulStop()
		close(stopped)
	}()

	// Force stop after timeout
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	select {
	case <-stopped:
		log.Println("gRPC server stopped gracefully")
	case <-ctx.Done():
		log.Println("Forcing gRPC server shutdown...")
		grpcServer.Stop()
	}
}

func getEnv(key, fallback string) string {
	if val := os.Getenv(key); val != "" {
		return val
	}
	return fallback
}
