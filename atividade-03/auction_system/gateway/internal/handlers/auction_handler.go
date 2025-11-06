package handlers

import (
	"encoding/json"
	"io"
	"net/http"

	"gateway/internal/service"
)

type AuctionHandler struct {
	service *service.AuctionService
}

func NewAuctionHandler(svc *service.AuctionService) *AuctionHandler {
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

	resp, err := h.service.CreateAuction(body)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	w.Write(resp)
}
