package models

import (
	"math"
	"testing"
)

// mockGetItems returns a function that produces dummy items of the requested type.
func mockGetItems(itemType ItemType, count int) []Item {
	items := make([]Item, count)
	for i := range items {
		items[i] = Item{
			SKU:  "test-sku",
			Name: "Test Item",
			Type: itemType,
		}
	}
	return items
}

func TestPBuyPersonaRanges(t *testing.T) {
	tests := []struct {
		name      string
		persona   Persona
		isWeekend bool
		season    Season
		favNum    int
		wantMin   float64
		wantMax   float64
	}{
		// Courier weekday
		{"Courier weekday favNum=1", &Courier{}, false, Winter, 1, 0.5, 0.51},
		{"Courier weekday favNum=100", &Courier{}, false, Winter, 100, 0.79, 0.81},
		{"Courier weekend", &Courier{}, true, Winter, 50, 0.001, 0.001},

		// Artificer weekday
		{"Artificer weekday favNum=1", &Artificer{}, false, Winter, 1, 0.0, 0.01},
		{"Artificer weekday favNum=100", &Artificer{}, false, Winter, 100, 0.39, 0.41},
		{"Artificer weekend", &Artificer{}, true, Winter, 50, 0.001, 0.001},

		// FeastReveler
		{"FeastReveler weekday", &FeastReveler{}, false, Winter, 50, 0.0, 0.0},
		{"FeastReveler weekend favNum=1", &FeastReveler{}, true, Winter, 1, 0.20, 0.21},
		{"FeastReveler weekend favNum=100", &FeastReveler{}, true, Winter, 100, 0.39, 0.41},

		// Apprentice
		{"Apprentice summer", &Apprentice{}, false, Summer, 50, 0.0, 0.0},
		{"Apprentice winter favNum=1", &Apprentice{}, false, Winter, 1, 0.10, 0.11},
		{"Apprentice winter favNum=100", &Apprentice{}, false, Winter, 100, 0.49, 0.51},

		// Wanderer — constant
		{"Wanderer weekday favNum=1", &Wanderer{}, false, Winter, 1, 0.1, 0.1},
		{"Wanderer weekend favNum=100", &Wanderer{}, true, Summer, 100, 0.1, 0.1},

		// Herbalist
		{"Herbalist summer favNum=1", &Herbalist{}, false, Summer, 1, 0.10, 0.11},
		{"Herbalist summer favNum=100", &Herbalist{}, false, Summer, 100, 0.49, 0.51},
		{"Herbalist winter", &Herbalist{}, false, Winter, 50, 0.2, 0.2},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := tt.persona.PBuyPersona(tt.isWeekend, tt.season, tt.favNum)
			if got < tt.wantMin-1e-9 || got > tt.wantMax+1e-9 {
				t.Errorf("PBuyPersona() = %f, want [%f, %f]", got, tt.wantMin, tt.wantMax)
			}
		})
	}
}

func TestCourierWeekdayVsWeekend(t *testing.T) {
	c := &Courier{}
	weekday := c.PBuyPersona(false, Winter, 50)
	weekend := c.PBuyPersona(true, Winter, 50)

	if weekday < 0.5 || weekday > 0.8 {
		t.Errorf("Courier weekday PBuy = %f, want in [0.5, 0.8]", weekday)
	}
	if weekend != 0.001 {
		t.Errorf("Courier weekend PBuy = %f, want 0.001", weekend)
	}
	if weekend >= weekday {
		t.Error("Courier weekend probability should be much lower than weekday")
	}
}

func TestFeastRevelerWeekdayZero(t *testing.T) {
	f := &FeastReveler{}
	for favNum := 1; favNum <= 100; favNum++ {
		got := f.PBuyPersona(false, Winter, favNum)
		if got != 0 {
			t.Errorf("FeastReveler weekday PBuy with favNum=%d = %f, want 0", favNum, got)
		}
	}
}

func TestApprenticeSummerZero(t *testing.T) {
	a := &Apprentice{}
	for favNum := 1; favNum <= 100; favNum++ {
		got := a.PBuyPersona(false, Summer, favNum)
		if got != 0 {
			t.Errorf("Apprentice summer PBuy with favNum=%d = %f, want 0", favNum, got)
		}
		got = a.PBuyPersona(true, Summer, favNum)
		if got != 0 {
			t.Errorf("Apprentice summer weekend PBuy with favNum=%d = %f, want 0", favNum, got)
		}
	}
}

func TestHerbalistSummerBoost(t *testing.T) {
	h := &Herbalist{}
	summerProb := h.PBuyPersona(false, Summer, 80)
	winterProb := h.PBuyPersona(false, Winter, 80)
	if summerProb <= winterProb {
		t.Errorf("Herbalist summer=%f should be > winter=%f for high favNum", summerProb, winterProb)
	}
}

