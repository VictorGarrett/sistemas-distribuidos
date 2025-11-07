package main

import (
	"context"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"
	"gateway/internal/handlers"
	//"gateway/internal/rabbitmq"
	"gateway/internal/services"
)

func main() {
	// Configuration (can be overridden with environment variables)
	port := getEnv("PORT", "9090")
	//rabbitURL := getEnv("RABBITMQ_URL", "amqp://guest:guest@localhost:5672/")
	auctionServiceURL := getEnv("AUCTION_SERVICE_URL", "http://localhost:8080")

	// Initialize dependencies
	auctionSvc := services.NewAuctionService(auctionServiceURL)
	handler := handlers.NewAuctionHandler(auctionSvc)

	// Start RabbitMQ consumer
	//go func() {
	//	if err := rabbitmq.Consume(rabbitURL, "events"); err != nil {
	//		log.Printf("RabbitMQ consumer error: %v", err)
	//	}
	//}()

	// Setup HTTP server
	mux := http.NewServeMux()
	mux.HandleFunc("/api/v1/auctions", handler.HandleAuctions)

	server := &http.Server{
		Addr:    ":" + port,
		Handler: mux,
	}

	// Graceful shutdown
	go func() {
		log.Printf("Server started on port %s", port)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("Server error: %v", err)
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit
	log.Println("Shutting down server...")

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if err := server.Shutdown(ctx); err != nil {
		log.Fatalf("Server forced to shutdown: %v", err)
	}
	log.Println("Server stopped gracefully")
}

func getEnv(key, fallback string) string {
	if val := os.Getenv(key); val != "" {
		return val
	}
	return fallback
}
