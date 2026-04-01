package catalog

import "rowing-machine/internal/models"

var allJaffleSKUs = []string{"JAF-001", "JAF-002", "JAF-003", "JAF-004", "JAF-005"}
var allBeverageSKUs = []string{"BEV-001", "BEV-002", "BEV-003", "BEV-004", "BEV-005"}

// Supplies contains all 29 supply items.
var Supplies = []models.Supply{
	// Non-perishable (packaging)
	{ID: "SUP-001", Name: "compostable cutlery - knife", Cost: 7, Perishable: false, SKUs: allJaffleSKUs},
	{ID: "SUP-002", Name: "cutlery - fork", Cost: 7, Perishable: false, SKUs: allJaffleSKUs},
	{ID: "SUP-003", Name: "serving boat", Cost: 11, Perishable: false, SKUs: allJaffleSKUs},
	{ID: "SUP-004", Name: "napkin", Cost: 4, Perishable: false, SKUs: allJaffleSKUs},
	{ID: "SUP-005", Name: "16oz compostable clear cup", Cost: 13, Perishable: false, SKUs: allBeverageSKUs},
	{ID: "SUP-006", Name: "16oz compostable clear lid", Cost: 4, Perishable: false, SKUs: allBeverageSKUs},
	{ID: "SUP-007", Name: "biodegradable straw", Cost: 13, Perishable: false, SKUs: allBeverageSKUs},

	// Perishable (ingredients)
	{ID: "SUP-008", Name: "chai mix", Cost: 98, Perishable: true, SKUs: []string{"BEV-002"}},
	{ID: "SUP-009", Name: "bread", Cost: 33, Perishable: true, SKUs: allJaffleSKUs},
	{ID: "SUP-010", Name: "cheese", Cost: 20, Perishable: true, SKUs: []string{"JAF-002", "JAF-003", "JAF-004", "JAF-005"}},
	{ID: "SUP-011", Name: "nutella", Cost: 46, Perishable: true, SKUs: []string{"JAF-001"}},
	{ID: "SUP-012", Name: "banana", Cost: 13, Perishable: true, SKUs: []string{"JAF-001"}},
	{ID: "SUP-013", Name: "beef stew", Cost: 169, Perishable: true, SKUs: []string{"JAF-002"}},
	{ID: "SUP-014", Name: "lamb and pork bratwurst", Cost: 234, Perishable: true, SKUs: []string{"JAF-003"}},
	{ID: "SUP-015", Name: "house-pickled cabbage sauerkraut", Cost: 43, Perishable: true, SKUs: []string{"JAF-003"}},
	{ID: "SUP-016", Name: "mustard", Cost: 7, Perishable: true, SKUs: []string{"JAF-003"}},
	{ID: "SUP-017", Name: "pulled pork", Cost: 215, Perishable: true, SKUs: []string{"JAF-004"}},
	{ID: "SUP-018", Name: "pineapple", Cost: 26, Perishable: true, SKUs: []string{"JAF-004"}},
	{ID: "SUP-019", Name: "melon", Cost: 33, Perishable: true, SKUs: []string{"JAF-005"}},
	{ID: "SUP-020", Name: "minced beef", Cost: 124, Perishable: true, SKUs: []string{"JAF-005"}},
	{ID: "SUP-021", Name: "ghost pepper sauce", Cost: 20, Perishable: true, SKUs: []string{"JAF-004"}},
	{ID: "SUP-022", Name: "mango", Cost: 32, Perishable: true, SKUs: []string{"BEV-001"}},
	{ID: "SUP-023", Name: "tangerine", Cost: 20, Perishable: true, SKUs: []string{"BEV-001"}},
	{ID: "SUP-024", Name: "oatmilk", Cost: 11, Perishable: true, SKUs: []string{"BEV-002"}},
	{ID: "SUP-025", Name: "whey protein", Cost: 36, Perishable: true, SKUs: []string{"BEV-002"}},
	{ID: "SUP-026", Name: "coffee", Cost: 52, Perishable: true, SKUs: []string{"BEV-003", "BEV-004"}},
	{ID: "SUP-027", Name: "french vanilla syrup", Cost: 72, Perishable: true, SKUs: []string{"BEV-003"}},
	{ID: "SUP-028", Name: "kiwi", Cost: 20, Perishable: true, SKUs: []string{"BEV-005"}},
	{ID: "SUP-029", Name: "lime", Cost: 13, Perishable: true, SKUs: []string{"BEV-005"}},
}

// SupplyRow is a denormalized representation of a supply-SKU pair for CSV output.
type SupplyRow struct {
	ID         string
	Name       string
	Cost       int64
	Perishable bool
	SKU        string
}

// DenormalizedSupplyRows returns one row per (supply, SKU) pair for CSV output.
func DenormalizedSupplyRows() []SupplyRow {
	var rows []SupplyRow
	for _, s := range Supplies {
		for _, sku := range s.SKUs {
			rows = append(rows, SupplyRow{
				ID:         s.ID,
				Name:       s.Name,
				Cost:       s.Cost,
				Perishable: s.Perishable,
				SKU:        sku,
			})
		}
	}
	return rows
}
