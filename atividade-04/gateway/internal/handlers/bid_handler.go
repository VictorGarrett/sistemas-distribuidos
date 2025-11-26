package handlers

import (
	//"encoding/json"
	"io"
	"net/http"
	"gateway/internal/services"
)

type BidHandler struct {
	service *services.BidService
}

func NewBidHandler(svc *services.BidService) *BidHandler {
	return &BidHandler{service: svc}
}

func (h *BidHandler) HandleBids(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case http.MethodPost:
		h.createBid(w, r)
	default:
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
	}
}


func (h *BidHandler) createBid(w http.ResponseWriter, r *http.Request) {
	body, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, "Failed to read body", http.StatusBadRequest)
		return
	}
	defer r.Body.Close()

	resp, err := h.service.CreateBid(body)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	w.Write(resp)
}
