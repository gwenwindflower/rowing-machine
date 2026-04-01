package catalog

import (
	"math/rand/v2"
	"testing"

	"rowing-machine/internal/models"
)

func TestMenuItemsCount(t *testing.T) {
	if got := len(MenuItems); got != 10 {
		t.Errorf("len(MenuItems) = %d, want 10", got)
	}
}

func TestJafflesCount(t *testing.T) {
	if got := len(Jaffles); got != 5 {
		t.Errorf("len(Jaffles) = %d, want 5", got)
	}
}

func TestBeveragesCount(t *testing.T) {
	if got := len(Beverages); got != 5 {
		t.Errorf("len(Beverages) = %d, want 5", got)
	}
}

func TestAllPricesPositive(t *testing.T) {
	for _, item := range MenuItems {
		if item.Price <= 0 {
			t.Errorf("item %s (%s) has non-positive price: %d", item.SKU, item.Name, item.Price)
		}
	}
}

func TestJafflesAllJaffleType(t *testing.T) {
	for _, item := range Jaffles {
		if item.Type != models.Jaffle {
			t.Errorf("Jaffles contains non-jaffle item: %s (type %d)", item.SKU, item.Type)
		}
	}
}

func TestBeveragesAllBeverageType(t *testing.T) {
	for _, item := range Beverages {
		if item.Type != models.Beverage {
			t.Errorf("Beverages contains non-beverage item: %s (type %d)", item.SKU, item.Type)
		}
	}
}

func TestRandomItemsCount(t *testing.T) {
	rng := rand.New(rand.NewPCG(42, 0))

	tests := []struct {
		name     string
		itemType models.ItemType
		count    int
	}{
		{"3 jaffles", models.Jaffle, 3},
		{"5 beverages", models.Beverage, 5},
		{"1 jaffle", models.Jaffle, 1},
		{"10 beverages", models.Beverage, 10},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			items := RandomItems(rng, tt.itemType, tt.count)
			if len(items) != tt.count {
				t.Errorf("RandomItems returned %d items, want %d", len(items), tt.count)
			}
		})
	}
}

func TestRandomItemsMatchType(t *testing.T) {
	rng := rand.New(rand.NewPCG(99, 0))

	jaffles := RandomItems(rng, models.Jaffle, 20)
	for i, item := range jaffles {
		if item.Type != models.Jaffle {
			t.Errorf("jaffles[%d] has type %d, want Jaffle", i, item.Type)
		}
	}

	beverages := RandomItems(rng, models.Beverage, 20)
	for i, item := range beverages {
		if item.Type != models.Beverage {
			t.Errorf("beverages[%d] has type %d, want Beverage", i, item.Type)
		}
	}
}

func TestRandomItemsDeterministic(t *testing.T) {
	rng1 := rand.New(rand.NewPCG(123, 0))
	rng2 := rand.New(rand.NewPCG(123, 0))

	items1 := RandomItems(rng1, models.Jaffle, 10)
	items2 := RandomItems(rng2, models.Jaffle, 10)

	for i := range items1 {
		if items1[i].SKU != items2[i].SKU {
			t.Errorf("item %d: got SKU %s and %s for same seed", i, items1[i].SKU, items2[i].SKU)
		}
	}
}
