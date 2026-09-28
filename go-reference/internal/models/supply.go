package models

// Supply represents a crafting reagent or enchanting material.
type Supply struct {
	ID           string
	Name         string
	Cost         int64 // cents
	Volatile     bool
	OriginRegion string
	SKUs         []string // which product SKUs use this supply
}
