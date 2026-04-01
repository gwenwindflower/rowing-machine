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
		// Commuter weekday
		{"Commuter weekday favNum=1", &Commuter{}, false, Winter, 1, 0.5, 0.51},
		{"Commuter weekday favNum=100", &Commuter{}, false, Winter, 100, 0.79, 0.81},
		{"Commuter weekend", &Commuter{}, true, Winter, 50, 0.001, 0.001},

		// RemoteWorker weekday
		{"RemoteWorker weekday favNum=1", &RemoteWorker{}, false, Winter, 1, 0.0, 0.01},
		{"RemoteWorker weekday favNum=100", &RemoteWorker{}, false, Winter, 100, 0.39, 0.41},
		{"RemoteWorker weekend", &RemoteWorker{}, true, Winter, 50, 0.001, 0.001},

		// BrunchCrowd
		{"BrunchCrowd weekday", &BrunchCrowd{}, false, Winter, 50, 0.0, 0.0},
		{"BrunchCrowd weekend favNum=1", &BrunchCrowd{}, true, Winter, 1, 0.20, 0.21},
		{"BrunchCrowd weekend favNum=100", &BrunchCrowd{}, true, Winter, 100, 0.39, 0.41},

		// Student
		{"Student summer", &Student{}, false, Summer, 50, 0.0, 0.0},
		{"Student winter favNum=1", &Student{}, false, Winter, 1, 0.10, 0.11},
		{"Student winter favNum=100", &Student{}, false, Winter, 100, 0.49, 0.51},

		// Casuals — constant
		{"Casuals weekday favNum=1", &Casuals{}, false, Winter, 1, 0.1, 0.1},
		{"Casuals weekend favNum=100", &Casuals{}, true, Summer, 100, 0.1, 0.1},

		// HealthNut
		{"HealthNut summer favNum=1", &HealthNut{}, false, Summer, 1, 0.10, 0.11},
		{"HealthNut summer favNum=100", &HealthNut{}, false, Summer, 100, 0.49, 0.51},
		{"HealthNut winter", &HealthNut{}, false, Winter, 50, 0.2, 0.2},
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

func TestCommuterWeekdayVsWeekend(t *testing.T) {
	c := &Commuter{}
	weekday := c.PBuyPersona(false, Winter, 50)
	weekend := c.PBuyPersona(true, Winter, 50)

	if weekday < 0.5 || weekday > 0.8 {
		t.Errorf("Commuter weekday PBuy = %f, want in [0.5, 0.8]", weekday)
	}
	if weekend != 0.001 {
		t.Errorf("Commuter weekend PBuy = %f, want 0.001", weekend)
	}
	if weekend >= weekday {
		t.Error("Commuter weekend probability should be much lower than weekday")
	}
}

func TestBrunchCrowdWeekdayZero(t *testing.T) {
	b := &BrunchCrowd{}
	for favNum := 1; favNum <= 100; favNum++ {
		got := b.PBuyPersona(false, Winter, favNum)
		if got != 0 {
			t.Errorf("BrunchCrowd weekday PBuy with favNum=%d = %f, want 0", favNum, got)
		}
	}
}

func TestStudentSummerZero(t *testing.T) {
	s := &Student{}
	for favNum := 1; favNum <= 100; favNum++ {
		got := s.PBuyPersona(false, Summer, favNum)
		if got != 0 {
			t.Errorf("Student summer PBuy with favNum=%d = %f, want 0", favNum, got)
		}
		got = s.PBuyPersona(true, Summer, favNum)
		if got != 0 {
			t.Errorf("Student summer weekend PBuy with favNum=%d = %f, want 0", favNum, got)
		}
	}
}

func TestHealthNutSummerBoost(t *testing.T) {
	h := &HealthNut{}
	// With high favNum, summer should be higher than non-summer.
	summerProb := h.PBuyPersona(false, Summer, 80)
	winterProb := h.PBuyPersona(false, Winter, 80)
	if summerProb <= winterProb {
		t.Errorf("HealthNut summer=%f should be > winter=%f for high favNum", summerProb, winterProb)
	}
}

