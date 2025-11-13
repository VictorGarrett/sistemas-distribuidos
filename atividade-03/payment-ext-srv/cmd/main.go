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
	port := flag.Int("port", 7070, "TCP port")

	err := godotenv.Load(".env")
	if err != nil {
		log.Panicf("Failed to load environment file!")
	}
	flag.Parse()

	baseURL := os.Getenv("BASE_URL")
	pm := internal.NewTransactionManager("http://" + baseURL + ":" + strconv.Itoa(*port))

	app := fiber.New()
	app.Post("/transaction", api.HandleNewTransaction(pm))
	app.Post("/transaction/pay/:tid", api.HandleTransactionPay(pm))
	app.Get("/transaction", api.HandleGetTransaction(pm))
	app.Listen(baseURL + ":" + strconv.Itoa(*port))
}
