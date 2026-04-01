package catalog

import (
	"math/rand/v2"
	"strings"
	"testing"
)

func TestFirstNamesCount(t *testing.T) {
	if got := len(FirstNames); got < 400 {
		t.Errorf("len(FirstNames) = %d, want >= 400", got)
	}
}

func TestLastNamesCount(t *testing.T) {
	if got := len(LastNames); got < 400 {
		t.Errorf("len(LastNames) = %d, want >= 400", got)
	}
}

func TestGenerateNameFormat(t *testing.T) {
	rng := rand.New(rand.NewPCG(42, 0))

	for i := range 100 {
		name := GenerateName(rng)
		parts := strings.SplitN(name, " ", 2)
		if len(parts) != 2 {
			t.Errorf("name %d: %q does not contain a space", i, name)
			continue
		}
		if parts[0] == "" || parts[1] == "" {
			t.Errorf("name %d: %q has empty first or last name", i, name)
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
