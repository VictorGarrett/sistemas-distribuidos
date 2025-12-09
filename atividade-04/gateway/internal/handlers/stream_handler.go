package handlers

import (
    "context"
    "encoding/json"
    "log"
    "os"
    "sync"

    "gateway/internal/rabbitmq"
    pb "gateway/proto-go/gateway"
)

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
    Status    string  `json:"status"`
}

func contains(slice []uint32, value uint32) bool {
    for _, v := range slice {
        if v == value {
            return true
        }
    }
    return false
}

// GrpcBroker manages active client connections and broadcasts events.
type GrpcBroker struct {
    mu              sync.RWMutex
    clients         map[uint32]chan rabbitmq.EventMessage
    clientInterests map[uint32][]uint32
    Broadcast       chan rabbitmq.EventMessage
}

// NewGrpcBroker creates and starts a new GrpcBroker.
func NewGrpcBroker() *GrpcBroker {
    broker := &GrpcBroker{
        clients:         make(map[uint32]chan rabbitmq.EventMessage),
        clientInterests: make(map[uint32][]uint32),
        Broadcast:       make(chan rabbitmq.EventMessage),
    }

    go broker.run()
    return broker
}

// run is the broker's main event loop.
func (b *GrpcBroker) run() {
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

    for event := range b.Broadcast {
        log.Printf("Received event: %+v", event)
        
        b.mu.RLock()
        for clientID, ch := range b.clients {
            interests := b.clientInterests[clientID]
            if contains(interests, uint32(event.AuctionId)) {
                log.Printf("Client ID: %d interested in auction: %d", clientID, event.AuctionId)
                
                select {
                case ch <- event:
                    // Message sent
                default:
                    // Client's buffer is full, skip
                    log.Printf("Client %d buffer full, skipping message", clientID)
                }
            }
        }
        b.mu.RUnlock()
    }
}

func (b *GrpcBroker) addClient(clientID uint32) chan rabbitmq.EventMessage {
    b.mu.Lock()
    defer b.mu.Unlock()
    
    ch := make(chan rabbitmq.EventMessage, 10)
    b.clients[clientID] = ch
    log.Printf("Added gRPC client: %d", clientID)
    return ch
}

func (b *GrpcBroker) removeClient(clientID uint32) {
    b.mu.Lock()
    defer b.mu.Unlock()
    
    if ch, exists := b.clients[clientID]; exists {
        close(ch)
        delete(b.clients, clientID)
        delete(b.clientInterests, clientID)
        log.Printf("Removed gRPC client: %d. Total clients: %d", clientID, len(b.clients))
    }
}

func (b *GrpcBroker) subscribe(clientID uint32, auctions []uint32) {
    b.mu.Lock()
    defer b.mu.Unlock()
    
    if b.clientInterests == nil {
        b.clientInterests = make(map[uint32][]uint32)
    }
    
    log.Printf("Subscribing Client ID: %d to auctions: %v", clientID, auctions)
    b.clientInterests[clientID] = append(b.clientInterests[clientID], auctions...)
}

func (b *GrpcBroker) unsubscribe(clientID uint32, auctions []uint32) {
    b.mu.Lock()
    defer b.mu.Unlock()
    
    currentInterests := b.clientInterests[clientID]
    updatedInterests := []uint32{}
    
    for _, auction := range currentInterests {
        if !contains(auctions, auction) {
            updatedInterests = append(updatedInterests, auction)
        }
    }
    
    b.clientInterests[clientID] = updatedInterests
    log.Printf("Unsubscribed Client ID: %d from auctions: %v", clientID, auctions)
}

// EventStreamServer implements the gRPC EventStream service.
type EventStreamServer struct {
    pb.UnimplementedEventStreamServer
    broker *GrpcBroker
}

// NewEventStreamServer creates a new EventStreamServer.
func NewEventStreamServer(broker *GrpcBroker) *EventStreamServer {
    return &EventStreamServer{broker: broker}
}

// StreamStart implements the streaming RPC for events.
func (s *EventStreamServer) StreamStart(req *pb.EventRequest, stream pb.EventStream_StreamStartServer) error {
    clientID := req.ClientId
    log.Printf("Client %d started streaming", clientID)
    
    // Register the client
    clientEvents := s.broker.addClient(clientID)
    
    // Ensure cleanup on disconnect
    defer s.broker.removeClient(clientID)
    
    // Stream events to the client
    for {
        select {
        case <-stream.Context().Done():
            // Client disconnected
            log.Printf("Client %d disconnected", clientID)
            return stream.Context().Err()
            
        case message, ok := <-clientEvents:
            if !ok {
                // Channel closed
                return nil
            }
            
            log.Printf("Sending event to client %d: %+v", clientID, message)
            
            // Filter based on event type and client ID
            if message.EventType == "link_pagamento" {
                var event PaymentLink
                if err := json.Unmarshal([]byte(message.Data), &event); err != nil {
                    log.Printf("Error unmarshaling payment link: %v", err)
                    continue
                }
                
                if event.ClientID != clientID {
                    continue
                }
            }
            
            if message.EventType == "status_pagamento" {
                var event PaymentStatus
                if err := json.Unmarshal([]byte(message.Data), &event); err != nil {
                    log.Printf("Error unmarshaling payment status: %v", err)
                    continue
                }
                
                if event.ClientID != clientID {
                    continue
                }
            }
            
            // Send the event to the client
            response := &pb.EventResponse{
                EventType: message.EventType,
                AuctionId: uint32(message.AuctionId),
                Data:      message.Data,
            }
            
            if err := stream.Send(response); err != nil {
                log.Printf("Error sending to client %d: %v", clientID, err)
                return err
            }
        }
    }
}

// AuctionSubscribe implements the subscription RPC.
func (s *EventStreamServer) AuctionSubscribe(ctx context.Context, req *pb.AuctionSubscribeRequest) (*pb.AuctionSubscribeResponse, error) {
    clientID := req.ClientId
    auctions := req.AuctionId
    
    log.Printf("Client %d subscribing to auctions: %v", clientID, auctions)
    
    s.broker.subscribe(clientID, auctions)
    
    return &pb.AuctionSubscribeResponse{
        Success: true,
    }, nil
}

// AuctionUnsubscribe implements the unsubscription RPC.
func (s *EventStreamServer) AuctionUnsubscribe(ctx context.Context, req *pb.AuctionUnsubscribeRequest) (*pb.AuctionUnsubscribeResponse, error) {
    clientID := req.ClientId
    auctions := req.AuctionId
    
    log.Printf("Client %d unsubscribing from auctions: %v", clientID, auctions)
    
    s.broker.unsubscribe(clientID, auctions)
    
    return &pb.AuctionUnsubscribeResponse{
        Success: true,
    }, nil
}