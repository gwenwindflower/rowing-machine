package simulation

import (
	"math"
	"time"
)

// AnnualCurve computes the seasonality effect for a given date.
// Formula: (cos(x) + 1) / 10 + 0.8
// where x = (dayOfYear - 1) / 365 * 2 * pi
// Range: [0.8, 1.0]. Peak at start of year (Jan 1), trough at mid-year.
func AnnualCurve(d time.Time) float64 {
	dayOfYear := float64(d.YearDay() - 1)
	x := dayOfYear / 365.0 * 2.0 * math.Pi
	return (math.Cos(x)+1.0)/10.0 + 0.8
}

// WeekendCurve returns the day-of-week effect.
// Returns 0.6 on Saturday/Sunday, 1.0 on weekdays.
// This fixes Python bug #1 where it always returned 1.0.
func WeekendCurve(wd time.Weekday) float64 {
	if wd == time.Saturday || wd == time.Sunday {
		return 0.6
	}
	return 1.0
}

// GrowthCurve computes the long-term growth multiplier for a given date.
// Formula: 1 + (monthOffset / 12) * 0.2
// where monthOffset = (year - 2016) * 12 + month
func GrowthCurve(d time.Time) float64 {
	monthOffset := float64((d.Year()-2016)*12 + int(d.Month()))
	return 1.0 + (monthOffset/12.0)*0.2
}
