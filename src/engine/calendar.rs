use anyhow::{Context, Result};
use jiff::civil::{Date, Weekday};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Season {
    Winter,
    Spring,
    Summer,
    Fall,
}

#[derive(Debug, Clone, Copy)]
pub struct DayState {
    pub index: usize,
    pub date: Date,
    pub is_weekend: bool,
    pub season: Season,
    pub effect: f64,
    pub opens_at: i32,
    pub closes_at: i32,
}

/// Computes the calendar and demand curves for every simulation day.
///
/// # Errors
///
/// Returns an error if the range extends beyond the supported calendar.
pub fn precompute(start: Date, days: usize) -> Result<Vec<DayState>> {
    let mut result = Vec::with_capacity(days);
    let mut date = start;
    for index in 0..days {
        if index > 0 {
            date = date
                .tomorrow()
                .context("computing simulation calendar date")?;
        }
        let is_weekend = matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday);
        let weekend = if is_weekend { 0.6 } else { 1.0 };
        let angle = f64::from(date.day_of_year() - 1) / 365.0 * std::f64::consts::TAU;
        let annual = (angle.cos() + 1.0) / 10.0 + 0.8;
        let month_offset = (i32::from(date.year()) - 2016) * 12 + i32::from(date.month());
        let growth = 1.0 + f64::from(month_offset) / 12.0 * 0.2;
        let (opens_at, closes_at) = if is_weekend { (480, 900) } else { (420, 1200) };
        let month_day = (date.month(), date.day());
        let season = if !((3, 21)..(12, 21)).contains(&month_day) {
            Season::Winter
        } else if month_day < (6, 21) {
            Season::Spring
        } else if month_day < (9, 21) {
            Season::Summer
        } else {
            Season::Fall
        };
        result.push(DayState {
            index,
            date,
            is_weekend,
            season,
            effect: annual * weekend * growth,
            opens_at,
            closes_at,
        });
    }
    Ok(result)
}

#[must_use]
pub fn penetration(days_since_open: usize) -> f64 {
    let days = u16::try_from(days_since_open).unwrap_or(365).min(365);
    let pct = f64::from(days) / 365.0;
    (1.0 + pct * (std::f64::consts::E - 1.0)).ln().min(1.0)
}

#[cfg(test)]
mod tests {
    use super::{Season, penetration, precompute};
    use jiff::civil::date;

    #[test]
    fn start_date_weekends_and_hours_follow_calendar() {
        let days = precompute(date(2023, 1, 1), 3).unwrap();
        assert_eq!(days[0].index, 0);
        assert_eq!(days[0].date, date(2023, 1, 1));
        assert!(days[0].is_weekend);
        assert_eq!((days[0].opens_at, days[0].closes_at), (480, 900));
        assert_eq!(days[1].date, date(2023, 1, 2));
        assert!(!days[1].is_weekend);
        assert_eq!((days[1].opens_at, days[1].closes_at), (420, 1200));
        assert_eq!(days[2].index, 2);
    }

    #[test]
    fn effects_multiply_annual_weekend_and_fractional_year_growth() {
        let days = precompute(date(2023, 1, 1), 183).unwrap();
        assert!((days[0].effect - 0.6 * (1.0 + 85.0 / 12.0 * 0.2)).abs() < 1e-12);
        let angle = 182.0 / 365.0 * std::f64::consts::TAU;
        let annual = (angle.cos() + 1.0) / 10.0 + 0.8;
        assert!((days[182].effect - annual * 0.6 * (1.0 + 91.0 / 12.0 * 0.2)).abs() < 1e-12);
    }

    #[test]
    fn seasons_change_on_the_twenty_first_including_leap_years() {
        for year in [2023, 2024] {
            for (month, before, after) in [
                (3, Season::Winter, Season::Spring),
                (6, Season::Spring, Season::Summer),
                (9, Season::Summer, Season::Fall),
                (12, Season::Fall, Season::Winter),
            ] {
                let days = precompute(date(year, month, 20), 2).unwrap();
                assert_eq!(days[0].season, before);
                assert_eq!(days[1].season, after);
            }
        }
        let days = precompute(date(2024, 2, 28), 3).unwrap();
        assert_eq!(days[1].date, date(2024, 2, 29));
        assert_eq!(days[2].date, date(2024, 3, 1));
    }

    #[test]
    fn penetration_rises_to_one_over_the_first_year_and_stays_there() {
        for (day, expected) in [
            (0, 0.0),
            (30, 0.1321),
            (180, 0.6132),
            (365, 1.0),
            (900, 1.0),
        ] {
            assert!((penetration(day) - expected).abs() < 0.001);
        }
    }

    #[test]
    fn calendar_reports_overflow_and_accepts_an_empty_range() {
        assert!(precompute(date(2023, 1, 1), 0).unwrap().is_empty());
        assert_eq!(precompute(date(9999, 12, 31), 1).unwrap().len(), 1);
        assert!(precompute(date(9999, 12, 31), 2).is_err());
    }
}
