package simulation

import (
	"fmt"
	"time"
)

// Progress tracks and displays simulation progress.
type Progress struct {
	quiet     bool
	startTime time.Time
	totalDays int
	lastPct   int // last percentage printed, to avoid reprinting same value
}

// NewProgress creates a new Progress tracker.
func NewProgress(quiet bool, totalDays int) *Progress {
	return &Progress{
		quiet:     quiet,
		startTime: time.Now(),
		totalDays: totalDays,
		lastPct:   -1,
	}
}

// PrintSeed prints the seed value (unless quiet).
func (p *Progress) PrintSeed(seed uint64) {
	if p.quiet {
		return
	}
	fmt.Printf("Seed: %d\n", seed)
}

// Update prints progress for the given day index (unless quiet).
// Prints at each 10% increment and on the final day.
func (p *Progress) Update(dayIndex int) {
	if p.quiet {
		return
	}
	if p.totalDays <= 0 {
		return
	}

	pct := (dayIndex + 1) * 100 / p.totalDays
	isFinal := dayIndex == p.totalDays-1

	// Print at each 10% boundary or on the final day.
	bucket := pct / 10 * 10
	if bucket == p.lastPct && !isFinal {
		return
	}
	if isFinal {
		bucket = 100
	}
	p.lastPct = bucket
	fmt.Printf("\rSimulating... %d%%", bucket)
}

// entityOrder defines the consistent order for printing summary counts.
var entityOrder = []string{
	"stores", "customers", "orders", "items", "products", "supplies", "tweets",
}

// PrintSummary prints generation summary with row counts and elapsed time.
func (p *Progress) PrintSummary(counts map[string]int, outputDir string) {
	if p.quiet {
		return
	}
	elapsed := time.Since(p.startTime).Round(time.Millisecond)
	fmt.Printf("\nDone in %s\n", elapsed)
	fmt.Printf("Output: %s\n", outputDir)
	for _, entity := range entityOrder {
		if count, ok := counts[entity]; ok {
			fmt.Printf("  %-12s %d rows\n", entity, count)
		}
	}
}
