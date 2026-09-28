package models

import (
	"math"
	"math/rand/v2"
	"time"
)

// Order represents a customer's order at a store.
type Order struct {
	ID         [16]byte
	CustomerID [16]byte
	StoreID    [16]byte
	Items      []Item
	OrderedAt  time.Time
	Subtotal   int64 // cents
	TaxPaid    int64 // cents
	OrderTotal int64 // cents
}

// NewOrder creates an order with computed monetary fields.
// All money is int64 cents. Tax = int64(math.Round(float64(subtotal) * taxRate)).
// OrderTotal = Subtotal + TaxPaid.
func NewOrder(rng *rand.Rand, customerID, storeID [16]byte, items []Item, orderedAt time.Time, taxRate float64) Order {
	var subtotal int64
	for _, item := range items {
		subtotal += item.Price
	}
	taxPaid := int64(math.Round(float64(subtotal) * taxRate))
	return Order{
		ID:         UUIDFromRNG(rng),
		CustomerID: customerID,
		StoreID:    storeID,
		Items:      items,
		OrderedAt:  orderedAt,
		Subtotal:   subtotal,
		TaxPaid:    taxPaid,
		OrderTotal: subtotal + taxPaid,
	}
}
