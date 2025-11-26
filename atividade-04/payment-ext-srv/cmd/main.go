package main

import (
	"flag"
	"os"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/log"
	"github.com/joho/godotenv"

	"payment-ext-srv/internal"
	"payment-ext-srv/internal/api"
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

	baseURL := os.Getenv("BASE_URL")
	port := os.Getenv("PORT")
	pm := internal.NewTransactionManager("http://" + baseURL + ":" + port)

	app := fiber.New()
	app.Post("/transaction", api.HandleNewTransaction(pm))
	app.Get("/pay/:tid", api.HandleTransactionPay(pm))
	app.Get("/transaction", api.HandleGetTransaction(pm))
	app.Listen(baseURL + ":" + port)
}
