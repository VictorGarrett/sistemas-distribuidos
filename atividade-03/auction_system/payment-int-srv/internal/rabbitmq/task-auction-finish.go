package rabbitmq

import (
	"encoding/json"
	"payment-srv/internal"
	"payment-srv/internal/models"

	"github.com/streadway/amqp"
)

type TaskAuctionFinish struct {
	pm         *internal.PaymentManager
	rmqChannel *amqp.Channel
}

func NewTaskAuctionFinish(paymentManager *internal.PaymentManager, conn *amqp.Connection) (*TaskAuctionFinish, error) {
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
		pm:         paymentManager,
		rmqChannel: amqpChannel,
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

		newPayment := taf.pm.CreateNewPayment(&auctionWinner)
		sendNewTransaction(newPayment)
	}

	return nil
}

func sendNewTransaction(newPayment *models.Payment) {
}