func TestWandererConstant(t *testing.T) {
	w := &Wanderer{}
	scenarios := []struct {
		isWeekend bool
		season    Season
		favNum    int
	}{
		{false, Winter, 1},
		{true, Summer, 100},
		{false, Spring, 50},
		{true, Fall, 25},
	}
	for _, s := range scenarios {
		got := w.PBuyPersona(s.isWeekend, s.season, s.favNum)
		if got != 0.1 {
			t.Errorf("Wanderer PBuy(weekend=%v, season=%d, favNum=%d) = %f, want 0.1",
				s.isWeekend, s.season, s.favNum, got)
		}
	}
}

func TestOrderMinuteDistributions(t *testing.T) {
	const iterations = 1000

	tests := []struct {
		name       string
		persona    Persona
		favNum     int
		wantMean   float64
		tolerance  float64
		checkClamp bool
	}{
		{"Courier mean near 450", &Courier{}, 50, 450, 15, true},
		{"Artificer mean near 420", &Artificer{}, 50, 420, 30, true},
		{"Apprentice mean near 540", &Apprentice{}, 50, 540, 20, true},
		{"Wanderer mean near 300", &Wanderer{}, 50, 300, 20, true},
		{"Herbalist mean near 300", &Herbalist{}, 50, 300, 20, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			rng := newTestRNG(42)
			sum := 0
			allNonNeg := true
			for i := 0; i < iterations; i++ {
				m := tt.persona.OrderMinute(rng, tt.favNum)
				sum += m
				if m < 0 {
					allNonNeg = false
				}
			}

			mean := float64(sum) / float64(iterations)
			if math.Abs(mean-tt.wantMean) > tt.tolerance {
				t.Errorf("mean OrderMinute = %.1f, want near %.1f (tolerance ±%.1f)",
					mean, tt.wantMean, tt.tolerance)
			}

			if tt.checkClamp && !allNonNeg {
				t.Error("OrderMinute returned negative value, clamping is broken")
			}
		})
	}
}

func TestCourierOrderMinuteNotPythonBug(t *testing.T) {
	rng := newTestRNG(123)
	sum := 0
	for i := 0; i < 1000; i++ {
		sum += (&Courier{}).OrderMinute(rng, 50)
	}
	mean := float64(sum) / 1000.0
	if mean < 200 {
		t.Errorf("Courier OrderMinute mean = %.1f, appears to have Python bug (mu=60). Expected ~450.", mean)
	}
}

func TestFeastRevelerOrderMinuteFavNumEffect(t *testing.T) {
	rng1 := newTestRNG(42)
	rng2 := newTestRNG(42)

	sum1, sum2 := 0, 0
	for i := 0; i < 1000; i++ {
		sum1 += (&FeastReveler{}).OrderMinute(rng1, 1)
		sum2 += (&FeastReveler{}).OrderMinute(rng2, 100)
	}
	mean1 := float64(sum1) / 1000.0
	mean2 := float64(sum2) / 1000.0

	if mean2 <= mean1 {
		t.Errorf("FeastReveler favNum=100 mean (%.1f) should be > favNum=1 mean (%.1f)", mean2, mean1)
	}
}

func TestOrderMinuteClampNonNegative(t *testing.T) {
	rng := newTestRNG(99)
	a := &Artificer{}
	for i := 0; i < 10000; i++ {
		m := a.OrderMinute(rng, 50)
		if m < 0 {
			t.Fatalf("OrderMinute returned %d on iteration %d, expected >= 0", m, i)
		}
	}
}

