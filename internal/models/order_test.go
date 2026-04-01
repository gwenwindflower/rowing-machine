package models

import (
	"math"
	"testing"
	"time"
)

var testItems = []Item{
	{SKU: "JAF-001", Name: "vanilla ice", Type: Jaffle, Price: 500},
	{SKU: "BEV-001", Name: "tangaroo", Type: Beverage, Price: 350},
	{SKU: "JAF-002", Name: "chai and mighty", Type: Jaffle, Price: 550},
}

func TestNewOrder_Subtotal(t *testing.T) {
	tests := []struct {
		name    string
		items   []Item
		wantSub int64
	}{
		{"single item", testItems[:1], 500},
		{"two items", testItems[:2], 850},
		{"three items", testItems[:3], 1400},
	}

	orderedAt := time.Date(2020, 1, 15, 12, 0, 0, 0, time.UTC)
	custID := [16]byte{1}
	storeID := [16]byte{2}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			rng := newTestRNG(42)
			order := NewOrder(rng, custID, storeID, tt.items, orderedAt, 0.0)
			if order.Subtotal != tt.wantSub {
				t.Errorf("Subtotal = %d, want %d", order.Subtotal, tt.wantSub)
			}
		})
	}
}

func TestNewOrder_TaxComputation(t *testing.T) {
	tests := []struct {
		name    string
		items   []Item
		taxRate float64
		wantTax int64
	}{
		{
			name:    "6% on 1100 cents",
			items:   []Item{{SKU: "X", Name: "x", Price: 1100}},
			taxRate: 0.06,
			wantTax: 66,
		},
		{
			name:    "6.25% on 1100 cents rounds up",
			items:   []Item{{SKU: "X", Name: "x", Price: 1100}},
			taxRate: 0.0625,
			wantTax: int64(math.Round(1100.0 * 0.0625)), // 68.75 -> 69
		},
		{
			name:    "zero tax rate",
			items:   []Item{{SKU: "X", Name: "x", Price: 999}},
			taxRate: 0.0,
			wantTax: 0,
		},
	}

	orderedAt := time.Date(2020, 6, 1, 9, 0, 0, 0, time.UTC)
	custID := [16]byte{1}
	storeID := [16]byte{2}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			rng := newTestRNG(99)
			order := NewOrder(rng, custID, storeID, tt.items, orderedAt, tt.taxRate)
			if order.TaxPaid != tt.wantTax {
				t.Errorf("TaxPaid = %d, want %d", order.TaxPaid, tt.wantTax)
			}
		})
	}
}

func TestNewOrder_TotalEqualsSubtotalPlusTax(t *testing.T) {
	tests := []struct {
		name    string
		items   []Item
		taxRate float64
	}{
		{"standard items", testItems, 0.08},
		{"single item low tax", testItems[:1], 0.01},
		{"empty items", nil, 0.10},
	}

	orderedAt := time.Date(2021, 3, 20, 14, 30, 0, 0, time.UTC)
	custID := [16]byte{3}
	storeID := [16]byte{4}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			rng := newTestRNG(7)
			order := NewOrder(rng, custID, storeID, tt.items, orderedAt, tt.taxRate)
			if order.OrderTotal != order.Subtotal+order.TaxPaid {
				t.Errorf("OrderTotal(%d) != Subtotal(%d) + TaxPaid(%d)",
					order.OrderTotal, order.Subtotal, order.TaxPaid)
			}
		})
	}
}

func TestNewOrder_EmptyItems(t *testing.T) {
	rng := newTestRNG(1)
	orderedAt := time.Date(2020, 1, 1, 0, 0, 0, 0, time.UTC)
	order := NewOrder(rng, [16]byte{}, [16]byte{}, nil, orderedAt, 0.06)

	if order.Subtotal != 0 {
		t.Errorf("Subtotal = %d, want 0", order.Subtotal)
	}
	if order.TaxPaid != 0 {
		t.Errorf("TaxPaid = %d, want 0", order.TaxPaid)
	}
	if order.OrderTotal != 0 {
		t.Errorf("OrderTotal = %d, want 0", order.OrderTotal)
	}
}

func TestNewOrder_DeterministicUUID(t *testing.T) {
	orderedAt := time.Date(2020, 5, 5, 10, 0, 0, 0, time.UTC)
	custID := [16]byte{10}
	storeID := [16]byte{20}
	items := testItems[:1]

	rng1 := newTestRNG(123)
	order1 := NewOrder(rng1, custID, storeID, items, orderedAt, 0.05)

	rng2 := newTestRNG(123)
	order2 := NewOrder(rng2, custID, storeID, items, orderedAt, 0.05)

	if order1.ID != order2.ID {
		t.Errorf("same seed produced different UUIDs: %v vs %v", order1.ID, order2.ID)
	}

	rng3 := newTestRNG(456)
	order3 := NewOrder(rng3, custID, storeID, items, orderedAt, 0.05)

	if order1.ID == order3.ID {
		t.Error("different seeds produced the same UUID")
	}
}

func TestNewOrder_FieldAssignment(t *testing.T) {
	rng := newTestRNG(55)
	custID := [16]byte{0xAA}
	storeID := [16]byte{0xBB}
	orderedAt := time.Date(2022, 12, 25, 8, 0, 0, 0, time.UTC)
	items := testItems[:2]

	order := NewOrder(rng, custID, storeID, items, orderedAt, 0.07)

	if order.CustomerID != custID {
		t.Errorf("CustomerID = %v, want %v", order.CustomerID, custID)
	}
	if order.StoreID != storeID {
		t.Errorf("StoreID = %v, want %v", order.StoreID, storeID)
	}
	if !order.OrderedAt.Equal(orderedAt) {
		t.Errorf("OrderedAt = %v, want %v", order.OrderedAt, orderedAt)
	}
	if len(order.Items) != 2 {
		t.Errorf("Items length = %d, want 2", len(order.Items))
	}
}
