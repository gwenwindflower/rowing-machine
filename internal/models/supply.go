package models

// Supply represents an ingredient or packaging material.
type Supply struct {
	ID         string
	Name       string
	Cost       int64 // cents
	Perishable bool
	SKUs       []string // which product SKUs use this supply
}
