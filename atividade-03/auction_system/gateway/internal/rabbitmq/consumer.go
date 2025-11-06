package rabbitmq

import (
	"log"

	"github.com/streadway/amqp"
)

func Consume(url, queueName string) error {
	conn, err := amqp.Dial(url)
	if err != nil {
		return err
	}
	defer conn.Close()

	ch, err := conn.Channel()
	if err != nil {
		return err
	}
	defer ch.Close()

	msgs, err := ch.Consume(
		queueName,
		"",
		true,  // auto-ack
		false, // exclusive
		false, // no-local
		false, // no-wait
		nil,
	)
	if err != nil {
		return err
	}

	log.Printf("RabbitMQ consumer started on queue: %s", queueName)
	for msg := range msgs {
		log.Printf("Received message: %s", msg.Body)
	}
	return nil
}
