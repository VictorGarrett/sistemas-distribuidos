package main

import (
	"flag"
	"log"
	"os"

	"github.com/gofiber/fiber/v2"
	"github.com/joho/godotenv"
	"github.com/streadway/amqp"

	"payment-srv/internal"
	"payment-srv/internal/api"
	"payment-srv/internal/models"
	"payment-srv/internal/rabbitmq"
)

func main() {
	dockerized:= flag.Bool("docker", false, "Specifies if it's running as a docker container")
	flag.Parse()
	
	if !*dockerized{
		log.Println("Running outside a docker container")
		err := godotenv.Load(".env")
		if err != nil {
			log.Panicf("Failed to load environment file!")
		}
	}
	baseURL := os.Getenv("BASE_URL")
	port := os.Getenv("PORT")
	rmqURL := os.Getenv("RMQ_URL")

	updatesChannel := make(chan models.PaymentUpdate)
	linksChannel := make(chan models.PaymentLink)

	conn, err := amqp.Dial(rmqURL)
	if err != nil {
		log.Fatalf("Failed to connect to rabbitmq: %v", err)
	}

	paymentManager := internal.NewPaymentManager(
		"http://"+baseURL+":"+ port,
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

	log.Printf("App initiated on %s:%d", baseURL, port)
	app.Listen(baseURL + ":" + port)
}
