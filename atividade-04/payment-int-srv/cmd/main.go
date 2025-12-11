package main

import (
	"flag"
	"fmt"
	"log"
	"net"
	"os"

	"github.com/joho/godotenv"
	"github.com/streadway/amqp"
	"google.golang.org/grpc"

	"payment-srv/internal"
	"payment-srv/internal/api"
	"payment-srv/internal/models"
	"payment-srv/internal/rabbitmq"
	"payment-srv/internal/services"
	pis "payment-srv/proto-go/payment-int-srv"
)

func main() {
	dockerized := flag.Bool("docker", false, "Specifies if it's running as a docker container")
	flag.Parse()

	if !*dockerized {
		log.Println("Running outside a docker container")
		err := godotenv.Load(".env")
		if err != nil {
			log.Panicf("Failed to load environment file!")
		}
	}
	baseURL := os.Getenv("BASE_URL")
	port := os.Getenv("PORT")
	rmqURL := os.Getenv("RMQ_URL")
	paymentExtSrvURL := os.Getenv("PAYMENT_EXT_SRV_URL")

	updatesChannel := make(chan models.PaymentUpdate)
	linksChannel := make(chan models.PaymentLink)

	conn, err := amqp.Dial(rmqURL)
	if err != nil {
		log.Fatalf("Failed to connect to rabbitmq: %v", err)
	}

	paymentManager := internal.NewPaymentManager(
		"http://"+baseURL+":"+port,
		updatesChannel,
	)

	paymentExtSrv, err := services.NewPaymentExternalService(paymentExtSrvURL)
	if err != nil {
		log.Fatalf("Failed to start Payment External Service Client: %v", err)
	}

	taskAuctionFinish, err := rabbitmq.NewTaskAuctionFinish(
		paymentManager,
		conn,
		linksChannel,
		paymentExtSrv,
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

	pisServer := api.NewPaymentInternalServiceServer(paymentManager)
	grpcServer := grpc.NewServer()

	pis.RegisterPaymentInternalServiceServer(grpcServer, pisServer)

	socket, err := net.Listen("tcp", baseURL+":"+port)
	if err != nil {
		log.Fatalf("Failed to listen on %s", baseURL+":"+port)
	}

	fmt.Println("Starting GRPC Server")
	if err := grpcServer.Serve(socket); err != nil {
		log.Fatalf("Failed to start grpcServer: %v", err)
	}
}
