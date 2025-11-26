package handlers

import (
	"encoding/json"
	"fmt"
	"gateway/internal/rabbitmq"
	"io"
	"log"
	"net/http"
	"os"
)

type Client struct {
	ClientID int
	Events   chan rabbitmq.EventMessage
}

type PaymentLink struct {
	AuctionID uint32  `json:"auction_id"`
	ClientID  uint32  `json:"client_id"`
	Value     float64 `json:"value"`
	Link      string  `json:"link"`
}

type PaymentStatus struct {
	AuctionID uint32  `json:"auction_id"`
	ClientID  uint32  `json:"client_id"`
	Value     float64 `json:"value"`
	Status    string  `json:"link"`
}

func contains(slice []int, value int) bool {
	for _, v := range slice {
		if v == value {
			return true
		}
	}
	return false
}

// SseBroker manages active client connections and broadcasts events.
type SseBroker struct {
	// Channel to broadcast events to.
	// We'll pass this to our RabbitMQ consumer.
	Broadcast chan rabbitmq.EventMessage

	// Internal channels for managing clients
	newClients    chan Client
	closedClients chan int
	clients       map[int]chan rabbitmq.EventMessage

	clientInterests map[int][]int
}

// NewSseBroker creates and starts a new SseBroker.
func NewSseBroker() *SseBroker {
	broker := &SseBroker{
		Broadcast:     make(chan rabbitmq.EventMessage),
		newClients:    make(chan Client),
		closedClients: make(chan int),
		clients:       make(map[int]chan rabbitmq.EventMessage),
	}

	// Start the broker's event loop in a goroutine
	go broker.run()
	return broker
}

// run is the broker's main event loop.
func (b *SseBroker) run() {
	rmqURL := os.Getenv("RMQ_URL")
	eventChannel, err := rabbitmq.Consume(rmqURL)
	if err != nil {
		log.Fatalf("Failed to start RabbitMQ consumer: %v", err)
	}

	// Start a goroutine to listen for messages and broadcast them
	go func() {
		for msg := range eventChannel {
			b.Broadcast <- msg
		}
	}()

	for {
		select {
		case client := <-b.newClients:
			// A new client has connected. Add it to the map
			b.clients[client.ClientID] = client.Events
			log.Printf("Received SSE add request: %d", client.ClientID)

		case client := <-b.closedClients:
			// A client has disconnected. Remove it from the map
			delete(b.clients, client)
			log.Println("SSE client removed. Total clients:", len(b.clients))

		case event := <-b.Broadcast:
			log.Printf("Received event: %+v", event)
			// A new event has arrived. Broadcast it to all clients.
			for id, ch := range b.clients {
				log.Printf("Client Interests: %+v", b.clientInterests)
				if contains(b.clientInterests[id], event.AuctionId) {
					log.Printf("Client ID: %d, Channel: %v", id, ch)

					select {
					case ch <- event:
						// Message sent
					default:
						// Client's buffer is full, or they are slow.
						// We don't block. We could log this if needed.
					}
				}
			}
		}
	}
}

// SseHandler is the HTTP handler for the SSE connection.
type SseHandler struct {
	broker *SseBroker
}

// NewSseHandler creates a new SseHandler.
func NewSseHandler(broker *SseBroker) *SseHandler {
	return &SseHandler{broker: broker}
}

