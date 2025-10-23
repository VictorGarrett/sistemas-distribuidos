package api

import (
	"payment-ext-srv/internal"
	"payment-ext-srv/internal/models"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/log"
)

func HandleNewPayment(pm *internal.PaymentManager) fiber.Handler {
	return func(c *fiber.Ctx) error {
		var newPayment models.NewPaymentRequest
		if err := c.BodyParser(&newPayment); err != nil {
			log.Errorf("Failed to parse request body to NewPaymentRequest type: %v")
			return c.SendStatus(fiber.ErrBadRequest.Code)
		}
		if newPayment.Amount <= 0.0 {
			log.Error("Payment Amount must be positive")
			return c.SendStatus(fiber.ErrBadRequest.Code)
		}

		res := pm.CreateNewPayment(&newPayment)

		return c.Status(fiber.StatusCreated).JSON(res)
	}
}

func HandlePay(pm *internal.PaymentManager) fiber.Handler {
	return func(c *fiber.Ctx) error {
		return nil
	}
}
