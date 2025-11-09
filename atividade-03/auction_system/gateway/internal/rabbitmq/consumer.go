package rabbitmq

import (
	"log"

	"github.com/streadway/amqp"
)

type Bid struct {
	AuctionID  uint32  `json:"auction_id"`
	ClientID   uint32  `json:"client_id"`
	Value      float64 `json:"value"`
	Signature  string  `json:"signature"`
	PublicKey  string  `json:"public_key"`
	Valid      bool    `json:"valid"`
}


type EventMessage struct {
	EventType string
	AuctionId int
	data 	[]byte
}



func Consume(url) error {


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

	// Lance validado
	_, err = ch.QueueDeclare(
		"lance_validado", // name
		true,  // durable
		false, // auto-delete
		false, // exclusive
		false, // no-wait
		nil,   // arguments
	)
	if err != nil {
		return err
	}

	lance_validado_msgs, err := ch.Consume(
		"lance_validado",
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
	// Lance invalidado
	_, err = ch.QueueDeclare(
		"lance_invalidado", // name
		true,  // durable
		false, // auto-delete
		false, // exclusive
		false, // no-wait
		nil,   // arguments
	)
	if err != nil {
		return err
	}

	lance_invalidado_msgs, err := ch.Consume(
		"lance_invalidado",
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
	// Leilao vencedor
	_, err = ch.QueueDeclare(
		"leilao_vencedor", // name
		true,  // durable
		false, // auto-delete
		false, // exclusive
		false, // no-wait
		nil,   // arguments
	)
	if err != nil {
		return err
	}

	lance_validado_msgs, err := ch.Consume(
		"leilao_vencedor",
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


	eventChannel := make(chan EventMessage)

	go func() {
		for {
			select {
			case msg := <-lance_validado_msgs:
				var bid Bid
				err := json.Unmarshal(msg.Body, &bid)
				if err != nil {
					log.Printf("Error unmarshalling message: %v", err)
					continue
				}
				log.Printf("Received valid bid: %+v", bid)
				eventChannel <- EventMessage{
					AuctionId: int(bid.AuctionID),
					EventType: "lance_validado",
					Data:      msg.Body,
				}
			case msg := <-lance_invalidado_msgs:
				var bid Bid
				err := json.Unmarshal(msg.Body, &bid)
				if err != nil {
					log.Printf("Error unmarshalling message: %v", err)
					continue
				}
				log.Printf("Received invalid bid: %+v", bid)
				eventChannel <- EventMessage{
					AuctionId: int(bid.AuctionID),
					EventType: "lance_invalidado",
					Data:      msg.Body,
				}
			case msg := <-leilao_vencedor_msgs:
				var bid Bid
				err := json.Unmarshal(msg.Body, &bid)
				if err != nil {
					log.Printf("Error unmarshalling message: %v", err)
					continue
				}
				log.Printf("Received winner: %+v", bid)
				eventChannel <- EventMessage{
					AuctionId: int(bid.AuctionID),
					EventType: "leilao_vencedor",
					Data:      msg.Body,
				}
			}
		}
	}()

	
	return eventChannel
}
