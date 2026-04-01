package models

import (
	"math/rand/v2"
	"strings"
	"testing"
	"time"
)

func TestBuildItemsSentence(t *testing.T) {
	items := []Item{
		{Name: "vanilla ice"},
		{Name: "tangaroo"},
		{Name: "chai and mighty"},
	}

	tests := []struct {
		name  string
		items []Item
		want  string
	}{
		{"zero items", nil, ""},
		{"one item", items[:1], "Ordered a vanilla ice"},
		{"two items", items[:2], "Ordered a vanilla ice and a tangaroo"},
		{"three items", items[:3], "Ordered a vanilla ice, a tangaroo, and a chai and mighty"},
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
	items := []Item{{Name: "vanilla ice"}}

	tests := []struct {
		name      string
		fanLevel  int
		wantStart string
		wantSub   string
	}{
		{
			name:      "positive fan (level 5)",
			fanLevel:  5,
			wantStart: "Jaffles from the Jaffle Shop are",
			wantSub:   "",
		},
		{
			name:      "positive fan (level 4)",
			fanLevel:  4,
			wantStart: "Jaffles from the Jaffle Shop are",
			wantSub:   "",
		},
		{
			name:      "negative fan (level 1)",
			fanLevel:  1,
			wantStart: "Jaffle Shop again.",
			wantSub:   "This place is",
		},
		{
			name:      "negative fan (level 2)",
			fanLevel:  2,
			wantStart: "Jaffle Shop again.",
			wantSub:   "This place is",
		},
		{
			name:      "neutral fan (level 3)",
			fanLevel:  3,
			wantStart: "Jaffle shop is",
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

func TestNewTweet_DelayRange(t *testing.T) {
	orderedAt := time.Date(2020, 6, 15, 12, 0, 0, 0, time.UTC)
	custID := [16]byte{1}
	items := []Item{{Name: "test item", Price: 100}}

	// Run many iterations to check delay is always in [0, 19] minutes.
	for seed := uint64(0); seed < 100; seed++ {
		rng := rand.New(rand.NewPCG(seed, 0))
		tweet := NewTweet(rng, custID, 3, items, orderedAt)

		delay := tweet.TweetedAt.Sub(orderedAt)
		if delay < 0 || delay >= 20*time.Minute {
			t.Errorf("seed %d: delay = %v, want [0, 20m)", seed, delay)
		}
	}
}

func TestNewTweet_Deterministic(t *testing.T) {
	orderedAt := time.Date(2020, 6, 15, 12, 0, 0, 0, time.UTC)
	custID := [16]byte{5}
	items := []Item{{Name: "vanilla ice", Price: 500}}

	rng1 := rand.New(rand.NewPCG(999, 0))
	tweet1 := NewTweet(rng1, custID, 4, items, orderedAt)

	rng2 := rand.New(rand.NewPCG(999, 0))
	tweet2 := NewTweet(rng2, custID, 4, items, orderedAt)

	if tweet1.ID != tweet2.ID {
		t.Errorf("same seed produced different IDs: %v vs %v", tweet1.ID, tweet2.ID)
	}
	if tweet1.Content != tweet2.Content {
		t.Errorf("same seed produced different content: %q vs %q", tweet1.Content, tweet2.Content)
	}
	if !tweet1.TweetedAt.Equal(tweet2.TweetedAt) {
		t.Errorf("same seed produced different times: %v vs %v", tweet1.TweetedAt, tweet2.TweetedAt)
	}
}

func TestNewTweet_UserIDSet(t *testing.T) {
	rng := rand.New(rand.NewPCG(42, 0))
	custID := [16]byte{0xDE, 0xAD}
	items := []Item{{Name: "test", Price: 100}}
	orderedAt := time.Date(2021, 1, 1, 0, 0, 0, 0, time.UTC)

	tweet := NewTweet(rng, custID, 3, items, orderedAt)
	if tweet.UserID != custID {
		t.Errorf("UserID = %v, want %v", tweet.UserID, custID)
	}
}

func TestBuildContent_IncludesItemsSentence(t *testing.T) {
	items := []Item{
		{Name: "vanilla ice"},
		{Name: "tangaroo"},
	}

	rng := rand.New(rand.NewPCG(42, 0))
	content := buildContent(rng, 5, items)

	if !strings.Contains(content, "Ordered a vanilla ice and a tangaroo") {
		t.Errorf("content = %q, does not contain items sentence", content)
	}
}
