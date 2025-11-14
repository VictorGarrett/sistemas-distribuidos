package api

import (
	"fmt"
	"payment-srv/internal"

	"github.com/gofiber/fiber/v2"
	"github.com/google/uuid"
)

type Payment struct {
	Pid string `json:"pid"`
}

func UpdatePayment(pm *internal.PaymentManager) fiber.Handler {
	return func(c *fiber.Ctx) error {

		var payment Payment
		if err := c.BodyParser(&payment); err != nil {
			fmt.Printf("Failed to parse request body to Payment type: %v\n", err)
			return c.Status(fiber.ErrBadRequest.Code).JSON(fiber.Map{
				"error": "Failed",
			})
		}

		fmt.Printf("Received request: %v\n", c.Body())
		paymentUUID, err := uuid.Parse(payment.Pid)

		if err != nil {
			fmt.Printf("Failed to extract paymentID from route: %v\n", err)
			return c.Status(fiber.ErrBadRequest.Code).JSON(fiber.Map{
				"error": "Failed",
			})
		}

		err = pm.SetPaid(paymentUUID)
		if err != nil {
			fmt.Printf("Failed to pay request: %v\n", err)
			return c.Status(fiber.ErrInternalServerError.Code).JSON(fiber.Map{
				"error": "Failed",
			})
		}

		return c.SendStatus(fiber.StatusOK)
	}
}
