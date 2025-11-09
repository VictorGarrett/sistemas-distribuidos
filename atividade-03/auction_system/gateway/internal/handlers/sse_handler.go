package handlers

import (
	"fmt"
	"log"
	"net/http"
)

type Client struct {
	ClientID int
	Events chan []byte
}

type EventMessage struct {
	EventType string
	AuctionId int
	data 	[]byte
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
	Broadcast chan []byte

	// Internal channels for managing clients
	newClients    chan Client
	closedClients chan int
	clients       	

	clientInterests map[int][]int
}

// NewSseBroker creates and starts a new SseBroker.
func NewSseBroker() *SseBroker {
	broker := &SseBroker{
		Broadcast:     make(chan []byte),
		newClients:    make(chan Client),
		closedClients: make(chan int),
		clients:       make(map[int]chan []byte),
	}

	// Start the broker's event loop in a goroutine
	go broker.run()
	return broker
}

// run is the broker's main event loop.
func (b *SseBroker) run() {


	eventChannel, err := rabbitmq.Consume("amqp://guest:guest@localhost:5672/")
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
			b.clients[client.ClientID] = Client.Events
			log.Println("SSE client added. Total clients:", len(b.clients))

		case client := <-b.closedClients:
			// A client has disconnected. Remove it from the map
			delete(b.clients, client)
			close(client) // Close the channel
			log.Println("SSE client removed. Total clients:", len(b.clients))

		case event := <-b.Broadcast:
			// A new event has arrived. Broadcast it to all clients.
			for id, ch := range b.clients {
				
				if contains(b.clientInterests[id], event.AuctionId) {
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

	// Create a new channel for this specific client
	messageChan := make(chan []byte, 10) // Buffered channel

	// Register the new client with the broker
	h.broker.newClients <- messageChan

	// Get the request context to detect client disconnect
	ctx := r.Context()

	// Defer unregistering the client when the handler exits
	defer func() {
		h.broker.closedClients <- messageChan
	}()

	// Start the event loop for this client
	for {
		select {
		case <-ctx.Done():
			// Client disconnected
			return

		case message := <-messageChan:
			// Received a message from the broker. Send it to the client.
			// The SSE format is "data: <message>\n\n"
			_, err := fmt.Fprintf(w, "data: %s\n\n", message)
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
		ClientID int   `json:"clientID"`
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
		ClientID int   `json:"clientID"`
		Auctions []int `json:"auctions"`
	}

	err = json.Unmarshal(body, &requestData)
	if err != nil {
		http.Error(w, "Invalid JSON", http.StatusBadRequest)
		return
	}

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