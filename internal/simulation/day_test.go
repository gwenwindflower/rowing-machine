package simulation

import (
	"testing"
	"time"

	"rowing-machine/internal/models"
)

func TestSeasonFromDate_Boundaries(t *testing.T) {
	tests := []struct {
		name string
		date time.Time
		want models.Season
	}{
		{"Jan 1 is Winter", time.Date(2020, 1, 1, 0, 0, 0, 0, time.UTC), models.Winter},
		{"Mar 20 is Winter", time.Date(2020, 3, 20, 0, 0, 0, 0, time.UTC), models.Winter},
		{"Mar 21 is Spring", time.Date(2020, 3, 21, 0, 0, 0, 0, time.UTC), models.Spring},
		{"Jun 20 is Spring", time.Date(2020, 6, 20, 0, 0, 0, 0, time.UTC), models.Spring},
		{"Jun 21 is Summer", time.Date(2020, 6, 21, 0, 0, 0, 0, time.UTC), models.Summer},
		{"Sep 20 is Summer", time.Date(2020, 9, 20, 0, 0, 0, 0, time.UTC), models.Summer},
		{"Sep 21 is Fall", time.Date(2020, 9, 21, 0, 0, 0, 0, time.UTC), models.Fall},
		{"Dec 20 is Fall", time.Date(2020, 12, 20, 0, 0, 0, 0, time.UTC), models.Fall},
		{"Dec 21 is Winter", time.Date(2020, 12, 21, 0, 0, 0, 0, time.UTC), models.Winter},
		{"Dec 31 is Winter", time.Date(2020, 12, 31, 0, 0, 0, 0, time.UTC), models.Winter},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			got := SeasonFromDate(tc.date)
			if got != tc.want {
				t.Errorf("SeasonFromDate(%s) = %v, want %v",
					tc.date.Format("Jan 2"), got, tc.want)
			}
		})
	}
}

func TestSeasonFromDate_MidSeasonDates(t *testing.T) {
	tests := []struct {
		name string
		date time.Time
		want models.Season
	}{
		{"Feb 15 is Winter", time.Date(2020, 2, 15, 0, 0, 0, 0, time.UTC), models.Winter},
		{"May 1 is Spring", time.Date(2020, 5, 1, 0, 0, 0, 0, time.UTC), models.Spring},
		{"Aug 1 is Summer", time.Date(2020, 8, 1, 0, 0, 0, 0, time.UTC), models.Summer},
		{"Nov 1 is Fall", time.Date(2020, 11, 1, 0, 0, 0, 0, time.UTC), models.Fall},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			got := SeasonFromDate(tc.date)
			if got != tc.want {
				t.Errorf("SeasonFromDate(%s) = %v, want %v",
					tc.date.Format("Jan 2"), got, tc.want)
			}
		})
	}
}

func TestPrecomputeDays_Count(t *testing.T) {
	start := time.Date(2018, 9, 1, 0, 0, 0, 0, time.UTC)
	years := 3
	numDays := years * 365
	days := PrecomputeDays(start, numDays)

	if len(days) != numDays {
		t.Errorf("PrecomputeDays returned %d days, want %d", len(days), numDays)
	}
}

func TestPrecomputeDays_FirstDay(t *testing.T) {
	start := time.Date(2018, 9, 1, 0, 0, 0, 0, time.UTC)
	days := PrecomputeDays(start, 10)

	if !days[0].Date.Equal(start) {
		t.Errorf("first day date = %v, want %v", days[0].Date, start)
	}
	if days[0].Index != 0 {
		t.Errorf("first day index = %d, want 0", days[0].Index)
	}
}

func TestPrecomputeDays_SequentialIndices(t *testing.T) {
	start := time.Date(2018, 9, 1, 0, 0, 0, 0, time.UTC)
	days := PrecomputeDays(start, 100)

	for i, d := range days {
		if d.Index != i {
			t.Errorf("day[%d].Index = %d, want %d", i, d.Index, i)
		}
	}
}

