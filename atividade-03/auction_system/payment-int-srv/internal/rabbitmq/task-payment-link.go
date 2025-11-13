package rabbitmq

import (
	"encoding/json"
	"payment-srv/internal"
	"payment-srv/internal/models"

	"github.com/gofiber/fiber/v2/log"
	"github.com/streadway/amqp"
)

type TaskPaymentLink struct {
	pm           *internal.PaymentManager
	rmqChannel   *amqp.Channel
	linksChannel chan models.PaymentLink
}

func NewTaskPaymentLink(
	paymentManager *internal.PaymentManager,
	conn *amqp.Connection,
	linksChannel chan models.PaymentLink,
) (*TaskPaymentLink, error) {

	amqpChannel, err := conn.Channel()
	if err != nil {
		return nil, err
	}

	amqpChannel.QueueDeclare(
		"link_pagamento",
		true,
		false,
		false,
		false,
		nil,
	)

	task := TaskPaymentLink{
		pm:           paymentManager,
		rmqChannel:   amqpChannel,
		linksChannel: linksChannel,
	}

	return &task, nil
}

func (taf *TaskPaymentLink) Run(conn *amqp.Connection) error {

	amqpChannel, err := conn.Channel()
	if err != nil {
		return err
	}

	for msg := range taf.linksChannel {
		payment := taf.pm.GetPayment(msg.PaymentID)
		body, err := json.Marshal(payment.ToPaymentLinkPublish(&msg))
		if err != nil {
			log.Errorf("Failed to encode structure: %v")
			continue
		}

		log.Infof("Publishing payment link message: %s", string(body))
		amqpChannel.Publish(
			"",
			"link_pagamento",
			false,
			false,
			amqp.Publishing{
				ContentType:  "application/json",
				Body:         body,
				DeliveryMode: amqp.Transient,
			},
		)
	}

	return nil
}
