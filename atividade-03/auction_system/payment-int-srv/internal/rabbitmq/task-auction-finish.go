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

		newPayment := taf.pm.CreateNewPayment(&auctionWinner)
		res := sendNewTransaction(newPayment)
		taf.publishLink(res)
	}

	return nil
}

func sendNewTransaction(newPayment *models.Payment) string {
	return ""
}

func (taf *TaskAuctionFinish) publishLink(payment string) {

}
