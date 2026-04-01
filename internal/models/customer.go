package models

import (
	"math"
	"math/rand/v2"
)

// GuildRank represents a customer's standing within the Arcanum Collective.
type GuildRank int

const (
	Initiate GuildRank = iota
	Journeyman
	Adept
	Master
)

func (g GuildRank) String() string {
	switch g {
	case Initiate:
		return "initiate"
	case Journeyman:
		return "journeyman"
	case Adept:
		return "adept"
	case Master:
		return "master"
	default:
		return "unknown"
	}
}

// GuildRankFromOrders returns the guild rank based on total order count.
func GuildRankFromOrders(orderCount int) GuildRank {
	switch {
	case orderCount >= 30:
		return Master
	case orderCount >= 15:
		return Adept
	case orderCount >= 5:
		return Journeyman
	default:
		return Initiate
	}
}

// Customer represents a simulated patron with behavioral traits.
type Customer struct {
	ID             [16]byte
	StoreID        [16]byte // which guild hall they frequent
	Name           string
	FavoriteNumber int // 1-100, drives variance in buy probability
	FanLevel       int // 1-5, drives sparrow sentiment
	Persona        Persona
	GuildRank      GuildRank
}

// Persona defines the behavioral interface for customer archetypes.
type Persona interface {
	// Name returns the archetype name (e.g. "Courier").
	Name() string
	// PBuyPersona returns the persona-specific purchase probability.
	PBuyPersona(isWeekend bool, season Season, favoriteNumber int) float64
	// PTweet returns the sparrow probability for this persona.
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
	{func() Persona { return &Courier{} }, 0.25},
	{func() Persona { return &Artificer{} }, 0.25},
	{func() Persona { return &FeastReveler{} }, 0.10},
	{func() Persona { return &Apprentice{} }, 0.20},
	{func() Persona { return &Wanderer{} }, 0.10},
	{func() Persona { return &Herbalist{} }, 0.10},
}

// sampleMinute draws from a normal distribution and clamps to >= 0.
func sampleMinute(rng *rand.Rand, mu, sigma float64) int {
	minute := int(math.Round(rng.NormFloat64()*sigma + mu))
	if minute < 0 {
		minute = 0
	}
	return minute
}

// solidItemType returns Weapon or Armor with equal probability.
func solidItemType(rng *rand.Rand) ItemType {
	if rng.Float64() < 0.5 {
		return Armor
	}
	return Weapon
}

// --- Courier (25% of market) ---

// Courier represents a guild messenger who grabs a quick elixir in the morning.
type Courier struct{}

func (c *Courier) Name() string { return "Courier" }

func (c *Courier) PBuyPersona(isWeekend bool, _ Season, favoriteNumber int) float64 {
	if isWeekend {
		return 0.001
	}
	return 0.5 + (float64(favoriteNumber)/100.0)*0.3
}

func (c *Courier) PTweet() float64 { return 0.2 }

// OrderMinute uses N(450, 30) = 7:30 AM.
func (c *Courier) OrderMinute(rng *rand.Rand, _ int) int {
	return sampleMinute(rng, 450, 30)
}

func (c *Courier) OrderItems(_ *rand.Rand, _ int, getItems func(ItemType, int) []Item) []Item {
	return getItems(Elixir, 1)
}

// --- Artificer (25% of market) ---

// Artificer represents a workshop crafter with varied ordering patterns.
type Artificer struct{}

func (a *Artificer) Name() string { return "Artificer" }

func (a *Artificer) PBuyPersona(isWeekend bool, _ Season, favoriteNumber int) float64 {
	if isWeekend {
		return 0.001
	}
	return (float64(favoriteNumber) / 100.0) * 0.4
}

func (a *Artificer) PTweet() float64 { return 0.01 }

func (a *Artificer) OrderMinute(rng *rand.Rand, _ int) int {
	return sampleMinute(rng, 420, 180)
}

