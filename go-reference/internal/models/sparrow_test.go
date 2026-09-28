package models

import (
	"math/rand/v2"
	"strings"
	"testing"
	"time"
)

func TestBuildItemsSentence(t *testing.T) {
	items := []Item{
		{Name: "frostmint vial"},
		{Name: "sunfire tonic"},
		{Name: "ironbark draught"},
	}

	tests := []struct {
		name  string
		items []Item
		want  string
	}{
		{"zero items", nil, ""},
		{"one item", items[:1], "Acquired a frostmint vial"},
		{"two items", items[:2], "Acquired a frostmint vial and a sunfire tonic"},
		{"three items", items[:3], "Acquired a frostmint vial, a sunfire tonic, and a ironbark draught"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := buildItemsSentence(tt.items)
			if got != tt.want {
				t.Errorf("buildItemsSentence() = %q, want %q", got, tt.want)
			}
		})
	}
}

func TestBuildContent_FanLevels(t *testing.T) {
	items := []Item{{Name: "frostmint vial"}}

	tests := []struct {
		name      string
		fanLevel  int
		wantStart string
		wantSub   string
	}{
		{
			name:      "positive fan (level 5)",
			fanLevel:  5,
			wantStart: "Wares from the Arcanum Collective are",
			wantSub:   "",
		},
		{
			name:      "positive fan (level 4)",
			fanLevel:  4,
			wantStart: "Wares from the Arcanum Collective are",
			wantSub:   "",
		},
		{
			name:      "negative fan (level 1)",
			fanLevel:  1,
			wantStart: "Arcanum Collective again.",
			wantSub:   "Their craft is",
		},
		{
			name:      "negative fan (level 2)",
			fanLevel:  2,
			wantStart: "Arcanum Collective again.",
			wantSub:   "Their craft is",
		},
		{
			name:      "neutral fan (level 3)",
			fanLevel:  3,
			wantStart: "The Arcanum Collective is",
			wantSub:   "",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			rng := rand.New(rand.NewPCG(42, 0))
			got := buildContent(rng, tt.fanLevel, items)

			if !strings.HasPrefix(got, tt.wantStart) {
				t.Errorf("content = %q, want prefix %q", got, tt.wantStart)
			}
			if tt.wantSub != "" && !strings.Contains(got, tt.wantSub) {
				t.Errorf("content = %q, want substring %q", got, tt.wantSub)
			}
		})
	}
}

func TestBuildContent_ContainsAdjective(t *testing.T) {
	items := []Item{{Name: "test item"}}

	tests := []struct {
		name     string
		fanLevel int
		pool     []string
	}{
		{"positive", 5, positiveAdj},
		{"negative", 1, negativeAdj},
		{"neutral", 3, neutralAdj},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			rng := rand.New(rand.NewPCG(77, 0))
			got := buildContent(rng, tt.fanLevel, items)

			found := false
			for _, adj := range tt.pool {
				if strings.Contains(got, adj) {
					found = true
					break
				}
			}
			if !found {
				t.Errorf("content = %q, does not contain any adjective from pool", got)
			}
		})
	}
}

func TestNewSparrow_DelayRange(t *testing.T) {
	orderedAt := time.Date(2020, 6, 15, 12, 0, 0, 0, time.UTC)
	custID := [16]byte{1}
	items := []Item{{Name: "test item", Price: 100}}

	for seed := uint64(0); seed < 100; seed++ {
		rng := rand.New(rand.NewPCG(seed, 0))
		sparrow := NewSparrow(rng, custID, 3, items, orderedAt)

		delay := sparrow.SentAt.Sub(orderedAt)
		if delay < 0 || delay >= 20*time.Minute {
			t.Errorf("seed %d: delay = %v, want [0, 20m)", seed, delay)
		}
	}
}

func TestNewSparrow_Deterministic(t *testing.T) {
	orderedAt := time.Date(2020, 6, 15, 12, 0, 0, 0, time.UTC)
	custID := [16]byte{5}
	items := []Item{{Name: "frostmint vial", Price: 500}}

	rng1 := rand.New(rand.NewPCG(999, 0))
	s1 := NewSparrow(rng1, custID, 4, items, orderedAt)

	rng2 := rand.New(rand.NewPCG(999, 0))
	s2 := NewSparrow(rng2, custID, 4, items, orderedAt)

	if s1.ID != s2.ID {
		t.Errorf("same seed produced different IDs: %v vs %v", s1.ID, s2.ID)
	}
	if s1.Content != s2.Content {
		t.Errorf("same seed produced different content: %q vs %q", s1.Content, s2.Content)
	}
	if !s1.SentAt.Equal(s2.SentAt) {
		t.Errorf("same seed produced different times: %v vs %v", s1.SentAt, s2.SentAt)
	}
}

func TestNewSparrow_UserIDSet(t *testing.T) {
	rng := rand.New(rand.NewPCG(42, 0))
	custID := [16]byte{0xDE, 0xAD}
	items := []Item{{Name: "test", Price: 100}}
	orderedAt := time.Date(2021, 1, 1, 0, 0, 0, 0, time.UTC)

	sparrow := NewSparrow(rng, custID, 3, items, orderedAt)
	if sparrow.UserID != custID {
		t.Errorf("UserID = %v, want %v", sparrow.UserID, custID)
	}
}

func TestBuildContent_IncludesItemsSentence(t *testing.T) {
	items := []Item{
		{Name: "frostmint vial"},
		{Name: "sunfire tonic"},
	}

	rng := rand.New(rand.NewPCG(42, 0))
	content := buildContent(rng, 5, items)

	if !strings.Contains(content, "Acquired a frostmint vial and a sunfire tonic") {
		t.Errorf("content = %q, does not contain items sentence", content)
	}
}
