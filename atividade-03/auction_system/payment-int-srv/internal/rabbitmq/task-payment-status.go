package rabbitmq

import (
	"encoding/json"
	"payment-srv/internal"
	"payment-srv/internal/models"

	"github.com/gofiber/fiber/v2/log"
	"github.com/streadway/amqp"
)

type TaskPaymentStatus struct {
	pm             *internal.PaymentManager
	rmqChannel     *amqp.Channel
	updatesChannel chan models.PaymentUpdate
}

func NewTaskPaymentStatus(
	paymentManager *internal.PaymentManager,
	conn *amqp.Connection,
	updatesChannel chan models.PaymentUpdate,
) (*TaskPaymentStatus, error) {

	amqpChannel, err := conn.Channel()
	if err != nil {
		return nil, err
	}

	amqpChannel.QueueDeclare(
		"status_pagamento",
		true,
		false,
		false,
		false,
		nil,
	)

	task := TaskPaymentStatus{
		pm:             paymentManager,
		rmqChannel:     amqpChannel,
		updatesChannel: updatesChannel,
	}

	return &task, nil
}

func (taf *TaskPaymentStatus) Run() error {
	for msg := range taf.updatesChannel {
		payment := taf.pm.GetPayment(msg.PaymentID)
		body, err := json.Marshal(payment.ToPaymentUpdatePublish(&msg))
		if err != nil {
			log.Errorf("Failed to encode structure: %v")
			continue
		}

		taf.rmqChannel.Publish(
			"",
			"status_pagamento",
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