func TestOrderItemsCounts(t *testing.T) {
	t.Run("Courier always 1 elixir", func(t *testing.T) {
		rng := newTestRNG(42)
		for i := 0; i < 100; i++ {
			items := (&Courier{}).OrderItems(rng, 50, mockGetItems)
			if len(items) != 1 {
				t.Fatalf("Courier OrderItems returned %d items, want 1", len(items))
			}
			if items[0].Type != Elixir {
				t.Fatal("Courier OrderItems should return an elixir")
			}
		}
	})

	t.Run("Herbalist always 1 elixir", func(t *testing.T) {
		rng := newTestRNG(42)
		for i := 0; i < 100; i++ {
			items := (&Herbalist{}).OrderItems(rng, 50, mockGetItems)
			if len(items) != 1 {
				t.Fatalf("Herbalist OrderItems returned %d items, want 1", len(items))
			}
			if items[0].Type != Elixir {
				t.Fatal("Herbalist OrderItems should return an elixir")
			}
		}
	})

	t.Run("Artificer at least 1 elixir", func(t *testing.T) {
		rng := newTestRNG(42)
		for i := 0; i < 100; i++ {
			items := (&Artificer{}).OrderItems(rng, 50, mockGetItems)
			if len(items) < 1 {
				t.Fatal("Artificer OrderItems should return at least 1 item")
			}
			if items[0].Type != Elixir {
				t.Fatal("Artificer first item should be an elixir")
			}
			// Max: 1 base elixir + 1 extra elixir + 1 solid = 3
			if len(items) > 3 {
				t.Fatalf("Artificer OrderItems returned %d items, want <= 3", len(items))
			}
		}
	})

	t.Run("FeastReveler count scales with favNum", func(t *testing.T) {
		rng := newTestRNG(42)
		// favNum=80 -> count = 1 + 80/20 = 5 -> 5 solid + 5 elixirs = 10
		items := (&FeastReveler{}).OrderItems(rng, 80, mockGetItems)
		expectedCount := (1 + 80/20) * 2 // solid + elixirs
		if len(items) != expectedCount {
			t.Errorf("FeastReveler OrderItems(favNum=80) returned %d items, want %d", len(items), expectedCount)
		}
		// Verify mix: should have solid items (weapon or armor) and elixirs
		solidCount, elixirCount := 0, 0
		for _, item := range items {
			switch item.Type {
			case Weapon, Armor:
				solidCount++
			case Elixir:
				elixirCount++
			}
		}
		if solidCount != 1+80/20 || elixirCount != 1+80/20 {
			t.Errorf("FeastReveler: got %d solid and %d elixirs, want %d each", solidCount, elixirCount, 1+80/20)
		}
	})

	t.Run("Apprentice at least 1 elixir, sometimes solid", func(t *testing.T) {
		rng := newTestRNG(42)
		gotSolid := false
		for i := 0; i < 100; i++ {
			items := (&Apprentice{}).OrderItems(rng, 50, mockGetItems)
			if len(items) < 1 || len(items) > 2 {
				t.Fatalf("Apprentice OrderItems returned %d items, want 1 or 2", len(items))
			}
			if items[0].Type != Elixir {
				t.Fatal("Apprentice first item should be an elixir")
			}
			if len(items) == 2 {
				if items[1].Type != Weapon && items[1].Type != Armor {
					t.Fatal("Apprentice second item should be a weapon or armor")
				}
				gotSolid = true
			}
		}
		if !gotSolid {
			t.Error("Apprentice never ordered a solid item in 100 iterations, expected ~50% chance")
		}
	})

	t.Run("Wanderer variable count", func(t *testing.T) {
		rng := newTestRNG(42)
		gotEmpty := false
		gotMultiple := false
		for i := 0; i < 200; i++ {
			items := (&Wanderer{}).OrderItems(rng, 50, mockGetItems)
			if len(items) == 0 {
				gotEmpty = true
			}
			if len(items) > 1 {
				gotMultiple = true
			}
			// Max is 3 elixirs + 3 solid = 6
			if len(items) > 6 {
				t.Fatalf("Wanderer OrderItems returned %d items, want <= 6", len(items))
			}
		}
		if !gotEmpty {
			t.Error("Wanderer never returned empty order in 200 iterations")
		}
		if !gotMultiple {
			t.Error("Wanderer never returned multiple items in 200 iterations")
		}
	})
}

func TestPersonaMixWeightsSumToOne(t *testing.T) {
	sum := 0.0
	for _, pw := range PersonaMix {
		sum += pw.Weight
	}
	if math.Abs(sum-1.0) > 1e-9 {
		t.Errorf("PersonaMix weights sum to %f, want 1.0", sum)
	}
}

func TestPersonaMixFactories(t *testing.T) {
	expected := []string{"Courier", "Artificer", "FeastReveler", "Apprentice", "Wanderer", "Herbalist"}
	if len(PersonaMix) != len(expected) {
		t.Fatalf("PersonaMix has %d entries, want %d", len(PersonaMix), len(expected))
	}
	for i, pw := range PersonaMix {
		p := pw.NewPersona()
		if p.Name() != expected[i] {
			t.Errorf("PersonaMix[%d].Name() = %q, want %q", i, p.Name(), expected[i])
		}
	}
}

func TestPTweetValues(t *testing.T) {
	tests := []struct {
		name string
		p    Persona
		want float64
	}{
		{"Courier", &Courier{}, 0.2},
		{"Artificer", &Artificer{}, 0.01},
		{"FeastReveler", &FeastReveler{}, 0.8},
		{"Apprentice", &Apprentice{}, 0.8},
		{"Wanderer", &Wanderer{}, 0.1},
		{"Herbalist", &Herbalist{}, 0.6},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := tt.p.PTweet()
			if got != tt.want {
				t.Errorf("PTweet() = %f, want %f", got, tt.want)
			}
		})
	}
}

func TestGuildRankFromOrders(t *testing.T) {
	tests := []struct {
		orders int
		want   GuildRank
	}{
		{0, Initiate},
		{4, Initiate},
		{5, Journeyman},
		{14, Journeyman},
		{15, Adept},
		{29, Adept},
		{30, Master},
		{100, Master},
	}
	for _, tt := range tests {
		got := GuildRankFromOrders(tt.orders)
		if got != tt.want {
			t.Errorf("GuildRankFromOrders(%d) = %s, want %s", tt.orders, got, tt.want)
		}
	}
}
