package api

import (
	"payment-ext-srv/internal"
	"payment-ext-srv/internal/models"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/log"
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
		return nil
	}
}
