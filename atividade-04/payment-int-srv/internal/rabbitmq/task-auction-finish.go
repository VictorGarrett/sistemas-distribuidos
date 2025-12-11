package rabbitmq

import (
	"encoding/json"
	"fmt"
	"payment-srv/internal"
	"payment-srv/internal/models"
	"payment-srv/internal/services"

	"github.com/google/uuid"
	"github.com/streadway/amqp"
)

type TaskAuctionFinish struct {
	pm                  *internal.PaymentManager
	rmqChannel          *amqp.Channel
	paymentExtSrvClient *services.PaymentExternalService

	linksChannel chan models.PaymentLink
}

func NewTaskAuctionFinish(
	paymentManager *internal.PaymentManager,
	conn *amqp.Connection,
	linksChannel chan models.PaymentLink,
	paymentExtSrvClient *services.PaymentExternalService,
) (*TaskAuctionFinish, error) {

	fmt.Println("Initializing TaskAuctionFinish")

	amqpChannel, err := conn.Channel()
	if err != nil {
		fmt.Printf("Failed to create AMQP channel %v", err)
		return nil, err
	}

	err = amqpChannel.ExchangeDeclare(
		"leilao_vencedor", // name
		"fanout",          // type
		false,             // durable
		false,             // auto-deleted
		false,             // internal
		false,             // no-wait
		nil,               // arguments
	)

	if err != nil {
		return nil, err
	}

	fmt.Println("Declared leilao_vencedor exchange")
	task := TaskAuctionFinish{
		pm:                  paymentManager,
		rmqChannel:          amqpChannel,
		linksChannel:        linksChannel,
		paymentExtSrvClient: paymentExtSrvClient,
	}

	return &task, nil
}

func (taf *TaskAuctionFinish) Run() error {
	fmt.Println("Running TaskAuctionFinish")

	defer func() {
		fmt.Printf("O I am slain")
	}()

	queue, err := taf.rmqChannel.QueueDeclare(
		"",    // empty name to let RabbitMQ generate a random name
		false, // durable
		true,  // delete when unused
		true,  // exclusive
		false, // no-wait
		nil,   // arguments
	)
	if err != nil {
		return fmt.Errorf("failed to declare a random queue: %v", err)
	}
	fmt.Printf("Declared random queue: %s\n", queue.Name)

	err = taf.rmqChannel.QueueBind(
		queue.Name,        // queue name
		"",                // routing key
		"leilao_vencedor", // exchange name
		false,             // no-wait
		nil,               // arguments
	)
	if err != nil {
		return fmt.Errorf("failed to bind queue to exchange: %v", err)
	}
	fmt.Printf("Bound queue %s to exchange leilao_vencedor\n", queue.Name)

	messages, err := taf.rmqChannel.Consume(
		queue.Name, // queue name
		"",         // consumer tag
		true,       // auto-ack
		false,      // exclusive
		false,      // no-local
		false,      // no-wait
		nil,        // arguments
	)
	if err != nil {
		return fmt.Errorf("failed to create consumer: %v", err)
	}
	fmt.Println("Consumer created")

	for msg := range messages {
		fmt.Printf("Received message: %s\n", string(msg.Body))
		var auctionWinner models.NewAuctionWinner
		err := json.Unmarshal(msg.Body, &auctionWinner)

		if err != nil {
			fmt.Printf("Error unmarshalling message for won auction: %v", err)
			fmt.Printf("Raw message body: %s", msg.Body)
			continue
		}

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
	res, err := taf.paymentExtSrvClient.CreateTransaction(auctionWinner.Amount)

	if err != nil {
		fmt.Println("Failed to req from ext payment srv", err)
		return nil
	}

	return &models.NewTransactionResponse{
		ID:     res.TransactionId,
		Amount: res.Amount,
		Link:   res.PaymentLink,
	}
}
