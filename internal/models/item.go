package models

// ItemType distinguishes jaffles from beverages.
type ItemType int

const (
	Jaffle ItemType = iota
	Beverage
)

func (t ItemType) String() string {
	switch t {
	case Jaffle:
		return "jaffle"
	case Beverage:
		return "beverage"
	default:
		return "unknown"
	}
}

// Item represents a menu product in the catalog.
type Item struct {
	SKU         string
	Name        string
	Description string
	Type        ItemType
	Price       int64 // cents
}
