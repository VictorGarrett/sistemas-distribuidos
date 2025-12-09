package main

import (
	"context"
	"flag"
	"gateway/internal/handlers"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	//"gateway/internal/rabbitmq"
	"gateway/internal/services"

	"github.com/joho/godotenv"
)

func withCORS(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*") // or "http://localhost:5173"
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type")

		// Handle preflight requests
		if r.Method == http.MethodOptions {
			w.WriteHeader(http.StatusOK)
			return
		}

		next.ServeHTTP(w, r)
	})
}

func main() {
	dockerize := flag.Bool("docker", false, "Use if running the program in a docker container")
	flag.Parse()

	if !*dockerize {
		err := godotenv.Load(".env")
		if err != nil {
			log.Panicf("Failed to load environment file!")
		}
	}
	baseURL := getEnv("BASE_URL", "localhost")
	port := getEnv("PORT", "8080")

	auctionServiceURL := getEnv("AUCTION_SERVICE_URL", "http://localhost:8080")
	bidServiceURL := getEnv("BID_SERVICE_URL", "http://localhost:8081")

	// Initialize dependencies
	auctionSvc, err := services.NewAuctionService(auctionServiceURL)
	if err != nil {
		log.Panicf("Failed to load grpc services!")
	}


	handler := handlers.NewAuctionHandler(auctionSvc)

	bidSvc := services.NewBidService(bidServiceURL)
	bidHandler := handlers.NewBidHandler(bidSvc)

	sseBroker := handlers.NewSseBroker()
	// 2. Create the SSE handler, giving it the broker.
	sseHandler := handlers.NewSseHandler(sseBroker)

	// Start RabbitMQ consumer
	//go func() {
	//	if err := rabbitmq.Consume(rabbitURL, "events"); err != nil {
	//		log.Printf("RabbitMQ consumer error: %v", err)
	//	}
	//}()

	// Setup HTTP server
	mux := http.NewServeMux()
	mux.HandleFunc("/api/v1/auctions", handler.HandleAuctions)
	mux.HandleFunc("/api/v1/bid", bidHandler.HandleBids)
	mux.HandleFunc("/api/v1/events", sseHandler.HandleEvents)
	mux.HandleFunc("/api/v1/subscribe", sseHandler.HandleSubscribe)
	mux.HandleFunc("/api/v1/unsubscribe", sseHandler.HandleUnsubscribe)

	server := &http.Server{
		Addr:    baseURL + ":" + port,
		Handler: withCORS(mux),
	}

	// Graceful shutdown
	go func() {
		log.Printf("Server started on %s", baseURL+":"+port)
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
