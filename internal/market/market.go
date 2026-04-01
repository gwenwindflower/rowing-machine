package market

import (
	"math"
	"math/rand/v2"
	"time"

	"rowing-machine/internal/catalog"
	"rowing-machine/internal/models"
)

// DayInfo holds the day-level information needed by the market simulation.
// This is a market-local type to avoid an import cycle with the simulation package.
type DayInfo struct {
	Index     int
	Date      time.Time
	IsWeekend bool
	Season    models.Season
	Effect    float64
	OpensAt   int
	ClosesAt  int
}

// OrderWithItemIDs pairs an order with deterministically generated UUIDs for each line item.
type OrderWithItemIDs struct {
	Order   models.Order
	ItemIDs [][16]byte
}

// DayResult holds all simulation output for a single day in a single market.
type DayResult struct {
	Orders       []OrderWithItemIDs
	Tweets       []models.Tweet
	NewCustomers []models.Customer
}

// Market manages a store's customer pool, activation, and daily simulation.
type Market struct {
	Store                models.Store
	RNG                  *rand.Rand
	allCustomers         []models.Customer
	addressableCustomers []models.Customer
	ActiveCustomers      []models.Customer
}

// NewMarket creates a market for the given store with a customer pool sized by scale.
// Customer creation order and shuffle are deterministic via the provided RNG.
func NewMarket(store models.Store, rng *rand.Rand, scale int) *Market {
	totalCustomers := store.TAMBase * scale

	var allCustomers []models.Customer
	for _, pw := range models.PersonaMix {
		count := int(pw.Weight * float64(totalCustomers))
		for range count {
			c := models.Customer{
				ID:             models.UUIDFromRNG(rng),
				StoreID:        store.ID,
				Name:           catalog.GenerateName(rng),
				FavoriteNumber: rng.IntN(100) + 1,
				FanLevel:       rng.IntN(5) + 1,
				Persona:        pw.NewPersona(),
			}
			allCustomers = append(allCustomers, c)
		}
	}

	// Shuffle for random activation order
	rng.Shuffle(len(allCustomers), func(i, j int) {
		allCustomers[i], allCustomers[j] = allCustomers[j], allCustomers[i]
	})

	// All start as addressable, none active
	addressable := make([]models.Customer, len(allCustomers))
	copy(addressable, allCustomers)

	return &Market{
		Store:                store,
		RNG:                  rng,
		allCustomers:         allCustomers,
		addressableCustomers: addressable,
		ActiveCustomers:      nil,
	}
}

// Penetration returns the fraction of the customer pool that should be active
// after the given number of days since store opening.
// Uses a smooth logarithmic curve: min(ln(1 + pct*(e-1)), 1.0)
// This fixes Python bug #3 (discontinuity at day 7).
func Penetration(daysSinceOpen int) float64 {
	if daysSinceOpen < 0 {
		return 0
	}
	pct := math.Min(float64(daysSinceOpen)/365.0, 1.0)
	return math.Min(math.Log(1+pct*(math.E-1)), 1.0)
}

// ActivateCustomers moves customers from addressable to active based on the
// penetration curve for the current day.
func (m *Market) ActivateCustomers(dayIndex int) {
	if !m.Store.IsOpen(dayIndex) {
		return
	}
	daysSinceOpen := m.Store.DaysSinceOpen(dayIndex)
	pen := Penetration(daysSinceOpen)
	desired := int(pen * float64(len(m.allCustomers)))
	toAdd := desired - len(m.ActiveCustomers)

	if toAdd <= 0 {
		return
	}
	if toAdd > len(m.addressableCustomers) {
		toAdd = len(m.addressableCustomers)
	}

	// Pop from end of addressable into active
	start := len(m.addressableCustomers) - toAdd
	m.ActiveCustomers = append(m.ActiveCustomers, m.addressableCustomers[start:]...)
	m.addressableCustomers = m.addressableCustomers[:start]
}

// SimDay simulates one day for this market. seenCustomers tracks which customers
// have placed at least one order across the entire simulation (for first-order tracking).
func (m *Market) SimDay(day DayInfo, seenCustomers map[[16]byte]bool) DayResult {
	if !m.Store.IsOpen(day.Index) {
		return DayResult{}
	}
	m.ActivateCustomers(day.Index)

	pBuySeason := m.Store.PBuy(day.Effect)
	var result DayResult

	for i := range m.ActiveCustomers {
		c := &m.ActiveCustomers[i]

		pBuyPersona := c.Persona.PBuyPersona(day.IsWeekend, day.Season, c.FavoriteNumber)
		pBuy := math.Sqrt(pBuySeason * pBuyPersona)

		if m.RNG.Float64() >= pBuy {
			continue
		}

		// Customer wants to buy — generate order
		orderMinute := c.Persona.OrderMinute(m.RNG, c.FavoriteNumber)

		// Check store hours
		if !m.Store.IsOpenAt(orderMinute, day.OpensAt, day.ClosesAt) {
			continue
		}

		// Select items
		items := c.Persona.OrderItems(m.RNG, c.FavoriteNumber, func(itemType models.ItemType, count int) []models.Item {
			return catalog.RandomItems(m.RNG, itemType, count)
		})
		if len(items) == 0 {
			continue
		}

		// Create order
		orderedAt := day.Date.Add(time.Duration(orderMinute) * time.Minute)
		order := models.NewOrder(m.RNG, c.ID, m.Store.ID, items, orderedAt, m.Store.TaxRate)

		// Generate deterministic item IDs from the market RNG
		itemIDs := make([][16]byte, len(items))
		for j := range items {
			itemIDs[j] = models.UUIDFromRNG(m.RNG)
		}

		result.Orders = append(result.Orders, OrderWithItemIDs{
			Order:   order,
			ItemIDs: itemIDs,
		})

		// Track new customer
		if !seenCustomers[c.ID] {
			seenCustomers[c.ID] = true
			result.NewCustomers = append(result.NewCustomers, *c)
		}

		// Maybe tweet (checked AFTER order, per Go version spec)
		if m.RNG.Float64() < c.Persona.PTweet() {
			tweet := models.NewTweet(m.RNG, c.ID, c.FanLevel, items, orderedAt)
			result.Tweets = append(result.Tweets, tweet)
		}
	}

	return result
}
