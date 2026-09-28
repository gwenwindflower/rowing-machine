package catalog

import "testing"

func TestSuppliesCount(t *testing.T) {
	if got := len(Supplies); got != 41 {
		t.Errorf("len(Supplies) = %d, want 41", got)
	}
}

func TestAllSupplyCostsPositive(t *testing.T) {
	for _, s := range Supplies {
		if s.Cost <= 0 {
			t.Errorf("supply %s (%s) has non-positive cost: %d", s.ID, s.Name, s.Cost)
		}
	}
}

func TestAllSuppliesHaveSKUs(t *testing.T) {
	for _, s := range Supplies {
		if len(s.SKUs) == 0 {
			t.Errorf("supply %s (%s) has no SKUs", s.ID, s.Name)
		}
	}
}

func TestAllSuppliesHaveOriginRegion(t *testing.T) {
	for _, s := range Supplies {
		if s.OriginRegion == "" {
			t.Errorf("supply %s (%s) has no origin region", s.ID, s.Name)
		}
	}
}

func TestDenormalizedSupplyRowsCount(t *testing.T) {
	rows := DenormalizedSupplyRows()
	if got := len(rows); got != 92 {
		t.Errorf("len(DenormalizedSupplyRows()) = %d, want 92", got)
	}
}

func TestDenormalizedSupplyRowsFields(t *testing.T) {
	rows := DenormalizedSupplyRows()
	for i, row := range rows {
		if row.ID == "" {
			t.Errorf("row %d has empty ID", i)
		}
		if row.Name == "" {
			t.Errorf("row %d has empty Name", i)
		}
		if row.SKU == "" {
			t.Errorf("row %d has empty SKU", i)
		}
		if row.Cost <= 0 {
			t.Errorf("row %d (%s) has non-positive cost: %d", i, row.ID, row.Cost)
		}
		if row.OriginRegion == "" {
			t.Errorf("row %d (%s) has empty origin region", i, row.ID)
		}
	}
}

func TestSupplyIDsUnique(t *testing.T) {
	seen := make(map[string]bool)
	for _, s := range Supplies {
		if seen[s.ID] {
			t.Errorf("duplicate supply ID: %s", s.ID)
		}
		seen[s.ID] = true
	}
}
