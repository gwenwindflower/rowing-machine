package main

import (
	"crypto/rand"
	"encoding/binary"
	"fmt"
	"os"
	"time"

	"rowing-machine/internal/simulation"

	"github.com/spf13/cobra"
)

func main() {
	var cfg simulation.Config
	var seedFlag int64
	var startDateStr string

	rootCmd := &cobra.Command{
		Use:   "rowing-machine",
		Short: "Synthetic data generator for Rowing Outfitters",
		Long:  "Generates realistic synthetic relational data (customers, orders, items, stores, products, supplies, tweets) simulating a chain of outdoors supply stores.",
		RunE: func(cmd *cobra.Command, args []string) error {
			// Parse start date
			t, err := time.Parse("2006-01-02", startDateStr)
			if err != nil {
				return fmt.Errorf("invalid start date %q: %w", startDateStr, err)
			}
			cfg.StartDate = t

			// Seed handling: 0 means pick a random seed
			if seedFlag == 0 {
				var b [8]byte
				if _, err := rand.Read(b[:]); err != nil {
					return fmt.Errorf("generating random seed: %w", err)
				}
				cfg.Seed = binary.LittleEndian.Uint64(b[:])
			} else {
				cfg.Seed = uint64(seedFlag)
			}

			return simulation.Run(cfg)
		},
	}

	flags := rootCmd.Flags()
	flags.IntVar(&cfg.Years, "years", 3, "Number of years to simulate (365 days each)")
	flags.IntVar(&cfg.Scale, "scale", 100, "Customer pool multiplier")
	flags.Int64Var(&seedFlag, "seed", 0, "Random seed (0 = random, prints chosen seed)")
	flags.StringVar(&startDateStr, "start-date", "2018-09-01", "Simulation epoch date (YYYY-MM-DD)")
	flags.StringVar(&cfg.OutputDir, "output-dir", "./factory-output", "Output directory")
	flags.StringVar(&cfg.Prefix, "pre", "raw", "Filename prefix")
	flags.BoolVar(&cfg.Quiet, "quiet", false, "Suppress progress output")

	if err := rootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}
