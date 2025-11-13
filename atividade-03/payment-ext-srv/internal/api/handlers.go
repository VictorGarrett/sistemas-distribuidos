package api

import (
	"net/http"
	"payment-ext-srv/internal"
	"payment-ext-srv/internal/models"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/log"
	"github.com/google/uuid"
)

func HandleNewTransaction(pm *internal.TransactionManager) fiber.Handler {
	return func(c *fiber.Ctx) error {
		var newTransaction models.NewTransactionRequest
		if err := c.BodyParser(&newTransaction); err != nil {
			log.Errorf("Failed to parse request body to newTransactionRequest type: %v")
			return c.SendStatus(fiber.ErrBadRequest.Code)
		}
		if newTransaction.Amount <= 0.0 {
			log.Error("Payment Amount must be positive")
			return c.SendStatus(fiber.ErrBadRequest.Code)
		}

		res := pm.CreateNewTransaction(&newTransaction)

		return c.Status(fiber.StatusCreated).JSON(res)
	}
}

func HandleTransactionPay(pm *internal.TransactionManager) fiber.Handler {
	return func(c *fiber.Ctx) error {
		tid, err := uuid.Parse(c.Params("tid"))
		if err != nil {
			log.Errorf("Invalid Query parameter \"tid\": %v", err)
			return c.Status(fiber.ErrBadRequest.Code).JSON(fiber.Map{
				"error": "Invalid query parameter \"tid\"",
			})
		}

		if err = pm.SetPaid(tid); err != nil {
			log.Errorf("Failed to pay transaction of ID %s: %v", tid.String(), err)
			return c.Status(fiber.ErrBadRequest.Code).JSON(fiber.Map{
				"error": "transaction not found for id " + tid.String(),
			})
		}

		transaction := pm.GetTransaction(tid)
		http.Post(transaction.Callback, "application/json", nil)

		return c.SendStatus(fiber.StatusOK)
	}
}

func HandleGetTransaction(pm *internal.TransactionManager) fiber.Handler {
	return func(c *fiber.Ctx) error {
		transactions := pm.GetAllTransactions()
		return c.Status(fiber.StatusOK).JSON(transactions)
	}
}
