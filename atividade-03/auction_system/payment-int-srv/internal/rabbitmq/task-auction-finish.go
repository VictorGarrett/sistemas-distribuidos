package rabbitmq

import (
	"bytes"
	"encoding/json"
	"net/http"
	"payment-srv/internal"
	"payment-srv/internal/models"

	"github.com/gofiber/fiber/v2/log"
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

	amqpChannel, err := conn.Channel()
	if err != nil {
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

	task := TaskAuctionFinish{
		pm:           paymentManager,
		rmqChannel:   amqpChannel,
		linksChannel: linksChannel,
	}

	return &task, nil
}

func (taf *TaskAuctionFinish) Run() error {
	messages, err := taf.rmqChannel.Consume(
		"leilao_vencedor",
		"",
		true,
		false,
		false,
		false,
		nil,
	)
	if err != nil {
		return err
	}

	for msg := range messages {
		var auctionWinner models.NewAuctionWinner
		json.Unmarshal(msg.Body, &auctionWinner)

		res := sendNewTransaction(&auctionWinner)
		transactionID, _ := uuid.Parse(res.ID)
		taf.pm.CreateNewPayment(&auctionWinner, transactionID)
		taf.linksChannel <- models.PaymentLink{
			PaymentID: transactionID,
			Link:      res.Link,
		}
	}

	return nil
}

func sendNewTransaction(auctionWinner *models.NewAuctionWinner) *models.NewTransactionResponse {
	transactionReq := &models.NewTransactionRequest{
		Amount:   auctionWinner.Amount,
		Callback: "callback",
	}

	body, _ := json.Marshal(transactionReq)

	res, err := http.Post(
		"service-url",
		"application/json",
		bytes.NewBuffer(body),
	)
	defer res.Body.Close()

	if err != nil {
		log.Error("Some Error yadayada")
		return nil
	}

	var transactionResponse models.NewTransactionResponse
	err = json.NewDecoder(res.Body).Decode(&transactionResponse)
	if err != nil {
		log.Error("Some Error yadayada")
		return nil
	}

	return &transactionResponse
}
