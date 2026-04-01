package simulation

import (
	"math"
	"testing"
	"time"
)

func TestAnnualCurve_Range(t *testing.T) {
	// Verify AnnualCurve stays within [0.8, 1.0] for every day of the year.
	for doy := 1; doy <= 365; doy++ {
		d := time.Date(2020, 1, 1, 0, 0, 0, 0, time.UTC).AddDate(0, 0, doy-1)
		val := AnnualCurve(d)
		if val < 0.8-1e-9 || val > 1.0+1e-9 {
			t.Errorf("AnnualCurve day %d (%s): got %f, want in [0.8, 1.0]", doy, d.Format("Jan 2"), val)
		}
	}
}

func TestAnnualCurve_PeakAndTrough(t *testing.T) {
	jan1 := time.Date(2020, 1, 1, 0, 0, 0, 0, time.UTC)
	jul1 := time.Date(2020, 7, 1, 0, 0, 0, 0, time.UTC)

	peak := AnnualCurve(jan1)
	trough := AnnualCurve(jul1)

	// Jan 1 should be at or very near peak (1.0).
	if math.Abs(peak-1.0) > 0.01 {
		t.Errorf("AnnualCurve(Jan 1) = %f, want ~1.0", peak)
	}

	// Mid-year should be near trough (0.8).
	if math.Abs(trough-0.8) > 0.02 {
		t.Errorf("AnnualCurve(Jul 1) = %f, want ~0.8", trough)
	}

	if peak <= trough {
		t.Errorf("peak (%f) should be greater than trough (%f)", peak, trough)
	}
}

func TestWeekendCurve_PythonBugFix(t *testing.T) {
	// This is the Python bug #1 fix: weekends must return 0.6, not 1.0.
	tests := []struct {
		name string
		day  time.Weekday
		want float64
	}{
		{"Monday", time.Monday, 1.0},
		{"Tuesday", time.Tuesday, 1.0},
		{"Wednesday", time.Wednesday, 1.0},
		{"Thursday", time.Thursday, 1.0},
		{"Friday", time.Friday, 1.0},
		{"Saturday", time.Saturday, 0.6},
		{"Sunday", time.Sunday, 0.6},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			got := WeekendCurve(tc.day)
			if got != tc.want {
				t.Errorf("WeekendCurve(%s) = %f, want %f", tc.name, got, tc.want)
			}
		})
	}
}

func TestGrowthCurve_Sep2018(t *testing.T) {
	// Sep 2018: monthOffset = (2018-2016)*12 + 9 = 33
	// growth = 1 + (33/12)*0.2 = 1 + 2.75*0.2 = 1.55
	d := time.Date(2018, 9, 15, 0, 0, 0, 0, time.UTC)
	got := GrowthCurve(d)
	want := 1.55
	if math.Abs(got-want) > 1e-9 {
		t.Errorf("GrowthCurve(Sep 2018) = %f, want %f", got, want)
	}
}

func TestGrowthCurve_MonotonicallyIncreasing(t *testing.T) {
	// Growth should increase month over month.
	prev := GrowthCurve(time.Date(2017, 1, 1, 0, 0, 0, 0, time.UTC))
	for year := 2017; year <= 2025; year++ {
		for month := time.January; month <= time.December; month++ {
			d := time.Date(year, month, 15, 0, 0, 0, 0, time.UTC)
			cur := GrowthCurve(d)
			if year == 2017 && month == time.January {
				continue // skip the starting point
			}
			if cur < prev {
				t.Errorf("GrowthCurve not monotonic: %s (%f) < previous (%f)",
					d.Format("Jan 2006"), cur, prev)
			}
			prev = cur
		}
	}
}

func TestGrowthCurve_KnownValues(t *testing.T) {
	tests := []struct {
		name string
		date time.Time
		want float64
	}{
		{
			name: "Jan 2016",
			date: time.Date(2016, 1, 15, 0, 0, 0, 0, time.UTC),
			// monthOffset = 0*12 + 1 = 1, growth = 1 + (1/12)*0.2
			want: 1.0 + (1.0/12.0)*0.2,
		},
		{
			name: "Dec 2016",
			date: time.Date(2016, 12, 15, 0, 0, 0, 0, time.UTC),
			// monthOffset = 0*12 + 12 = 12, growth = 1 + (12/12)*0.2 = 1.2
			want: 1.2,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			got := GrowthCurve(tc.date)
			if math.Abs(got-tc.want) > 1e-9 {
				t.Errorf("GrowthCurve(%s) = %f, want %f", tc.name, got, tc.want)
			}
		})
	}
}
