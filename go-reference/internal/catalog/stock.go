package catalog

import "rowing-machine/internal/models"

var allWeaponSKUs = []string{"WEP-001", "WEP-002", "WEP-003", "WEP-004", "WEP-005"}
var allArmorSKUs = []string{"ARM-001", "ARM-002", "ARM-003", "ARM-004", "ARM-005"}
var allElixirSKUs = []string{"ELX-001", "ELX-002", "ELX-003", "ELX-004", "ELX-005"}

// Supplies contains all reagent and enchanting material items.
var Supplies = []models.Supply{
	// Non-volatile (enchanting materials — shared across item types)
	{ID: "SUP-001", Name: "enchanted wrapping cloth", Cost: 7, Volatile: false, OriginRegion: "Arcanum Workshops", SKUs: allWeaponSKUs},
	{ID: "SUP-002", Name: "binding rune seal", Cost: 7, Volatile: false, OriginRegion: "Arcanum Workshops", SKUs: allWeaponSKUs},
	{ID: "SUP-003", Name: "display crystal case", Cost: 11, Volatile: false, OriginRegion: "Arcanum Workshops", SKUs: allWeaponSKUs},
	{ID: "SUP-004", Name: "arcane parchment label", Cost: 4, Volatile: false, OriginRegion: "Arcanum Workshops", SKUs: allWeaponSKUs},
	{ID: "SUP-005", Name: "reinforcement stitching", Cost: 7, Volatile: false, OriginRegion: "Arcanum Workshops", SKUs: allArmorSKUs},
	{ID: "SUP-006", Name: "fitting buckles", Cost: 7, Volatile: false, OriginRegion: "Arcanum Workshops", SKUs: allArmorSKUs},
	{ID: "SUP-007", Name: "padding lining", Cost: 11, Volatile: false, OriginRegion: "Arcanum Workshops", SKUs: allArmorSKUs},
	{ID: "SUP-008", Name: "glass vial - large", Cost: 13, Volatile: false, OriginRegion: "Arcanum Workshops", SKUs: allElixirSKUs},
	{ID: "SUP-009", Name: "cork stopper - waxed", Cost: 4, Volatile: false, OriginRegion: "Arcanum Workshops", SKUs: allElixirSKUs},
	{ID: "SUP-010", Name: "silkworm thread seal", Cost: 13, Volatile: false, OriginRegion: "Arcanum Workshops", SKUs: allElixirSKUs},

	// Volatile (crafting reagents — weapon-specific)
	{ID: "SUP-011", Name: "iron ore ingot", Cost: 33, Volatile: true, OriginRegion: "Embervault Mines", SKUs: allWeaponSKUs},
	{ID: "SUP-012", Name: "drake fire ember", Cost: 46, Volatile: true, OriginRegion: "Embervault Mines", SKUs: []string{"WEP-001"}},
	{ID: "SUP-013", Name: "raw iron bar", Cost: 13, Volatile: true, OriginRegion: "Embervault Mines", SKUs: []string{"WEP-001"}},
	{ID: "SUP-014", Name: "thunderhawk sinew", Cost: 169, Volatile: true, OriginRegion: "Skyreach Plateau", SKUs: []string{"WEP-002"}},
	{ID: "SUP-015", Name: "emberveil obsidian", Cost: 234, Volatile: true, OriginRegion: "Embervault Mines", SKUs: []string{"WEP-003"}},
	{ID: "SUP-016", Name: "flickerflame oil", Cost: 43, Volatile: true, OriginRegion: "Embervault Mines", SKUs: []string{"WEP-003"}},
	{ID: "SUP-017", Name: "volcanic glass shard", Cost: 215, Volatile: true, OriginRegion: "Embervault Mines", SKUs: []string{"WEP-004"}},
	{ID: "SUP-018", Name: "phoenix core feather", Cost: 26, Volatile: true, OriginRegion: "Skyreach Plateau", SKUs: []string{"WEP-004"}},
	{ID: "SUP-019", Name: "molten fire salt", Cost: 20, Volatile: true, OriginRegion: "Embervault Mines", SKUs: []string{"WEP-004"}},
	{ID: "SUP-020", Name: "void crystal", Cost: 33, Volatile: true, OriginRegion: "Voidrift Expanse", SKUs: []string{"WEP-005"}},
	{ID: "SUP-021", Name: "dimensional rift ink", Cost: 124, Volatile: true, OriginRegion: "Voidrift Expanse", SKUs: []string{"WEP-005"}},

	// Volatile (crafting reagents — armor-specific)
	{ID: "SUP-022", Name: "leather strapping", Cost: 20, Volatile: true, OriginRegion: "Darkwood Forest", SKUs: allArmorSKUs},
	{ID: "SUP-023", Name: "steel rivets", Cost: 7, Volatile: true, OriginRegion: "Embervault Mines", SKUs: []string{"ARM-001", "ARM-002", "ARM-003"}},
	{ID: "SUP-024", Name: "ironbark heartwood", Cost: 98, Volatile: true, OriginRegion: "Darkwood Forest", SKUs: []string{"ARM-001"}},
	{ID: "SUP-025", Name: "permafrost oak plank", Cost: 234, Volatile: true, OriginRegion: "Frostpeak Mountains", SKUs: []string{"ARM-002"}},
	{ID: "SUP-026", Name: "glacial ward rune", Cost: 43, Volatile: true, OriginRegion: "Frostpeak Mountains", SKUs: []string{"ARM-002"}},
	{ID: "SUP-027", Name: "drake scale plates", Cost: 215, Volatile: true, OriginRegion: "Skyreach Plateau", SKUs: []string{"ARM-003"}},
	{ID: "SUP-028", Name: "drake scale resin", Cost: 20, Volatile: true, OriginRegion: "Skyreach Plateau", SKUs: []string{"ARM-003"}},
	{ID: "SUP-029", Name: "phoenix down plume", Cost: 169, Volatile: true, OriginRegion: "Skyreach Plateau", SKUs: []string{"ARM-004"}},
	{ID: "SUP-030", Name: "fireweave thread", Cost: 72, Volatile: true, OriginRegion: "Embervault Mines", SKUs: []string{"ARM-004"}},
	{ID: "SUP-031", Name: "dimensional rift silk", Cost: 234, Volatile: true, OriginRegion: "Voidrift Expanse", SKUs: []string{"ARM-005"}},
	{ID: "SUP-032", Name: "void essence dye", Cost: 124, Volatile: true, OriginRegion: "Voidrift Expanse", SKUs: []string{"ARM-005"}},

	// Volatile (crafting reagents — elixir-specific)
	{ID: "SUP-033", Name: "sunfruit pulp", Cost: 32, Volatile: true, OriginRegion: "Shimmerfen Wetlands", SKUs: []string{"ELX-001"}},
	{ID: "SUP-034", Name: "citrine essence", Cost: 20, Volatile: true, OriginRegion: "Shimmerfen Wetlands", SKUs: []string{"ELX-001"}},
	{ID: "SUP-035", Name: "stormspice blend", Cost: 98, Volatile: true, OriginRegion: "Skyreach Plateau", SKUs: []string{"ELX-002"}},
	{ID: "SUP-036", Name: "ironbark sap", Cost: 11, Volatile: true, OriginRegion: "Darkwood Forest", SKUs: []string{"ELX-002"}},
	{ID: "SUP-037", Name: "basilisk bone powder", Cost: 36, Volatile: true, OriginRegion: "Voidrift Expanse", SKUs: []string{"ELX-002"}},
	{ID: "SUP-038", Name: "shadowbean grounds", Cost: 52, Volatile: true, OriginRegion: "Shimmerfen Wetlands", SKUs: []string{"ELX-003", "ELX-004"}},
	{ID: "SUP-039", Name: "vanilla frost crystal", Cost: 72, Volatile: true, OriginRegion: "Frostpeak Mountains", SKUs: []string{"ELX-003"}},
	{ID: "SUP-040", Name: "emerald kiwi extract", Cost: 20, Volatile: true, OriginRegion: "Darkwood Forest", SKUs: []string{"ELX-005"}},
	{ID: "SUP-041", Name: "serpent lime juice", Cost: 13, Volatile: true, OriginRegion: "Shimmerfen Wetlands", SKUs: []string{"ELX-005"}},
}

// SupplyRow is a denormalized representation of a supply-SKU pair for CSV output.
type SupplyRow struct {
	ID           string
	Name         string
	Cost         int64
	Volatile     bool
	OriginRegion string
	SKU          string
}

// DenormalizedSupplyRows returns one row per (supply, SKU) pair for CSV output.
func DenormalizedSupplyRows() []SupplyRow {
	var rows []SupplyRow
	for _, s := range Supplies {
		for _, sku := range s.SKUs {
			rows = append(rows, SupplyRow{
				ID:           s.ID,
				Name:         s.Name,
				Cost:         s.Cost,
				Volatile:     s.Volatile,
				OriginRegion: s.OriginRegion,
				SKU:          sku,
			})
		}
	}
	return rows
}
