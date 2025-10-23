package main

import (
	"flag"
	"os"
	"strconv"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/log"
	"github.com/joho/godotenv"

	"payment-ext-srv/internal"
	"payment-ext-srv/internal/api"
)

func main() {
	port := flag.Int("port", 8080, "TCP port")

	err := godotenv.Load(".env")
	if err != nil {
		log.Panicf("Failed to load environment file!")
	}
	flag.Parse()

	baseURL := os.Getenv("BASE_URL")

	pm := internal.NewPaymentManager(baseURL)

	app := fiber.New()
	app.Post("/new-payment", api.HandleNewPayment(pm))
	app.Post("/pay", api.HandlePay(pm))
	app.Listen(baseURL + ":" + strconv.Itoa(*port))
}
