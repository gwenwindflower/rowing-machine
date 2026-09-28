package main

import (
	"crypto/rand"
	"encoding/binary"
	"fmt"
	"io"
	"os"
	"time"

	"rowing-machine/internal/simulation"

	"github.com/spf13/cobra"
)

func main() {
	if err := newRootCommand(simulation.Run, rand.Reader).Execute(); err != nil {
		os.Exit(1)
	}
}

func newRootCommand(run func(simulation.Config) error, seedSource io.Reader) *cobra.Command {
	var cfg simulation.Config
	var seedFlag int64
	var startDateStr string

	rootCmd := &cobra.Command{
		Use:   "rowing-machine",
		Short: "Synthetic data generator for Queria",
		Long:  "Generates realistic synthetic relational data (patrons, orders, items, guild halls, products, reagents, sparrows) simulating the Arcanum Collective mage guild across the land of Queria.",
		RunE: func(cmd *cobra.Command, args []string) error {
			t, err := time.Parse("2006-01-02", startDateStr)
			if err != nil {
				return fmt.Errorf("--start-date must use YYYY-MM-DD; got %q: %w", startDateStr, err)
			}
			cfg.StartDate = t
			if err := cfg.Validate(); err != nil {
				return err
			}

			if seedFlag == 0 {
				var b [8]byte
				if _, err := io.ReadFull(seedSource, b[:]); err != nil {
					return fmt.Errorf("generating random seed: %w", err)
				}
				cfg.Seed = binary.LittleEndian.Uint64(b[:])
			} else {
				cfg.Seed = uint64(seedFlag)
			}

			return run(cfg)
		},
	}

	flags := rootCmd.Flags()
	flags.IntVar(&cfg.Years, "years", 4, "Number of years to simulate (365 days each)")
	flags.IntVar(&cfg.Scale, "scale", 100, "Customer pool multiplier")
	flags.Int64Var(&seedFlag, "seed", 0, "Random seed (0 = random, prints chosen seed)")
	flags.StringVar(&startDateStr, "start-date", "2023-01-01", "Simulation epoch date (YYYY-MM-DD)")
	flags.StringVar(&cfg.OutputDir, "output-dir", "./factory-output", "Output directory")
	flags.StringVar(&cfg.Prefix, "pre", "raw", "Filename prefix")
	flags.BoolVar(&cfg.Quiet, "quiet", false, "Suppress progress output")

	return rootCmd
}
