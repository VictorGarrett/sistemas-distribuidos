package main

import (
	"flag"
	"log"
	"os"
	"strconv"

	"github.com/gofiber/fiber/v2"
	"github.com/joho/godotenv"
	"github.com/streadway/amqp"

	"payment-srv/internal"
	"payment-srv/internal/api"
	"payment-srv/internal/models"
	"payment-srv/internal/rabbitmq"
)

func main() {
	port := flag.Int("port", 8082, "HTTP server port")

	err := godotenv.Load(".env")
	if err != nil {
		log.Panicf("Failed to load environment file!")
	}
	flag.Parse()
	baseURL := os.Getenv("BASE_URL")

	updatesChannel := make(chan models.PaymentUpdate)
	linksChannel := make(chan models.PaymentLink)

	conn, err := amqp.Dial("amqp://guest:guest@127.0.0.1:5672/%2f")
	if err != nil {
		log.Fatalf("Failed to connect to rabbitmq: %v", err)
	}

	paymentManager := internal.NewPaymentManager(
		"http://"+baseURL+":"+strconv.Itoa(*port),
		updatesChannel,
	)

	taskAuctionFinish, err := rabbitmq.NewTaskAuctionFinish(
		paymentManager,
		conn,
		linksChannel,
	)
	if err != nil {
		log.Fatalf("Failed to Create TaskAuctionFinish: %v", err)
	}

	taskPaymentStatus, err := rabbitmq.NewTaskPaymentStatus(
		paymentManager,
		conn,
		updatesChannel,
	)
	if err != nil {
		log.Fatalf("Failed to Create TaskPaymentStatus: %v", err)
	}

	taskPaymentLink, err := rabbitmq.NewTaskPaymentLink(
		paymentManager,
		conn,
		linksChannel,
	)
	if err != nil {
		log.Fatalf("Failed to Create TaskPaymentLink: %v", err)
	}

	go taskAuctionFinish.Run()
	go taskPaymentStatus.Run()
	go taskPaymentLink.Run()

	app := fiber.New()
	app.Post("/api/update-payment", api.UpdatePayment(paymentManager))

	log.Printf("App initiated on %s:%d", baseURL, *port)
	app.Listen(baseURL + ":" + strconv.Itoa(*port))
}
