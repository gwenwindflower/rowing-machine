package models

// Season represents a calendar season for simulation purposes.
type Season int

const (
	Winter Season = iota
	Spring
	Summer
	Fall
)

func (s Season) String() string {
	switch s {
	case Winter:
		return "winter"
	case Spring:
		return "spring"
	case Summer:
		return "summer"
	case Fall:
		return "fall"
	default:
		return "unknown"
	}
}