// HandleEvents is the http.HandlerFunc for the SSE endpoint.
func (h *SseHandler) HandleEvents(w http.ResponseWriter, r *http.Request) {
	// Set the required headers for SSE
	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")

	// Get the Flusher interface
	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "Streaming unsupported!", http.StatusInternalServerError)
		return
	}

	// Extract client ID from query (e.g. /events?clientID=123)
	clientIDStr := r.URL.Query().Get("clientID")
	if clientIDStr == "" {
		http.Error(w, "missing clientID query parameter", http.StatusBadRequest)
		return
	}

	var clientID int
	if _, err := fmt.Sscan(clientIDStr, &clientID); err != nil {
		http.Error(w, "invalid clientID", http.StatusBadRequest)
		return
	}

	// Create the client with its own Event channel
	clientEvents := make(chan rabbitmq.EventMessage, 10)
	client := Client{
		ClientID: clientID,
		Events:   clientEvents,
	}

	// Register the client with the broker
	h.broker.newClients <- client

	// Get the request context to detect client disconnect
	ctx := r.Context()

	// Defer unregistering the client when the handler exits
	defer func() {
		h.broker.closedClients <- clientID
	}()

	// Start the event loop for this client
	for {
		select {
		case <-ctx.Done():
			// Client disconnected
			return

		case message := <-clientEvents:
			log.Println("SSE channel received msg")
			// Received a message from the broker. Send it to the client.
			// The SSE format is "data: <message>\n\n"

			if message.EventType == "link_pagamento" {
				var event PaymentLink
				json.Unmarshal([]byte(message.Data), &event)

				if event.ClientID != uint32(clientID) {
					// Not intended for this client
					continue
				}
			}
			if message.EventType == "status_pagamento" {
				var event PaymentStatus
				json.Unmarshal([]byte(message.Data), &event)

				if event.ClientID != uint32(clientID) {
					// Not intended for this client
					continue
				}
			}

			payload, err := json.Marshal(message)
			if err != nil {
				// Failed to serialize message; treat as client disconnect or skip
				return
			}
			log.Println("SSE message sent: ", payload)
			_, err = fmt.Fprintf(w, "data: %s\n\n", payload)
			if err != nil {
				// Error most likely means client disconnected
				return
			}
			// Flush the data to the client
			flusher.Flush()
		}
	}
}

func (h *SseHandler) HandleSubscribe(w http.ResponseWriter, r *http.Request) {
	body, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, "Failed to read body", http.StatusBadRequest)
		return
	}
	defer r.Body.Close()

	var requestData struct {
		ClientID int   `json:"client_id"`
		Auctions []int `json:"auctions"`
	}

	err = json.Unmarshal(body, &requestData)
	if err != nil {
		http.Error(w, "Invalid JSON", http.StatusBadRequest)
		return
	}

	if h.broker.clientInterests == nil {
		h.broker.clientInterests = make(map[int][]int)
	}

	log.Printf("Subscribing Client ID: %d", requestData.ClientID)
	h.broker.clientInterests[requestData.ClientID] = append(h.broker.clientInterests[requestData.ClientID], requestData.Auctions...)

	resp := []byte(`{"status":"subscribed"}`)

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	w.Write(resp)
}

func (h *SseHandler) HandleUnsubscribe(w http.ResponseWriter, r *http.Request) {
	body, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, "Failed to read body", http.StatusBadRequest)
		return
	}
	defer r.Body.Close()

	var requestData struct {
		ClientID int   `json:"client_id"`
		Auctions []int `json:"auctions"`
	}

	err = json.Unmarshal(body, &requestData)
	if err != nil {
		http.Error(w, "Invalid JSON", http.StatusBadRequest)
		return
	}

	fmt.Printf("Unsubscribing Client ID: %d from auctions: %+v\n", requestData.ClientID, requestData.Auctions)
	if h.broker.clientInterests == nil {
		http.Error(w, "No subscriptions found", http.StatusBadRequest)
		return
	}

	// Remove the specified auctions from the client's interests
	currentInterests := h.broker.clientInterests[requestData.ClientID]
	updatedInterests := []int{}
	for _, auction := range currentInterests {
		if !contains(requestData.Auctions, auction) {
			updatedInterests = append(updatedInterests, auction)
		}
	}

	// Update the client's interests
	h.broker.clientInterests[requestData.ClientID] = updatedInterests

	resp := []byte(`{"status":"unsubscribed"}`)

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	w.Write(resp)
}
