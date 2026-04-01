package catalog

import (
	"math/rand/v2"

	"rowing-machine/internal/models"
)

// MenuItems contains all 10 items on the menu.
var MenuItems = []models.Item{
	{SKU: "JAF-001", Name: "nutellaphone who dis?", Price: 1100, Type: models.Jaffle, Description: "nutella and banana jaffle"},
	{SKU: "JAF-002", Name: "doctor stew", Price: 1100, Type: models.Jaffle, Description: "house-made beef stew jaffle"},
	{SKU: "JAF-003", Name: "the krautback", Price: 1200, Type: models.Jaffle, Description: "lamb and pork bratwurst with house-pickled cabbage sauerkraut and mustard"},
	{SKU: "JAF-004", Name: "flame impala", Price: 1400, Type: models.Jaffle, Description: "pulled pork and pineapple al pastor marinated in ghost pepper sauce"},
	{SKU: "JAF-005", Name: "mel-bun", Price: 1200, Type: models.Jaffle, Description: "melon and minced beef bao, in a jaffle, savory and sweet"},
	{SKU: "BEV-001", Name: "tangaroo", Price: 600, Type: models.Beverage, Description: "mango and tangerine smoothie"},
	{SKU: "BEV-002", Name: "chai and mighty", Price: 500, Type: models.Beverage, Description: "oatmilk chai latte with protein boost"},
	{SKU: "BEV-003", Name: "vanilla ice", Price: 600, Type: models.Beverage, Description: "iced coffee with house-made french vanilla syrup"},
	{SKU: "BEV-004", Name: "for richer or pourover", Price: 700, Type: models.Beverage, Description: "daily selection of single estate beans for a delicious hot pourover"},
	{SKU: "BEV-005", Name: "adele-ade", Price: 400, Type: models.Beverage, Description: "a kiwi and lime agua fresca"},
}

// Jaffles contains only the jaffle menu items.
var Jaffles = filterByType(MenuItems, models.Jaffle)

// Beverages contains only the beverage menu items.
var Beverages = filterByType(MenuItems, models.Beverage)

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
	case models.Jaffle:
		pool = Jaffles
	case models.Beverage:
		pool = Beverages
	}

	result := make([]models.Item, count)
	for i := range count {
		result[i] = pool[rng.IntN(len(pool))]
	}
	return result
}
