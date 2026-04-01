package models

import (
	"math"
	"testing"
)

func TestPBuy(t *testing.T) {
	tests := []struct {
		name           string
		basePopularity float64
		dayEffect      float64
		want           float64
	}{
		{"dayEffect 1.0 returns base", 0.85, 1.0, 0.85},
		{"dayEffect 0.5 returns half base", 0.85, 0.5, 0.425},
		{"dayEffect 0.0 returns zero", 0.92, 0.0, 0.0},
		{"dayEffect 2.0 doubles base", 0.95, 2.0, 1.9},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			s := &Store{BasePopularity: tt.basePopularity}
			got := s.PBuy(tt.dayEffect)
			if math.Abs(got-tt.want) > 1e-12 {
				t.Errorf("PBuy(%v) = %v, want %v", tt.dayEffect, got, tt.want)
			}
		})
	}
}

func TestIsOpen(t *testing.T) {
	tests := []struct {
		name      string
		openedDay int
		dayIndex  int
		want      bool
	}{
		{"before opening day", 192, 191, false},
		{"on opening day", 192, 192, true},
		{"after opening day", 192, 193, true},
		{"day zero store on day zero", 0, 0, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			s := &Store{OpenedDay: tt.openedDay}
			got := s.IsOpen(tt.dayIndex)
			if got != tt.want {
				t.Errorf("IsOpen(%d) = %v, want %v", tt.dayIndex, got, tt.want)
			}
		})
	}
}

func TestDaysSinceOpen(t *testing.T) {
	tests := []struct {
		name      string
		openedDay int
		dayIndex  int
		want      int
	}{
		{"before opening", 192, 100, -92},
		{"on opening day", 192, 192, 0},
		{"after opening", 192, 292, 100},
		{"day zero store on day zero", 0, 0, 0},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			s := &Store{OpenedDay: tt.openedDay}
			got := s.DaysSinceOpen(tt.dayIndex)
			if got != tt.want {
				t.Errorf("DaysSinceOpen(%d) = %d, want %d", tt.dayIndex, got, tt.want)
			}
		})
	}
}

func TestIsOpenAt(t *testing.T) {
	s := &Store{}

	// Weekday hours: 420 (7:00 AM) to 1200 (8:00 PM)
	weekdayTests := []struct {
		name   string
		minute int
		want   bool
	}{
		{"weekday just before open", 419, false},
		{"weekday at open", 420, true},
		{"weekday just before close", 1199, true},
		{"weekday at close", 1200, false},
	}

	for _, tt := range weekdayTests {
		t.Run(tt.name, func(t *testing.T) {
			got := s.IsOpenAt(tt.minute, 420, 1200)
			if got != tt.want {
				t.Errorf("IsOpenAt(%d, 420, 1200) = %v, want %v", tt.minute, got, tt.want)
			}
		})
	}

	// Weekend hours: 480 (8:00 AM) to 900 (3:00 PM)
	weekendTests := []struct {
		name   string
		minute int
		want   bool
	}{
		{"weekend just before open", 479, false},
		{"weekend at open", 480, true},
		{"weekend just before close", 899, true},
		{"weekend at close", 900, false},
	}

	for _, tt := range weekendTests {
		t.Run(tt.name, func(t *testing.T) {
			got := s.IsOpenAt(tt.minute, 480, 900)
			if got != tt.want {
				t.Errorf("IsOpenAt(%d, 480, 900) = %v, want %v", tt.minute, got, tt.want)
			}
		})
	}
}

func TestStoreConfigs(t *testing.T) {
	stores := StoreConfigs()

	if len(stores) != 6 {
		t.Fatalf("StoreConfigs() returned %d stores, want 6", len(stores))
	}

	expected := []struct {
		name    string
		taxRate float64
	}{
		{"Philadelphia", 0.06},
		{"Brooklyn", 0.04},
		{"Chicago", 0.0625},
		{"San Francisco", 0.075},
		{"New Orleans", 0.04},
		{"Los Angeles", 0.08},
	}

	for i, exp := range expected {
		if stores[i].Name != exp.name {
			t.Errorf("store[%d].Name = %q, want %q", i, stores[i].Name, exp.name)
		}
		if stores[i].TaxRate != exp.taxRate {
			t.Errorf("store[%d].TaxRate = %v, want %v", i, stores[i].TaxRate, exp.taxRate)
		}
	}

	// Verify all IDs are zero (unset)
	var zeroID [16]byte
	for i, s := range stores {
		if s.ID != zeroID {
			t.Errorf("store[%d].ID should be zero, got %v", i, s.ID)
		}
	}
}
