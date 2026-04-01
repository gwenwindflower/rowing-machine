package market

import (
	"math"
	"math/rand/v2"
	"testing"
	"time"

	"rowing-machine/internal/models"
)

func newTestRNG(seed uint64) *rand.Rand {
	return rand.New(rand.NewPCG(seed, 0))
}

func testStore() models.Store {
	return models.Store{
		ID:             [16]byte{1},
		Name:           "Test Store",
		BasePopularity: 0.90,
		OpenedDay:      0,
		TaxRate:        0.06,
		TAMBase:        10,
	}
}

func TestPenetration_NegativeDay(t *testing.T) {
	if got := Penetration(-1); got != 0 {
		t.Errorf("Penetration(-1) = %f, want 0", got)
	}
}

func TestPenetration_DayZero(t *testing.T) {
	got := Penetration(0)
	// ln(1 + 0*(e-1)) = ln(1) = 0
	if got != 0 {
		t.Errorf("Penetration(0) = %f, want 0", got)
	}
}

func TestPenetration_Day365(t *testing.T) {
	got := Penetration(365)
	// pct = 1.0, ln(1 + 1*(e-1)) = ln(e) = 1.0
	if math.Abs(got-1.0) > 1e-10 {
		t.Errorf("Penetration(365) = %f, want 1.0", got)
	}
}

func TestPenetration_Day180(t *testing.T) {
	got := Penetration(180)
	// pct = 180/365 ≈ 0.4932
	// ln(1 + 0.4932*(e-1)) ≈ 0.62
	if got < 0.58 || got > 0.66 {
		t.Errorf("Penetration(180) = %f, want approximately 0.62", got)
	}
}

func TestPenetration_MonotonicallyIncreasing(t *testing.T) {
	prev := Penetration(0)
	for d := 1; d <= 365; d++ {
		cur := Penetration(d)
		if cur < prev {
			t.Errorf("Penetration not monotonically increasing: day %d (%f) < day %d (%f)", d, cur, d-1, prev)
		}
		prev = cur
	}
}

func TestPenetration_NoDiscontinuity(t *testing.T) {
	// Verify smooth transition around day 7 (Python bug #3 had a discontinuity here)
	p6 := Penetration(6)
	p7 := Penetration(7)
	p8 := Penetration(8)
	if !(p6 < p7 && p7 < p8) {
		t.Errorf("Discontinuity around day 7: Penetration(6)=%f, (7)=%f, (8)=%f", p6, p7, p8)
	}
}

func TestNewMarket_CustomerCount(t *testing.T) {
	store := testStore()
	rng := newTestRNG(42)
	m := NewMarket(store, rng, 100)

	// TAMBase=10, scale=100, total=1000
	// PersonaMix weights sum to 1.0, so all 1000 should be created
	// (due to int truncation, some may be lost: 0.25*1000=250, 0.10*1000=100, etc.)
	expected := 0
	for _, pw := range models.PersonaMix {
		expected += int(pw.Weight * float64(1000))
	}

	if len(m.allCustomers) != expected {
		t.Errorf("NewMarket created %d customers, want %d", len(m.allCustomers), expected)
	}
	if len(m.addressableCustomers) != expected {
		t.Errorf("addressableCustomers = %d, want %d", len(m.addressableCustomers), expected)
	}
	if len(m.ActiveCustomers) != 0 {
		t.Errorf("ActiveCustomers = %d, want 0", len(m.ActiveCustomers))
	}
}

func TestSimDay_BeforeStoreOpens(t *testing.T) {
	store := testStore()
	store.OpenedDay = 100
	rng := newTestRNG(42)
	m := NewMarket(store, rng, 10)

	day := DayInfo{
		Index:     50,
		Date:      time.Date(2018, 10, 21, 0, 0, 0, 0, time.UTC),
		IsWeekend: false,
		Season:    models.Fall,
		Effect:    1.0,
		OpensAt:   420,
		ClosesAt:  1200,
	}
	seen := make(map[[16]byte]bool)
	result := m.SimDay(day, seen)

	if len(result.Orders) != 0 {
		t.Errorf("SimDay before store open produced %d orders, want 0", len(result.Orders))
	}
	if len(result.Tweets) != 0 {
		t.Errorf("SimDay before store open produced %d tweets, want 0", len(result.Tweets))
	}
}

func TestSimDay_Deterministic(t *testing.T) {
	// Run the same day twice with the same seed and verify identical results
	run := func() DayResult {
		store := testStore()
		rng := newTestRNG(42)
		m := NewMarket(store, rng, 50)

		day := DayInfo{
			Index:     30,
			Date:      time.Date(2018, 10, 1, 0, 0, 0, 0, time.UTC),
			IsWeekend: false,
			Season:    models.Fall,
			Effect:    0.9,
			OpensAt:   420,
			ClosesAt:  1200,
		}
		seen := make(map[[16]byte]bool)
		return m.SimDay(day, seen)
	}

	r1 := run()
	r2 := run()

	if len(r1.Orders) != len(r2.Orders) {
		t.Fatalf("Non-deterministic: run1 orders=%d, run2 orders=%d", len(r1.Orders), len(r2.Orders))
	}
	for i := range r1.Orders {
		if r1.Orders[i].Order.ID != r2.Orders[i].Order.ID {
			t.Errorf("Order %d ID mismatch", i)
		}
	}
	if len(r1.Tweets) != len(r2.Tweets) {
		t.Fatalf("Non-deterministic: run1 tweets=%d, run2 tweets=%d", len(r1.Tweets), len(r2.Tweets))
	}
}

func TestCustomerActivation_IncreasesOverTime(t *testing.T) {
	store := testStore()
	rng := newTestRNG(42)
	m := NewMarket(store, rng, 100)

	m.ActivateCustomers(0)
	count0 := len(m.ActiveCustomers)

	m.ActivateCustomers(30)
	count30 := len(m.ActiveCustomers)

	m.ActivateCustomers(180)
	count180 := len(m.ActiveCustomers)

	if count30 <= count0 {
		t.Errorf("Active at day 30 (%d) should exceed day 0 (%d)", count30, count0)
	}
	if count180 <= count30 {
		t.Errorf("Active at day 180 (%d) should exceed day 30 (%d)", count180, count30)
	}
}

func TestSimDay_ItemIDsGenerated(t *testing.T) {
	store := testStore()
	rng := newTestRNG(42)
	m := NewMarket(store, rng, 50)

	day := DayInfo{
		Index:     30,
		Date:      time.Date(2018, 10, 1, 0, 0, 0, 0, time.UTC),
		IsWeekend: false,
		Season:    models.Fall,
		Effect:    0.9,
		OpensAt:   420,
		ClosesAt:  1200,
	}
	seen := make(map[[16]byte]bool)
	result := m.SimDay(day, seen)

	for i, owi := range result.Orders {
		if len(owi.ItemIDs) != len(owi.Order.Items) {
			t.Errorf("Order %d: %d item IDs for %d items", i, len(owi.ItemIDs), len(owi.Order.Items))
		}
	}
}