func (a *Artificer) OrderItems(rng *rand.Rand, _ int, getItems func(ItemType, int) []Item) []Item {
	items := getItems(Elixir, 1)
	if rng.Float64() < 0.3 {
		items = append(items, getItems(Elixir, 1)...)
	}
	if rng.Float64() < 0.3 {
		items = append(items, getItems(solidItemType(rng), 1)...)
	}
	return items
}

// --- FeastReveler (10% of market) ---

// FeastReveler represents weekend festival-goers who order generously.
type FeastReveler struct{}

func (f *FeastReveler) Name() string { return "FeastReveler" }

func (f *FeastReveler) PBuyPersona(isWeekend bool, _ Season, favoriteNumber int) float64 {
	if !isWeekend {
		return 0
	}
	return 0.2 + (float64(favoriteNumber)/100.0)*0.2
}

func (f *FeastReveler) PTweet() float64 { return 0.8 }

func (f *FeastReveler) OrderMinute(rng *rand.Rand, favoriteNumber int) int {
	mu := 300 + float64(favoriteNumber-50)/50.0*120
	return sampleMinute(rng, mu, 120)
}

func (f *FeastReveler) OrderItems(rng *rand.Rand, favoriteNumber int, getItems func(ItemType, int) []Item) []Item {
	count := 1 + favoriteNumber/20
	items := getItems(solidItemType(rng), count)
	items = append(items, getItems(Elixir, count)...)
	return items
}

// --- Apprentice (20% of market) ---

// Apprentice represents a guild academy student absent during summer solstice.
type Apprentice struct{}

func (a *Apprentice) Name() string { return "Apprentice" }

func (a *Apprentice) PBuyPersona(_ bool, season Season, favoriteNumber int) float64 {
	if season == Summer {
		return 0
	}
	return 0.1 + (float64(favoriteNumber)/100.0)*0.4
}

func (a *Apprentice) PTweet() float64 { return 0.8 }

func (a *Apprentice) OrderMinute(rng *rand.Rand, _ int) int {
	return sampleMinute(rng, 540, 120)
}

func (a *Apprentice) OrderItems(rng *rand.Rand, _ int, getItems func(ItemType, int) []Item) []Item {
	items := getItems(Elixir, 1)
	if rng.Float64() < 0.5 {
		items = append(items, getItems(solidItemType(rng), 1)...)
	}
	return items
}

// --- Wanderer (10% of market) ---

// Wanderer represents traveling adventurers with random ordering patterns.
type Wanderer struct{}

func (w *Wanderer) Name() string { return "Wanderer" }

func (w *Wanderer) PBuyPersona(_ bool, _ Season, _ int) float64 {
	return 0.1
}

func (w *Wanderer) PTweet() float64 { return 0.1 }

func (w *Wanderer) OrderMinute(rng *rand.Rand, _ int) int {
	return sampleMinute(rng, 300, 120)
}

func (w *Wanderer) OrderItems(rng *rand.Rand, _ int, getItems func(ItemType, int) []Item) []Item {
	elixirCount := int(rng.Float64() * 10 / 3)
	solidCount := int(rng.Float64() * 10 / 3)
	var items []Item
	if elixirCount > 0 {
		items = append(items, getItems(Elixir, elixirCount)...)
	}
	if solidCount > 0 {
		items = append(items, getItems(solidItemType(rng), solidCount)...)
	}
	return items
}

// --- Herbalist (10% of market) ---

// Herbalist represents a nature-focused patron most active in summer.
type Herbalist struct{}

func (h *Herbalist) Name() string { return "Herbalist" }

func (h *Herbalist) PBuyPersona(_ bool, season Season, favoriteNumber int) float64 {
	if season == Summer {
		return 0.1 + (float64(favoriteNumber)/100.0)*0.4
	}
	return 0.2
}

func (h *Herbalist) PTweet() float64 { return 0.6 }

func (h *Herbalist) OrderMinute(rng *rand.Rand, _ int) int {
	return sampleMinute(rng, 300, 120)
}

func (h *Herbalist) OrderItems(_ *rand.Rand, _ int, getItems func(ItemType, int) []Item) []Item {
	return getItems(Elixir, 1)
}