func TestCasualsConstant(t *testing.T) {
	c := &Casuals{}
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
		got := c.PBuyPersona(s.isWeekend, s.season, s.favNum)
		if got != 0.1 {
			t.Errorf("Casuals PBuy(weekend=%v, season=%d, favNum=%d) = %f, want 0.1",
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
		tolerance  float64 // allowed deviation from expected mean
		checkClamp bool
	}{
		// Commuter: N(450, 30) — THIS IS THE PYTHON BUG FIX.
		// Python used N(60, 30) = 1 AM. Go uses N(450, 30) = 7:30 AM.
		{"Commuter mean near 450 (NOT 60 - Python bug fix)", &Commuter{}, 50, 450, 15, true},
		{"RemoteWorker mean near 420", &RemoteWorker{}, 50, 420, 30, true},
		{"Student mean near 540", &Student{}, 50, 540, 20, true},
		{"Casuals mean near 300", &Casuals{}, 50, 300, 20, true},
		{"HealthNut mean near 300", &HealthNut{}, 50, 300, 20, true},
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

func TestCommuterOrderMinuteNotPythonBug(t *testing.T) {
	// Explicit regression test: Commuter mean must NOT be near 60 (Python bug).
	rng := newTestRNG(123)
	sum := 0
	for i := 0; i < 1000; i++ {
		sum += (&Commuter{}).OrderMinute(rng, 50)
	}
	mean := float64(sum) / 1000.0
	if mean < 200 {
		t.Errorf("Commuter OrderMinute mean = %.1f, appears to have Python bug (mu=60). Expected ~450.", mean)
	}
}

func TestBrunchCrowdOrderMinuteFavNumEffect(t *testing.T) {
	// BrunchCrowd mu depends on favoriteNumber: 300 + (favNum-50)/50 * 120
	// favNum=1 -> mu ~ 182, favNum=100 -> mu ~ 420
	rng1 := newTestRNG(42)
	rng2 := newTestRNG(42)

	sum1, sum2 := 0, 0
	for i := 0; i < 1000; i++ {
		sum1 += (&BrunchCrowd{}).OrderMinute(rng1, 1)
		sum2 += (&BrunchCrowd{}).OrderMinute(rng2, 100)
	}
	mean1 := float64(sum1) / 1000.0
	mean2 := float64(sum2) / 1000.0

	if mean2 <= mean1 {
		t.Errorf("BrunchCrowd favNum=100 mean (%.1f) should be > favNum=1 mean (%.1f)", mean2, mean1)
	}
}

func TestOrderMinuteClampNonNegative(t *testing.T) {
	// Use a wide-sigma persona (RemoteWorker, sigma=180) and run many iterations.
	rng := newTestRNG(99)
	rw := &RemoteWorker{}
	for i := 0; i < 10000; i++ {
		m := rw.OrderMinute(rng, 50)
		if m < 0 {
			t.Fatalf("OrderMinute returned %d on iteration %d, expected >= 0", m, i)
		}
	}
}

func TestOrderItemsCounts(t *testing.T) {
	t.Run("Commuter always 1 beverage", func(t *testing.T) {
		rng := newTestRNG(42)
		for i := 0; i < 100; i++ {
			items := (&Commuter{}).OrderItems(rng, 50, mockGetItems)
			if len(items) != 1 {
				t.Fatalf("Commuter OrderItems returned %d items, want 1", len(items))
			}
			if items[0].Type != Beverage {
				t.Fatal("Commuter OrderItems should return a beverage")
			}
		}
	})

	t.Run("HealthNut always 1 beverage", func(t *testing.T) {
		rng := newTestRNG(42)
		for i := 0; i < 100; i++ {
			items := (&HealthNut{}).OrderItems(rng, 50, mockGetItems)
			if len(items) != 1 {
				t.Fatalf("HealthNut OrderItems returned %d items, want 1", len(items))
			}
			if items[0].Type != Beverage {
				t.Fatal("HealthNut OrderItems should return a beverage")
			}
		}
	})

	t.Run("RemoteWorker at least 1 beverage", func(t *testing.T) {
		rng := newTestRNG(42)
		for i := 0; i < 100; i++ {
			items := (&RemoteWorker{}).OrderItems(rng, 50, mockGetItems)
			if len(items) < 1 {
				t.Fatal("RemoteWorker OrderItems should return at least 1 item")
			}
			if items[0].Type != Beverage {
				t.Fatal("RemoteWorker first item should be a beverage")
			}
			// Max: 1 base bev + 1 extra bev + 1 jaffle = 3
			if len(items) > 3 {
				t.Fatalf("RemoteWorker OrderItems returned %d items, want <= 3", len(items))
			}
		}
	})

	t.Run("BrunchCrowd count scales with favNum", func(t *testing.T) {
		rng := newTestRNG(42)
		// favNum=80 -> count = 1 + 80/20 = 5 -> 5 jaffles + 5 beverages = 10
		items := (&BrunchCrowd{}).OrderItems(rng, 80, mockGetItems)
		expectedCount := (1 + 80/20) * 2 // jaffles + beverages
		if len(items) != expectedCount {
			t.Errorf("BrunchCrowd OrderItems(favNum=80) returned %d items, want %d", len(items), expectedCount)
		}
		// Verify mix of types
		jaffles, beverages := 0, 0
		for _, item := range items {
			switch item.Type {
			case Jaffle:
				jaffles++
			case Beverage:
				beverages++
			}
		}
		if jaffles != 1+80/20 || beverages != 1+80/20 {
			t.Errorf("BrunchCrowd: got %d jaffles and %d beverages, want %d each", jaffles, beverages, 1+80/20)
		}
	})

	t.Run("Student at least 1 beverage, sometimes jaffle", func(t *testing.T) {
		rng := newTestRNG(42)
		gotJaffle := false
		for i := 0; i < 100; i++ {
			items := (&Student{}).OrderItems(rng, 50, mockGetItems)
			if len(items) < 1 || len(items) > 2 {
				t.Fatalf("Student OrderItems returned %d items, want 1 or 2", len(items))
			}
			if items[0].Type != Beverage {
				t.Fatal("Student first item should be a beverage")
			}
			if len(items) == 2 {
				if items[1].Type != Jaffle {
					t.Fatal("Student second item should be a jaffle")
				}
				gotJaffle = true
			}
		}
		if !gotJaffle {
			t.Error("Student never ordered a jaffle in 100 iterations, expected ~50% chance")
		}
	})

	t.Run("Casuals variable count 0-3 each type", func(t *testing.T) {
		rng := newTestRNG(42)
		gotEmpty := false
		gotMultiple := false
		for i := 0; i < 200; i++ {
			items := (&Casuals{}).OrderItems(rng, 50, mockGetItems)
			if len(items) == 0 {
				gotEmpty = true
			}
			if len(items) > 1 {
				gotMultiple = true
			}
			// Max is 3 beverages + 3 jaffles = 6
			if len(items) > 6 {
				t.Fatalf("Casuals OrderItems returned %d items, want <= 6", len(items))
			}
		}
		if !gotEmpty {
			t.Error("Casuals never returned empty order in 200 iterations")
		}
		if !gotMultiple {
			t.Error("Casuals never returned multiple items in 200 iterations")
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
	expected := []string{"Commuter", "RemoteWorker", "BrunchCrowd", "Student", "Casuals", "HealthNut"}
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
		{"Commuter", &Commuter{}, 0.2},
		{"RemoteWorker", &RemoteWorker{}, 0.01},
		{"BrunchCrowd", &BrunchCrowd{}, 0.8},
		{"Student", &Student{}, 0.8},
		{"Casuals", &Casuals{}, 0.1},
		{"HealthNut", &HealthNut{}, 0.6},
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
