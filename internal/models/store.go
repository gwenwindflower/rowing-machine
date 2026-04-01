package models

// Store represents a physical store location.
type Store struct {
	ID             [16]byte // UUID
	Name           string
	BasePopularity float64 // [0.85, 0.95]
	OpenedDay      int     // day index when store opened (0-based from epoch)
	TaxRate        float64 // e.g. 0.06 for 6%
	TAMBase        int     // base Total Addressable Market before scale multiplier
}

// PBuy returns the seasonal purchase probability for this store.
// dayEffect is the pre-computed product of annual * weekend * growth curves.
func (s *Store) PBuy(dayEffect float64) float64 {
	return s.BasePopularity * dayEffect
}

// IsOpen returns true if the store has opened by the given day index.
func (s *Store) IsOpen(dayIndex int) bool {
	return dayIndex >= s.OpenedDay
}

// DaysSinceOpen returns the number of days since the store opened.
// Can be negative if store hasn't opened yet.
func (s *Store) DaysSinceOpen(dayIndex int) int {
	return dayIndex - s.OpenedDay
}

// IsOpenAt returns true if the store is open at the given minute of the day.
// opensAt and closesAt are in minutes from midnight (e.g. 420 = 7:00 AM).
func (s *Store) IsOpenAt(minute, opensAt, closesAt int) bool {
	return minute >= opensAt && minute < closesAt
}

// StoreConfigs returns the 6 hardcoded store configurations.
// IDs are not set — they must be assigned using UUIDFromRNG by the caller.
func StoreConfigs() []Store {
	return []Store{
		{Name: "Philadelphia", BasePopularity: 0.85, OpenedDay: 0, TAMBase: 9, TaxRate: 0.06},
		{Name: "Brooklyn", BasePopularity: 0.95, OpenedDay: 192, TAMBase: 14, TaxRate: 0.04},
		{Name: "Chicago", BasePopularity: 0.92, OpenedDay: 605, TAMBase: 12, TaxRate: 0.0625},
		{Name: "San Francisco", BasePopularity: 0.87, OpenedDay: 615, TAMBase: 11, TaxRate: 0.075},
		{Name: "New Orleans", BasePopularity: 0.92, OpenedDay: 920, TAMBase: 8, TaxRate: 0.04},
		{Name: "Los Angeles", BasePopularity: 0.87, OpenedDay: 1107, TAMBase: 8, TaxRate: 0.08},
	}
}
