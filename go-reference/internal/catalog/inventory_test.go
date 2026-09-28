package catalog

import (
	"math/rand/v2"
	"testing"

	"rowing-machine/internal/models"
)

func TestMenuItemsCount(t *testing.T) {
	if got := len(MenuItems); got != 15 {
		t.Errorf("len(MenuItems) = %d, want 15", got)
	}
}

func TestWeaponsCount(t *testing.T) {
	if got := len(Weapons); got != 5 {
		t.Errorf("len(Weapons) = %d, want 5", got)
	}
}

func TestArmorItemsCount(t *testing.T) {
	if got := len(ArmorItems); got != 5 {
		t.Errorf("len(ArmorItems) = %d, want 5", got)
	}
}

func TestElixirsCount(t *testing.T) {
	if got := len(Elixirs); got != 5 {
		t.Errorf("len(Elixirs) = %d, want 5", got)
	}
}

func TestAllPricesPositive(t *testing.T) {
	for _, item := range MenuItems {
		if item.Price <= 0 {
			t.Errorf("item %s (%s) has non-positive price: %d", item.SKU, item.Name, item.Price)
		}
	}
}

func TestEachProductTypeCoversEveryPowerLevel(t *testing.T) {
	productGroups := map[string][]models.Item{
		"weapons": Weapons,
		"armor":   ArmorItems,
		"elixirs": Elixirs,
	}
	wantLevels := []models.PowerLevel{
		models.Common,
		models.Uncommon,
		models.Rare,
		models.Epic,
		models.Legendary,
	}

	for name, products := range productGroups {
		t.Run(name, func(t *testing.T) {
			counts := make(map[models.PowerLevel]int, len(products))
			for _, product := range products {
				counts[product.PowerLevel]++
			}
			for _, level := range wantLevels {
				if counts[level] != 1 {
					t.Errorf("%s products with power level %q = %d, want 1", name, level.String(), counts[level])
				}
			}
		})
	}
}

func TestWeaponsAllWeaponType(t *testing.T) {
	for _, item := range Weapons {
		if item.Type != models.Weapon {
			t.Errorf("Weapons contains non-weapon item: %s (type %d)", item.SKU, item.Type)
		}
	}
}

func TestArmorAllArmorType(t *testing.T) {
	for _, item := range ArmorItems {
		if item.Type != models.Armor {
			t.Errorf("ArmorItems contains non-armor item: %s (type %d)", item.SKU, item.Type)
		}
	}
}

func TestElixirsAllElixirType(t *testing.T) {
	for _, item := range Elixirs {
		if item.Type != models.Elixir {
			t.Errorf("Elixirs contains non-elixir item: %s (type %d)", item.SKU, item.Type)
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
		{"3 weapons", models.Weapon, 3},
		{"5 elixirs", models.Elixir, 5},
		{"1 weapon", models.Weapon, 1},
		{"10 elixirs", models.Elixir, 10},
		{"2 armor", models.Armor, 2},
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

	weapons := RandomItems(rng, models.Weapon, 20)
	for i, item := range weapons {
		if item.Type != models.Weapon {
			t.Errorf("weapons[%d] has type %d, want Weapon", i, item.Type)
		}
	}

	elixirs := RandomItems(rng, models.Elixir, 20)
	for i, item := range elixirs {
		if item.Type != models.Elixir {
			t.Errorf("elixirs[%d] has type %d, want Elixir", i, item.Type)
		}
	}

	armor := RandomItems(rng, models.Armor, 20)
	for i, item := range armor {
		if item.Type != models.Armor {
			t.Errorf("armor[%d] has type %d, want Armor", i, item.Type)
		}
	}
}

func TestRandomItemsDeterministic(t *testing.T) {
	rng1 := rand.New(rand.NewPCG(123, 0))
	rng2 := rand.New(rand.NewPCG(123, 0))

	items1 := RandomItems(rng1, models.Weapon, 10)
	items2 := RandomItems(rng2, models.Weapon, 10)

	for i := range items1 {
		if items1[i].SKU != items2[i].SKU {
			t.Errorf("item %d: got SKU %s and %s for same seed", i, items1[i].SKU, items2[i].SKU)
		}
	}
}
