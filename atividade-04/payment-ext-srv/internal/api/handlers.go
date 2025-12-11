package api

import (
	"fmt"
	"log"
	"payment-ext-srv/internal"
	"payment-ext-srv/internal/models"
	"payment-ext-srv/internal/services"

	"github.com/gofiber/fiber/v2"
	"github.com/google/uuid"
)

type Payment struct {
	Pid string `json:"pid"`
}

func HandleNewTransaction(pm *internal.TransactionManager) fiber.Handler {
	return func(c *fiber.Ctx) error {
		fmt.Println("Received new transaction request")
		var newTransaction models.NewTransactionRequest
		if err := c.BodyParser(&newTransaction); err != nil {
			fmt.Printf("Failed to parse request body to newTransactionRequest type: %v\n", err)
			return c.SendStatus(fiber.ErrBadRequest.Code)
		}
		if newTransaction.Amount <= 0.0 {
			fmt.Println("Payment Amount must be positive")
			return c.SendStatus(fiber.ErrBadRequest.Code)
		}

		res := pm.CreateNewTransaction(&newTransaction)

		fmt.Println(res)

		return c.Status(fiber.StatusCreated).JSON(res)
	}
}

func HandleTransactionPay(
	pm *internal.TransactionManager,
	paymentIntSrv *services.PaymentInternalService,
) fiber.Handler {
	return func(c *fiber.Ctx) error {
		tid, err := uuid.Parse(c.Params("tid"))
		if err != nil {
			fmt.Printf("Invalid Query parameter \"tid\": %v\n", err)
			return c.Status(fiber.ErrBadRequest.Code).JSON(fiber.Map{
				"error": "Invalid query parameter \"tid\"",
			})
		}

		if err = pm.SetPaid(tid); err != nil {
			fmt.Printf("Failed to pay transaction of ID %s: %v\n", tid.String(), err)
			return c.Status(fiber.ErrBadRequest.Code).JSON(fiber.Map{
				"error": "transaction not found for id " + tid.String(),
			})
		}

		fmt.Printf("getting transaction: %s/\n", tid.String())

		err = paymentIntSrv.UpdatePayment(tid.String())
		if err != nil {
			log.Printf("Waaaaaaa... Failed to update Payment of id %s", tid.String())
		}

		return c.SendStatus(fiber.StatusOK)
	}
}

func HandleGetTransaction(pm *internal.TransactionManager) fiber.Handler {
	return func(c *fiber.Ctx) error {
		transactions := pm.GetAllTransactions()
		return c.Status(fiber.StatusOK).JSON(transactions)
	}
}
