package rabbitmq

import (
	"encoding/json"
	"log"

	"github.com/streadway/amqp"
)

type Bid struct {
	AuctionID uint32  `json:"auction_id"`
	ClientID  uint32  `json:"client_id"`
	Value     float64 `json:"value"`
	Signature string  `json:"signature"`
	PublicKey string  `json:"public_key"`
	Valid     bool    `json:"valid"`
}

type PaymentLink struct {
	AuctionID uint32  `json:"auction_id"`
	Value     float64 `json:"value"`
	Link      string  `json:"link"`
}

type PaymentStatus struct {
	AuctionID uint32  `json:"auction_id"`
	ClientID  uint32  `json:"client_id"`
	Value     float64 `json:"value"`
	Status    string  `json:"link"`
}

type EventMessage struct {
	EventType string
	AuctionId int
	Data      string
}

func Consume(url string) (chan EventMessage, error) {

	conn, err := amqp.Dial(url)
	if err != nil {
		return nil, err
	}

	ch, err := conn.Channel()
	if err != nil {
		return nil, err
	}

	// Lance validado
	_, err = ch.QueueDeclare(
		"lance_validado", // name
		false,            // durable
		false,            // auto-delete
		false,            // exclusive
		false,            // no-wait
		nil,              // arguments
	)
	if err != nil {
		return nil, err
	}

	lance_validado_msgs, _ := ch.Consume(
		"lance_validado",
		"",
		true,  // auto-ack
		false, // exclusive
		false, // no-local
		false, // no-wait
		nil,
	)

	// Lance invalidado
	_, err = ch.QueueDeclare(
		"lance_invalidado", // name
		false,              // durable
		false,              // auto-delete
		false,              // exclusive
		false,              // no-wait
		nil,                // arguments
	)
	if err != nil {
		return nil, err
	}

	lance_invalidado_msgs, _ := ch.Consume(
		"lance_invalidado",
		"",
		true,  // auto-ack
		false, // exclusive
		false, // no-local
		false, // no-wait
		nil,
	)
	// Leilao vencedor
	err = ch.ExchangeDeclare(
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

	q, err := ch.QueueDeclare(
		"",    // name (empty string for a random queue)
		false, // durable
		true,  // auto-delete
		true,  // exclusive
		false, // no-wait
		nil,   // arguments
	)
	if err != nil {
		return nil, err
	}

	err = ch.QueueBind(
		q.Name,            // queue name
		"",                // routing key
		"leilao_vencedor", // exchange
		false,             // no-wait
		nil,               // arguments
	)
	if err != nil {
		return nil, err
	}

	leilao_vencedor_msgs, _ := ch.Consume(
		q.Name,
		"",
		true,  // auto-ack
		false, // exclusive
		false, // no-local
		false, // no-wait
		nil,
	)

	// link pagamento
	_, err = ch.QueueDeclare(
		"link_pagamento", // name
		false,            // durable
		false,            // auto-delete
		false,            // exclusive
		false,            // no-wait
		nil,              // arguments
	)
	if err != nil {
		return nil, err
	}

	link_pagamento_msgs, _ := ch.Consume(
		"link_pagamento",
		"",
		true,  // auto-ack
		false, // exclusive
		false, // no-local
		false, // no-wait
		nil,
	)

	// status pagamento
	_, err = ch.QueueDeclare(
		"status_pagamento", // name
		false,              // durable
		false,              // auto-delete
		false,              // exclusive
		false,              // no-wait
		nil,                // arguments
	)
	if err != nil {
		return nil, err
	}

	status_pagamento_msgs, _ := ch.Consume(
		"status_pagamento",
		"",
		true,  // auto-ack
		false, // exclusive
		false, // no-local
		false, // no-wait
		nil,
	)

	eventChannel := make(chan EventMessage)

	go func() {
		for {
			select {
			case msg := <-lance_validado_msgs:
				log.Printf("Received message from lance_validado queue: %s", msg.Body)
				var bid Bid
				err := json.Unmarshal(msg.Body, &bid)
				if err != nil {
					log.Printf("Error unmarshalling message for lance_validado: %v", err)
					log.Printf("Raw message body: %s", msg.Body)

					continue
				}
				log.Printf("Received valid bid: %+v", bid)
				eventChannel <- EventMessage{
					AuctionId: int(bid.AuctionID),
					EventType: "lance_validado",
					Data:      string(msg.Body),
				}
			case msg := <-lance_invalidado_msgs:
				var bid Bid
				err := json.Unmarshal(msg.Body, &bid)
				if err != nil {
					log.Printf("Error unmarshalling message for lance_invalidado: %v", err)
					log.Printf("Raw message body: %s", msg.Body)

					continue
				}
				log.Printf("Received invalid bid: %+v", bid)
				eventChannel <- EventMessage{
					AuctionId: int(bid.AuctionID),
					EventType: "lance_invalidado",
					Data:      string(msg.Body),
				}
			case msg := <-leilao_vencedor_msgs:
				var bid Bid
				err := json.Unmarshal(msg.Body, &bid)
				if err != nil {
					log.Printf("Error unmarshalling message for leilao_vencedor: %v", err)
					log.Printf("Raw message body: %s", msg.Body)
					continue
				}
				log.Printf("Received winner: %+v", bid)
				eventChannel <- EventMessage{
					AuctionId: int(bid.AuctionID),
					EventType: "leilao_vencedor",
					Data:      string(msg.Body),
				}
			case msg := <-link_pagamento_msgs:
				var link PaymentLink
				err := json.Unmarshal(msg.Body, &link)
				if err != nil {
					log.Printf("Error unmarshalling message for payment link: %v", err)
					log.Printf("Raw message body: %s", msg.Body)
					continue
				}
				log.Printf("Received payment link: %+v", link)
				eventChannel <- EventMessage{
					AuctionId: int(link.AuctionID),
					EventType: "link_pagamento",
					Data:      string(msg.Body),
				}
			case msg := <-status_pagamento_msgs:
				var status PaymentStatus
				err := json.Unmarshal(msg.Body, &status)
				if err != nil {
					log.Printf("Error unmarshalling message for payment status: %v", err)
					log.Printf("Raw message body: %s", msg.Body)
					continue
				}
				log.Printf("Received payment status: %+v", status)
				eventChannel <- EventMessage{
					AuctionId: int(status.AuctionID),
					EventType: "status_pagamento",
					Data:      string(msg.Body),
				}
			}
		}
	}()

	return eventChannel, nil
}
