package api

import (
	"payment-ext-srv/internal"

	"github.com/gofiber/fiber/v2"
)

func HandleNewPayment(pm *internal.PaymentManager) fiber.Handler {
	return func(c *fiber.Ctx) error {
		return nil
	}
}

func HandlePay(pm *internal.PaymentManager) fiber.Handler {
	return func(c *fiber.Ctx) error {
		return nil
	}
}
