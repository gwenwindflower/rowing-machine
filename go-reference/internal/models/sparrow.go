package models

import (
	"math/rand/v2"
	"strings"
	"time"
)

// Sparrow represents a magical message bird post about an order.
type Sparrow struct {
	ID     [16]byte
	UserID [16]byte // customer ID
	SentAt time.Time
	Content string
}

// Adjective pools for sparrow content generation.
var positiveAdj = []string{"the finest", "truly enchanted", "magnificent", "extraordinary", "masterwork", "legendary quality", "my prized possession"}
var negativeAdj = []string{"cursed", "the worst enchantment", "a total misfire", "completely mundane", "defective", "barely magical", "an utter waste of gold"}
var neutralAdj = []string{"serviceable", "adequate", "fair enough", "unremarkable", "decent craftsmanship", "passable", "nothing special", "just ordinary"}

// NewSparrow creates a sparrow for an order.
// Delay is 0-19 minutes randomly added to the order time.
// Content is generated based on fanLevel and the items ordered.
func NewSparrow(rng *rand.Rand, customerID [16]byte, fanLevel int, items []Item, orderedAt time.Time) Sparrow {
	delay := rng.IntN(20) // 0-19 minutes
	sentAt := orderedAt.Add(time.Duration(delay) * time.Minute)

	content := buildContent(rng, fanLevel, items)

	return Sparrow{
		ID:      UUIDFromRNG(rng),
		UserID:  customerID,
		SentAt:  sentAt,
		Content: content,
	}
}

// buildItemsSentence constructs a natural-language sentence listing acquired items.
func buildItemsSentence(items []Item) string {
	if len(items) == 0 {
		return ""
	}
	if len(items) == 1 {
		return "Acquired a " + items[0].Name
	}
	if len(items) == 2 {
		return "Acquired a " + items[0].Name + " and a " + items[1].Name
	}
	// 3+ items: "Acquired a X, a Y, ..., and a Z"
	var b strings.Builder
	b.WriteString("Acquired a ")
	for i, item := range items {
		if i == len(items)-1 {
			b.WriteString("and a ")
			b.WriteString(item.Name)
		} else if i == len(items)-2 {
			b.WriteString(item.Name)
			b.WriteString(", ")
		} else {
			b.WriteString(item.Name)
			b.WriteString(", a ")
		}
	}
	return b.String()
}

// buildContent generates sparrow text based on fan level and items.
func buildContent(rng *rand.Rand, fanLevel int, items []Item) string {
	itemsSentence := buildItemsSentence(items)

	switch {
	case fanLevel > 3:
		adj := positiveAdj[rng.IntN(len(positiveAdj))]
		return "Wares from the Arcanum Collective are " + adj + "! " + itemsSentence + "."
	case fanLevel < 3:
		adj := negativeAdj[rng.IntN(len(negativeAdj))]
		return "Arcanum Collective again. " + itemsSentence + ". Their craft is " + adj + "."
	default:
		adj := neutralAdj[rng.IntN(len(neutralAdj))]
		return "The Arcanum Collective is " + adj + ". " + itemsSentence + "."
	}
}
