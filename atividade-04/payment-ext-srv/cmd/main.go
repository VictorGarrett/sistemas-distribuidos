package main

import (
	"flag"
	"fmt"
	"net"
	"os"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/log"
	"github.com/joho/godotenv"
	"google.golang.org/grpc"

	"payment-ext-srv/internal"
	"payment-ext-srv/internal/api"
	"payment-ext-srv/internal/services"
	pes "payment-ext-srv/proto-go/payment-ext-srv"
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
	grpcPort := os.Getenv("GRPC_PORT")
	restPort := os.Getenv("REST_PORT")
	pm := internal.NewTransactionManager("http://" + baseURL + ":" + restPort)

	paymentIntSrvUrl := os.Getenv("PAYMENT_INT_SRV_URL")
	paymentIntSrv, err := services.NewPaymentInternalService(paymentIntSrvUrl)
	if err != nil {
		log.Fatalf("Failed to start PaymentInternalService")
	}

	pesServer := api.NewPaymentExternalServiceServer(pm)
	grpcServer := grpc.NewServer()

	pes.RegisterPaymentExternalServiceServer(grpcServer, pesServer)

	socket, err := net.Listen("tcp", baseURL+":"+grpcPort)
	if err != nil {
		log.Fatalf("Failed to listen on %s:%d", baseURL, grpcPort)
	}

	go func() {
		fmt.Println("Starting GRPC Server")
		if err := grpcServer.Serve(socket); err != nil {
			log.Fatalf("Failed to start grpcServer")
		}
	}()

	app := fiber.New()
	app.Post("/transaction", api.HandleNewTransaction(pm))
	app.Get("/pay/:tid", api.HandleTransactionPay(pm, paymentIntSrv))
	app.Get("/transaction", api.HandleGetTransaction(pm))
	app.Listen(baseURL + ":" + restPort)

	fmt.Println("Exiting...")
}
