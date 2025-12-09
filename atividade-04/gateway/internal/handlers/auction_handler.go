package handlers

import (
	"encoding/json"
	"io"
	"net/http"
	"gateway/internal/services"
)



type CreateAuctionPayload struct {
    ItemName        string `json:"itemName"`
    StartTimestamp  uint64 `json:"startTimestamp"`
    EndTimestamp    uint64 `json:"endTimestamp"`
}

type AuctionHandler struct {
	service *services.AuctionService
}

func NewAuctionHandler(svc *services.AuctionService) *AuctionHandler {
	return &AuctionHandler{service: svc}
}

func (h *AuctionHandler) HandleAuctions(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodGet:
		h.getActiveAuctions(w, r)
	case http.MethodPost:
		h.createAuction(w, r)
	default:
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
	}
}

func (h *AuctionHandler) getActiveAuctions(w http.ResponseWriter, r *http.Request) {
	auctions, err := h.service.GetActiveAuctions()
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(auctions)
}

func (h *AuctionHandler) createAuction(w http.ResponseWriter, r *http.Request) {
    body, err := io.ReadAll(r.Body)
    if err != nil {
        http.Error(w, "Failed to read body", http.StatusBadRequest)
        return
    }
    defer r.Body.Close()

    // Parse incoming JSON
    var payload struct {
        ItemName        string `json:"itemName"`
        StartTimestamp  uint64 `json:"startTimestamp"`
        EndTimestamp    uint64 `json:"endTimestamp"`
    }

    if err := json.Unmarshal(body, &payload); err != nil {
        http.Error(w, "Invalid JSON format", http.StatusBadRequest)
        return
    }

    // Call gRPC service correctly
    resp, err := h.service.CreateAuction(
        payload.ItemName,
        payload.StartTimestamp,
        payload.EndTimestamp,
    )
    if err != nil {
        http.Error(w, err.Error(), http.StatusInternalServerError)
        return
    }

    // Marshal resp into JSON
    out, err := json.Marshal(resp)
    if err != nil {
        http.Error(w, "Failed to serialize response", http.StatusInternalServerError)
        return
    }

    w.Header().Set("Content-Type", "application/json")
    w.WriteHeader(http.StatusCreated)
    w.Write(out)
}
