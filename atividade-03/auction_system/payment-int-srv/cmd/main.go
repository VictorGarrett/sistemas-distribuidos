package main

import (
	"flag"
	"log"
	"os"
	"strconv"

	"github.com/gofiber/fiber/v2"
	"github.com/joho/godotenv"

	"payment-srv/internal"
	"payment-srv/internal/api"
)

func main() {
	port := flag.Int("port", 8081, "HTTP server port")

	err := godotenv.Load(".env")
	if err != nil {
		log.Panicf("Failed to load environment file!")
	}
	flag.Parse()
	baseURL := os.Getenv("BASE_URL")

	paymentManager := internal.NewPaymentManager(baseURL)
	app := fiber.New()
	app.Put("/api/update-payment/:payment-id", api.UpdatePayment(paymentManager))
	app.Listen(baseURL + ":" + strconv.Itoa(*port))
}
