package models

import (
	"math"
	"math/rand/v2"
)

// Customer represents a simulated customer with behavioral traits.
type Customer struct {
	ID             [16]byte
	StoreID        [16]byte // which store they frequent
	Name           string
	FavoriteNumber int // 1-100, drives variance in buy probability
	FanLevel       int // 1-5, drives tweet sentiment
	Persona        Persona
}

// Persona defines the behavioral interface for customer types.
type Persona interface {
	// Name returns the persona type name (e.g. "Commuter").
	Name() string
	// PBuyPersona returns the persona-specific purchase probability.
	PBuyPersona(isWeekend bool, season Season, favoriteNumber int) float64
	// PTweet returns the tweet probability for this persona.
	PTweet() float64
	// OrderMinute samples an order time from the persona's distribution.
	// Returns minutes from midnight. Result is clamped to >= 0.
	OrderMinute(rng *rand.Rand, favoriteNumber int) int
	// OrderItems selects items for an order using the getItems function.
	// getItems(itemType, count) returns count random items of that type.
	OrderItems(rng *rand.Rand, favoriteNumber int, getItems func(ItemType, int) []Item) []Item
}

// PersonaWeight pairs a persona factory with its market share weight.
type PersonaWeight struct {
	NewPersona func() Persona // factory function
	Weight     float64
}

// PersonaMix defines the distribution of personas in the customer pool.
var PersonaMix = []PersonaWeight{
	{func() Persona { return &Commuter{} }, 0.25},
	{func() Persona { return &RemoteWorker{} }, 0.25},
	{func() Persona { return &BrunchCrowd{} }, 0.10},
	{func() Persona { return &Student{} }, 0.20},
	{func() Persona { return &Casuals{} }, 0.10},
	{func() Persona { return &HealthNut{} }, 0.10},
}

// sampleMinute draws from a normal distribution and clamps to >= 0.
func sampleMinute(rng *rand.Rand, mu, sigma float64) int {
	minute := int(math.Round(rng.NormFloat64()*sigma + mu))
	if minute < 0 {
		minute = 0
	}
	return minute
}

// --- Commuter (25% of market) ---

// Commuter represents a weekday morning commuter who grabs a quick beverage.
type Commuter struct{}

func (c *Commuter) Name() string { return "Commuter" }

func (c *Commuter) PBuyPersona(isWeekend bool, _ Season, favoriteNumber int) float64 {
	if isWeekend {
		return 0.001
	}
	return 0.5 + (float64(favoriteNumber)/100.0)*0.3
}

func (c *Commuter) PTweet() float64 { return 0.2 }

// OrderMinute uses N(450, 30) = 7:30 AM. This is the fix for Python bug #2
// where the original used N(60, 30) = 1:00 AM.
func (c *Commuter) OrderMinute(rng *rand.Rand, _ int) int {
	return sampleMinute(rng, 450, 30)
}

func (c *Commuter) OrderItems(_ *rand.Rand, _ int, getItems func(ItemType, int) []Item) []Item {
	return getItems(Beverage, 1)
}

// --- RemoteWorker (25% of market) ---

// RemoteWorker represents someone working from home with varied ordering patterns.
type RemoteWorker struct{}

func (r *RemoteWorker) Name() string { return "RemoteWorker" }

func (r *RemoteWorker) PBuyPersona(isWeekend bool, _ Season, favoriteNumber int) float64 {
	if isWeekend {
		return 0.001
	}
	return (float64(favoriteNumber) / 100.0) * 0.4
}

func (r *RemoteWorker) PTweet() float64 { return 0.01 }

func (r *RemoteWorker) OrderMinute(rng *rand.Rand, _ int) int {
	return sampleMinute(rng, 420, 180)
}

