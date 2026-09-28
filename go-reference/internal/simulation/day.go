package simulation

import (
	"time"

	"rowing-machine/internal/models"
)

// SeasonFromDate determines the season for a given date.
//
//	Winter: Jan 1 - Mar 20, Dec 21 - Dec 31
//	Spring: Mar 21 - Jun 20
//	Summer: Jun 21 - Sep 20
//	Fall:   Sep 21 - Dec 20
func SeasonFromDate(t time.Time) models.Season {
	month := t.Month()
	day := t.Day()

	switch {
	case month <= time.March && !(month == time.March && day >= 21):
		return models.Winter
	case (month == time.March && day >= 21) || month <= time.June && !(month == time.June && day >= 21):
		return models.Spring
	case (month == time.June && day >= 21) || month <= time.September && !(month == time.September && day >= 21):
		return models.Summer
	case (month == time.September && day >= 21) || month <= time.December && !(month == time.December && day >= 21):
		return models.Fall
	default:
		// Dec 21 - Dec 31
		return models.Winter
	}
}

// DayState holds all pre-computed information for a single simulation day.
type DayState struct {
	Index     int           // 0-based day index from epoch
	Date      time.Time     // actual calendar date
	IsWeekend bool          // Saturday or Sunday
	Season    models.Season // season for this date
	Effect    float64       // product of AnnualCurve * WeekendCurve * GrowthCurve
	OpensAt   int           // store opening time in minutes from midnight
	ClosesAt  int           // store closing time in minutes from midnight
}

// PrecomputeDays builds the full []DayState slice for the simulation.
// numDays = years * 365.
//
// Hours of operation:
//
//	Weekday: 07:00 (420 min) to 20:00 (1200 min)
//	Weekend: 08:00 (480 min) to 15:00 (900 min)
func PrecomputeDays(startDate time.Time, numDays int) []DayState {
	days := make([]DayState, numDays)

	for i := range numDays {
		date := startDate.AddDate(0, 0, i)
		wd := date.Weekday()
		isWeekend := wd == time.Saturday || wd == time.Sunday

		opensAt := 420   // 07:00
		closesAt := 1200 // 20:00
		if isWeekend {
			opensAt = 480  // 08:00
			closesAt = 900 // 15:00
		}

		effect := AnnualCurve(date) * WeekendCurve(wd) * GrowthCurve(date)

		days[i] = DayState{
			Index:     i,
			Date:      date,
			IsWeekend: isWeekend,
			Season:    SeasonFromDate(date),
			Effect:    effect,
			OpensAt:   opensAt,
			ClosesAt:  closesAt,
		}
	}

	return days
}
