package models

import (
	"math/rand/v2"
	"strings"
	"time"
)

// Tweet represents a customer's social media post about an order.
type Tweet struct {
	ID        [16]byte
	UserID    [16]byte // customer ID
	TweetedAt time.Time
	Content   string
}

// Adjective pools for tweet content generation.
var positiveAdj = []string{"the best", "awesome", "delicious", "amazing", "fantastic", "sooo gooood", "my favorite"}
var negativeAdj = []string{"terrible", "the worst", "awful", "disgusting", "gross", "inedible", "my least favorite"}
var neutralAdj = []string{"okay", "fine", "alright", "average", "pretty decent", "solid", "not bad", "just meh"}

// NewTweet creates a tweet for an order.
// Delay is 0-19 minutes randomly added to the order time.
// Content is generated based on fanLevel and the items ordered.
func NewTweet(rng *rand.Rand, customerID [16]byte, fanLevel int, items []Item, orderedAt time.Time) Tweet {
	delay := rng.IntN(20) // 0-19 minutes
	tweetedAt := orderedAt.Add(time.Duration(delay) * time.Minute)

	content := buildContent(rng, fanLevel, items)

	return Tweet{
		ID:        UUIDFromRNG(rng),
		UserID:    customerID,
		TweetedAt: tweetedAt,
		Content:   content,
	}
}

// buildItemsSentence constructs a natural-language sentence listing ordered items.
func buildItemsSentence(items []Item) string {
	if len(items) == 0 {
		return ""
	}
	if len(items) == 1 {
		return "Ordered a " + items[0].Name
	}
	if len(items) == 2 {
		return "Ordered a " + items[0].Name + " and a " + items[1].Name
	}
	// 3+ items: "Ordered a X, a Y, ..., and a Z"
	var b strings.Builder
	b.WriteString("Ordered a ")
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

// buildContent generates tweet text based on fan level and items.
func buildContent(rng *rand.Rand, fanLevel int, items []Item) string {
	itemsSentence := buildItemsSentence(items)

	switch {
	case fanLevel > 3:
		adj := positiveAdj[rng.IntN(len(positiveAdj))]
		return "Jaffles from the Jaffle Shop are " + adj + "! " + itemsSentence + "."
	case fanLevel < 3:
		adj := negativeAdj[rng.IntN(len(negativeAdj))]
		return "Jaffle Shop again. " + itemsSentence + ". This place is " + adj + "."
	default:
		adj := neutralAdj[rng.IntN(len(neutralAdj))]
		return "Jaffle shop is " + adj + ". " + itemsSentence + "."
	}
}
