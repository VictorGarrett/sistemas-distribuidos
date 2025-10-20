package main

import (
	"flag"
	"os"
	"strconv"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/log"
	"github.com/joho/godotenv"
)

func main() {
	port := flag.Int("port", 8080, "TCP port")

	err := godotenv.Load(".env")
	if err != nil {
		log.Panicf("Failed to load environment file!")
	}
	flag.Parse()

	app := fiber.New()
	app.Post("/new-payment")
	app.Post("/pay")
	app.Listen(os.Getenv("BASE_URL") + ":" + strconv.Itoa(*port))
}
