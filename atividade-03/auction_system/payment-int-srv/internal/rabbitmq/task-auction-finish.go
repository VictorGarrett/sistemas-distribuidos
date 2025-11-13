package rabbitmq

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"payment-srv/internal"
	"payment-srv/internal/models"

	"github.com/google/uuid"
	"github.com/streadway/amqp"
)

type TaskAuctionFinish struct {
	pm         *internal.PaymentManager
	rmqChannel *amqp.Channel

	linksChannel chan models.PaymentLink
}

func NewTaskAuctionFinish(
	paymentManager *internal.PaymentManager,
	conn *amqp.Connection,
	linksChannel chan models.PaymentLink,
) (*TaskAuctionFinish, error) {

	fmt.Println("Initializing TaskAuctionFinish")

	amqpChannel, err := conn.Channel()
	if err != nil {
		fmt.Printf("Failed to create AMQP channel %v", err)
		return nil, err
	}

	amqpChannel.QueueDeclare(
		"leilao_vencedor",
		true,
		false,
		false,
		false,
		nil,
	)
	fmt.Println("Queue leilao_vencedor declared")
	task := TaskAuctionFinish{
		pm:           paymentManager,
		rmqChannel:   amqpChannel,
		linksChannel: linksChannel,
	}

	return &task, nil
}

func (taf *TaskAuctionFinish) Run(conn *amqp.Connection) error {
	fmt.Println("Running TaskAuctionFinish")

	amqpChannel, err := conn.Channel()
	if err != nil {
		fmt.Printf("Failed to create AMQP channel %v", err)
		return err
	}

	messages, err := amqpChannel.Consume(
		"leilao_vencedor",
		"",
		true,
		false,
		false,
		false,
		nil,
	)
	if err != nil {
		fmt.Printf("Failed to create leilao_vencedor consumer %v", err)
		return err
	}
	fmt.Println("Waiting for messages on queue 'leilao_vencedor'")

	for msg := range messages {
		fmt.Printf("Received message: %s\n", string(msg.Body))
		var auctionWinner models.NewAuctionWinner
		json.Unmarshal(msg.Body, &auctionWinner)

		res := taf.sendNewTransaction(&auctionWinner)
		transactionID, _ := uuid.Parse(res.ID)
		taf.pm.CreateNewPayment(&auctionWinner, transactionID)
		taf.linksChannel <- models.PaymentLink{
			PaymentID: transactionID,
			Link:      res.Link,
		}
	}
	fmt.Println("ended TaskAuctionFinish")

	return nil
}

func (taf *TaskAuctionFinish) sendNewTransaction(auctionWinner *models.NewAuctionWinner) *models.NewTransactionResponse {
	transactionReq := &models.NewTransactionRequest{
		Amount:   auctionWinner.Amount,
		Callback: taf.pm.Url + "/api/update-payment/",
	}

	body, _ := json.Marshal(transactionReq)

	res, err := http.Post(
		"localhost:7070/transaction",
		"application/json",
		bytes.NewBuffer(body),
	)
	defer res.Body.Close()

	if err != nil {
		fmt.Println("Some Error yadayada")
		return nil
	}

	var transactionResponse models.NewTransactionResponse
	err = json.NewDecoder(res.Body).Decode(&transactionResponse)
	if err != nil {
		fmt.Println("Some Error yadayada")
		return nil
	}
	fmt.Printf("Received response: %+v\n", transactionResponse)
	return &transactionResponse
}
