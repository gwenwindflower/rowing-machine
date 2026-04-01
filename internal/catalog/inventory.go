package catalog

import (
	"math/rand/v2"

	"rowing-machine/internal/models"
)

// MenuItems contains all 15 items in the guild catalog.
var MenuItems = []models.Item{
	// Weapons
	{SKU: "WEP-001", Name: "wyrmfang edge", Price: 1100, Type: models.Weapon, PowerLevel: models.Common, Description: "iron short sword tempered in drake fire"},
	{SKU: "WEP-002", Name: "stormcaller bow", Price: 1100, Type: models.Weapon, PowerLevel: models.Uncommon, Description: "recurve bow strung with thunderhawk sinew"},
	{SKU: "WEP-003", Name: "emberveil dagger", Price: 1200, Type: models.Weapon, PowerLevel: models.Rare, Description: "obsidian blade that trails embers when drawn"},
	{SKU: "WEP-004", Name: "inferno maul", Price: 1400, Type: models.Weapon, PowerLevel: models.Epic, Description: "warhammer forged in volcanic glass with phoenix core"},
	{SKU: "WEP-005", Name: "void sigil staff", Price: 1200, Type: models.Weapon, PowerLevel: models.Legendary, Description: "quarterstaff inscribed with a dimensional rift glyph"},
	// Armor
	{SKU: "ARM-001", Name: "ironbark buckler", Price: 800, Type: models.Armor, PowerLevel: models.Common, Description: "small round shield carved from ironbark heartwood"},
	{SKU: "ARM-002", Name: "glacial bulwark", Price: 1200, Type: models.Armor, PowerLevel: models.Uncommon, Description: "tower shield reinforced with permafrost oak"},
	{SKU: "ARM-003", Name: "drake scale cuirass", Price: 1500, Type: models.Armor, PowerLevel: models.Rare, Description: "chest plate layered with drake scale resin"},
	{SKU: "ARM-004", Name: "phoenix ward mantle", Price: 1800, Type: models.Armor, PowerLevel: models.Epic, Description: "shoulder guard woven with phoenix feathers"},
	{SKU: "ARM-005", Name: "voidweave vestments", Price: 2000, Type: models.Armor, PowerLevel: models.Legendary, Description: "robes threaded with dimensional rift silk"},
	// Elixirs
	{SKU: "ELX-001", Name: "sunfire tonic", Price: 600, Type: models.Elixir, PowerLevel: models.Common, Description: "mango and tangerine essence energy brew"},
	{SKU: "ELX-002", Name: "ironbark draught", Price: 500, Type: models.Elixir, PowerLevel: models.Common, Description: "oatmilk and spice fortification potion"},
	{SKU: "ELX-003", Name: "frostmint vial", Price: 600, Type: models.Elixir, PowerLevel: models.Uncommon, Description: "chilled coffee infused with vanilla frost crystals"},
	{SKU: "ELX-004", Name: "oracle's brew", Price: 700, Type: models.Elixir, PowerLevel: models.Rare, Description: "single-origin bean vision-enhancing elixir"},
	{SKU: "ELX-005", Name: "serpent's kiss", Price: 400, Type: models.Elixir, PowerLevel: models.Uncommon, Description: "kiwi and lime venom-neutralizing tincture"},
}

// Weapons contains only weapon items.
var Weapons = filterByType(MenuItems, models.Weapon)

// ArmorItems contains only armor items.
var ArmorItems = filterByType(MenuItems, models.Armor)

// Elixirs contains only elixir items.
var Elixirs = filterByType(MenuItems, models.Elixir)

func filterByType(items []models.Item, t models.ItemType) []models.Item {
	var result []models.Item
	for _, item := range items {
		if item.Type == t {
			result = append(result, item)
		}
	}
	return result
}

// RandomItems picks count items of the given type with replacement from the catalog.
func RandomItems(rng *rand.Rand, itemType models.ItemType, count int) []models.Item {
	var pool []models.Item
	switch itemType {
	case models.Weapon:
		pool = Weapons
	case models.Armor:
		pool = ArmorItems
	case models.Elixir:
		pool = Elixirs
	}

	result := make([]models.Item, count)
	for i := range count {
		result[i] = pool[rng.IntN(len(pool))]
	}
	return result
}
