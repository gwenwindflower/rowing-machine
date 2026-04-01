package models

// ItemType distinguishes weapons, armor, and elixirs.
type ItemType int

const (
	Weapon ItemType = iota
	Armor
	Elixir
)

func (t ItemType) String() string {
	switch t {
	case Weapon:
		return "weapon"
	case Armor:
		return "armor"
	case Elixir:
		return "elixir"
	default:
		return "unknown"
	}
}

// PowerLevel represents the rarity tier of an item.
type PowerLevel int

const (
	Common PowerLevel = iota
	Uncommon
	Rare
	Epic
	Legendary
)

func (p PowerLevel) String() string {
	switch p {
	case Common:
		return "common"
	case Uncommon:
		return "uncommon"
	case Rare:
		return "rare"
	case Epic:
		return "epic"
	case Legendary:
		return "legendary"
	default:
		return "unknown"
	}
}

// Item represents a product in the guild catalog.
type Item struct {
	SKU         string
	Name        string
	Description string
	Type        ItemType
	PowerLevel  PowerLevel
	Price       int64 // cents
}
