package catalog

import (
	"math/rand/v2"
	"testing"
)

func TestFullNamesCount(t *testing.T) {
	if got := len(FullNames); got < 400 {
		t.Errorf("len(FullNames) = %d, want >= 400", got)
	}
}

func TestGenerateNameNonEmpty(t *testing.T) {
	rng := rand.New(rand.NewPCG(42, 0))

	for i := range 100 {
		name := GenerateName(rng)
		if name == "" {
			t.Errorf("name %d is empty", i)
		}
	}
}

func TestGenerateNameDeterministic(t *testing.T) {
	rng1 := rand.New(rand.NewPCG(77, 0))
	rng2 := rand.New(rand.NewPCG(77, 0))

	for range 50 {
		name1 := GenerateName(rng1)
		name2 := GenerateName(rng2)
		if name1 != name2 {
			t.Errorf("same seed produced different names: %q vs %q", name1, name2)
		}
	}
}
