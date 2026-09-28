package main

import (
	"strings"
	"testing"
	"time"

	"rowing-machine/internal/simulation"
)

func TestDefaultConfigurationCovers2023Through2026(t *testing.T) {
	var got simulation.Config
	cmd := newRootCommand(func(cfg simulation.Config) error {
		got = cfg
		return nil
	}, strings.NewReader("unused"))
	cmd.SetArgs([]string{"--seed", "42", "--quiet"})

	if err := cmd.Execute(); err != nil {
		t.Fatalf("Execute: %v", err)
	}

	wantStart := time.Date(2023, time.January, 1, 0, 0, 0, 0, time.UTC)
	if !got.StartDate.Equal(wantStart) {
		t.Errorf("StartDate = %s, want %s", got.StartDate.Format(time.DateOnly), wantStart.Format(time.DateOnly))
	}
	if got.Years != 4 {
		t.Errorf("Years = %d, want 4", got.Years)
	}
	lastDate := got.StartDate.AddDate(0, 0, got.Years*365-1)
	if lastDate.Year() != 2026 {
		t.Errorf("default last date = %s, want a date in 2026", lastDate.Format(time.DateOnly))
	}
}

func TestInvalidGenerationSizesFailBeforeRun(t *testing.T) {
	tests := []struct {
		name string
		args []string
		want string
	}{
		{name: "zero years", args: []string{"--years", "0"}, want: `--years must be greater than zero; got 0`},
		{name: "negative scale", args: []string{"--scale", "-2"}, want: `--scale must be greater than zero; got -2`},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			runCalled := false
			cmd := newRootCommand(func(simulation.Config) error {
				runCalled = true
				return nil
			}, strings.NewReader("unused"))
			cmd.SetArgs(append(tt.args, "--seed", "42", "--quiet"))

			err := cmd.Execute()
			if err == nil || !strings.Contains(err.Error(), tt.want) {
				t.Fatalf("Execute error = %v, want containing %q", err, tt.want)
			}
			if runCalled {
				t.Fatal("simulation ran with invalid configuration")
			}
		})
	}
}