func (r *RemoteWorker) OrderItems(rng *rand.Rand, _ int, getItems func(ItemType, int) []Item) []Item {
	items := getItems(Beverage, 1)
	if rng.Float64() < 0.3 {
		items = append(items, getItems(Beverage, 1)...)
	}
	if rng.Float64() < 0.3 {
		items = append(items, getItems(Jaffle, 1)...)
	}
	return items
}

// --- BrunchCrowd (10% of market) ---

// BrunchCrowd represents weekend-only customers who order generously.
type BrunchCrowd struct{}

func (b *BrunchCrowd) Name() string { return "BrunchCrowd" }

func (b *BrunchCrowd) PBuyPersona(isWeekend bool, _ Season, favoriteNumber int) float64 {
	if !isWeekend {
		return 0
	}
	return 0.2 + (float64(favoriteNumber)/100.0)*0.2
}

func (b *BrunchCrowd) PTweet() float64 { return 0.8 }

func (b *BrunchCrowd) OrderMinute(rng *rand.Rand, favoriteNumber int) int {
	mu := 300 + float64(favoriteNumber-50)/50.0*120
	return sampleMinute(rng, mu, 120)
}

func (b *BrunchCrowd) OrderItems(_ *rand.Rand, favoriteNumber int, getItems func(ItemType, int) []Item) []Item {
	count := 1 + favoriteNumber/20
	items := getItems(Jaffle, count)
	items = append(items, getItems(Beverage, count)...)
	return items
}

// --- Student (20% of market) ---

// Student represents a seasonal customer absent in summer.
type Student struct{}

func (s *Student) Name() string { return "Student" }

func (s *Student) PBuyPersona(_ bool, season Season, favoriteNumber int) float64 {
	if season == Summer {
		return 0
	}
	return 0.1 + (float64(favoriteNumber)/100.0)*0.4
}

func (s *Student) PTweet() float64 { return 0.8 }

func (s *Student) OrderMinute(rng *rand.Rand, _ int) int {
	return sampleMinute(rng, 540, 120)
}

func (s *Student) OrderItems(rng *rand.Rand, _ int, getItems func(ItemType, int) []Item) []Item {
	items := getItems(Beverage, 1)
	if rng.Float64() < 0.5 {
		items = append(items, getItems(Jaffle, 1)...)
	}
	return items
}

// --- Casuals (10% of market) ---

// Casuals represents irregular customers with random ordering patterns.
type Casuals struct{}

func (c *Casuals) Name() string { return "Casuals" }

func (c *Casuals) PBuyPersona(_ bool, _ Season, _ int) float64 {
	return 0.1
}

func (c *Casuals) PTweet() float64 { return 0.1 }

func (c *Casuals) OrderMinute(rng *rand.Rand, _ int) int {
	return sampleMinute(rng, 300, 120)
}

func (c *Casuals) OrderItems(rng *rand.Rand, _ int, getItems func(ItemType, int) []Item) []Item {
	beverageCount := int(rng.Float64() * 10 / 3)
	jaffleCount := int(rng.Float64() * 10 / 3)
	var items []Item
	if beverageCount > 0 {
		items = append(items, getItems(Beverage, beverageCount)...)
	}
	if jaffleCount > 0 {
		items = append(items, getItems(Jaffle, jaffleCount)...)
	}
	return items
}

// --- HealthNut (10% of market) ---

// HealthNut represents a health-conscious customer most active in summer.
type HealthNut struct{}

func (h *HealthNut) Name() string { return "HealthNut" }

func (h *HealthNut) PBuyPersona(_ bool, season Season, favoriteNumber int) float64 {
	if season == Summer {
		return 0.1 + (float64(favoriteNumber)/100.0)*0.4
	}
	return 0.2
}

func (h *HealthNut) PTweet() float64 { return 0.6 }

func (h *HealthNut) OrderMinute(rng *rand.Rand, _ int) int {
	return sampleMinute(rng, 300, 120)
}

func (h *HealthNut) OrderItems(_ *rand.Rand, _ int, getItems func(ItemType, int) []Item) []Item {
	return getItems(Beverage, 1)
}
