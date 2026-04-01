package simulation

import (
	"testing"
)

func TestNewProgress(t *testing.T) {
	p := NewProgress(true, 365)
	if !p.quiet {
		t.Error("expected quiet=true")
	}
	if p.totalDays != 365 {
		t.Errorf("totalDays: got %d, want 365", p.totalDays)
	}
	if p.lastPct != -1 {
		t.Errorf("lastPct: got %d, want -1", p.lastPct)
	}
	if p.startTime.IsZero() {
		t.Error("startTime should not be zero")
	}
}

func TestQuietModeProducesNoOutput(t *testing.T) {
	p := NewProgress(true, 100)

	// These should not panic or produce visible output when quiet.
	p.PrintSeed(42)
	for i := 0; i < 100; i++ {
		p.Update(i)
	}
	p.PrintSummary(map[string]int{
		"stores":    6,
		"customers": 1000,
		"orders":    5000,
		"items":     10000,
		"products":  10,
		"supplies":  29,
		"tweets":    500,
	}, "/tmp/output")
}

func TestUpdateTracksPercentage(t *testing.T) {
	p := NewProgress(true, 100)

	// Simulate a full run in quiet mode; just verify no panic
	// and that lastPct stays at -1 (since quiet suppresses updates).
	for i := 0; i < 100; i++ {
		p.Update(i)
	}
	if p.lastPct != -1 {
		t.Errorf("quiet mode should not update lastPct, got %d", p.lastPct)
	}
}