func TestPrecomputeDays_WeekendHours(t *testing.T) {
	// Sep 1, 2018 is a Saturday.
	start := time.Date(2018, 9, 1, 0, 0, 0, 0, time.UTC)
	days := PrecomputeDays(start, 7)

	for _, d := range days {
		if d.IsWeekend {
			if d.OpensAt != 480 {
				t.Errorf("%s (weekend): OpensAt = %d, want 480",
					d.Date.Format("Mon Jan 2"), d.OpensAt)
			}
			if d.ClosesAt != 900 {
				t.Errorf("%s (weekend): ClosesAt = %d, want 900",
					d.Date.Format("Mon Jan 2"), d.ClosesAt)
			}
		} else {
			if d.OpensAt != 420 {
				t.Errorf("%s (weekday): OpensAt = %d, want 420",
					d.Date.Format("Mon Jan 2"), d.OpensAt)
			}
			if d.ClosesAt != 1200 {
				t.Errorf("%s (weekday): ClosesAt = %d, want 1200",
					d.Date.Format("Mon Jan 2"), d.ClosesAt)
			}
		}
	}
}

func TestPrecomputeDays_WeekendDetection(t *testing.T) {
	// Sep 1, 2018 is Saturday, Sep 2 Sunday, Sep 3 Monday.
	start := time.Date(2018, 9, 1, 0, 0, 0, 0, time.UTC)
	days := PrecomputeDays(start, 7)

	expectations := []struct {
		dayName   string
		isWeekend bool
	}{
		{"Sat", true},
		{"Sun", true},
		{"Mon", false},
		{"Tue", false},
		{"Wed", false},
		{"Thu", false},
		{"Fri", false},
	}

	for i, exp := range expectations {
		if days[i].IsWeekend != exp.isWeekend {
			t.Errorf("day %d (%s): IsWeekend = %v, want %v",
				i, exp.dayName, days[i].IsWeekend, exp.isWeekend)
		}
	}
}

func TestPrecomputeDays_EffectBounds(t *testing.T) {
	// Over a 3-year simulation, all effects should be positive and within
	// reasonable bounds.
	start := time.Date(2018, 9, 1, 0, 0, 0, 0, time.UTC)
	numDays := 3 * 365
	days := PrecomputeDays(start, numDays)

	for _, d := range days {
		if d.Effect <= 0 {
			t.Errorf("day %d (%s): Effect = %f, want > 0",
				d.Index, d.Date.Format("2006-01-02"), d.Effect)
		}
		if d.Effect < 0.3 || d.Effect > 3.0 {
			t.Errorf("day %d (%s): Effect = %f, want in (0.3, 3.0)",
				d.Index, d.Date.Format("2006-01-02"), d.Effect)
		}
	}
}

func TestPrecomputeDays_EffectComposition(t *testing.T) {
	// Verify the effect is actually the product of the three curves.
	start := time.Date(2018, 9, 1, 0, 0, 0, 0, time.UTC)
	days := PrecomputeDays(start, 30)

	for _, d := range days {
		expected := AnnualCurve(d.Date) * WeekendCurve(d.Date.Weekday()) * GrowthCurve(d.Date)
		if d.Effect != expected {
			t.Errorf("day %d: Effect = %f, want %f (product of curves)",
				d.Index, d.Effect, expected)
		}
	}
}

func TestPrecomputeDays_SeasonAssignment(t *testing.T) {
	// Verify that seasons are correctly assigned by spot-checking dates.
	start := time.Date(2020, 1, 1, 0, 0, 0, 0, time.UTC)
	days := PrecomputeDays(start, 365)

	// Jan 15 (index 14) should be Winter.
	if days[14].Season != models.Winter {
		t.Errorf("Jan 15: Season = %v, want Winter", days[14].Season)
	}

	// Apr 15 (index 105, since Jan=31+Feb=29+Mar=31+15-1=105) should be Spring.
	// 2020 is a leap year: Jan(31)+Feb(29)+Mar(31)+Apr(15)-Jan(1) = 105
	if days[105].Season != models.Spring {
		t.Errorf("Apr 15 (index 105): Season = %v, want Spring", days[105].Season)
	}

	// Jul 15 should be Summer.
	// Jan(31)+Feb(29)+Mar(31)+Apr(30)+May(31)+Jun(30)+Jul(15)-1 = 196
	if days[196].Season != models.Summer {
		t.Errorf("Jul 15 (index 196): Season = %v, want Summer", days[196].Season)
	}

	// Oct 15 should be Fall.
	// +Aug(31)+Sep(30)+Oct(15) = 196+31+30+15 = 288
	if days[288].Season != models.Fall {
		t.Errorf("Oct 15 (index 288): Season = %v, want Fall", days[288].Season)
	}
}
