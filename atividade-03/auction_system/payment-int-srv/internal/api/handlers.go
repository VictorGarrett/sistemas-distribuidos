package api

import (
	"payment-srv/internal"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/log"
	"github.com/google/uuid"
)

func UpdatePayment(pm *internal.PaymentManager) fiber.Handler {
	return func(c *fiber.Ctx) error {
		paymentIDStr := c.Params("payment-id", "")
		paymentUUID, err := uuid.Parse(paymentIDStr)

		if err != nil {
			log.Errorf("Failed to extract paymentID from route: %v", err)
			return c.Status(fiber.ErrBadRequest.Code).JSON(fiber.Map{
				"error": "Failed",
			})
		}

		err = pm.SetPaid(paymentUUID)
		if err != nil {
			log.Errorf("Failed to pay request: %v", err)
			return c.Status(fiber.ErrInternalServerError.Code).JSON(fiber.Map{
				"error": "Failed",
			})
		}

		return c.SendStatus(fiber.StatusOK)
	}
}
