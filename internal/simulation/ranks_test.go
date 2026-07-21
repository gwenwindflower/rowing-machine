package simulation

import (
	"testing"

	"rowing-machine/internal/models"
)

func TestAssignGuildRanksBalancesOrderCountCohorts(t *testing.T) {
	customers := make([]models.Customer, 10)
	orderCounts := make(map[[16]byte]int, len(customers))
	for i := range customers {
		customers[i].ID = [16]byte{byte(i + 1)}
		orderCounts[customers[i].ID] = i + 1
	}

	assignGuildRanks(customers, orderCounts)

	rankCounts := make([]int, 4)
	for _, customer := range customers {
		rankCounts[int(customer.GuildRank)]++
	}
	minCount, maxCount := rankCounts[0], rankCounts[0]
	for _, count := range rankCounts[1:] {
		if count < minCount {
			minCount = count
		}
		if count > maxCount {
			maxCount = count
		}
	}
	if maxCount-minCount > 1 {
		t.Errorf("guild rank cohort sizes = %v, want difference at most 1", rankCounts)
	}

	for _, lower := range customers {
		for _, higher := range customers {
			if orderCounts[lower.ID] < orderCounts[higher.ID] && lower.GuildRank > higher.GuildRank {
				t.Errorf("customer with %d orders has rank %s above customer with %d orders at rank %s", orderCounts[lower.ID], lower.GuildRank, orderCounts[higher.ID], higher.GuildRank)
			}
		}
	}
}

func TestAssignGuildRanksBreaksOrderCountTiesByUUID(t *testing.T) {
	ascending := make([]models.Customer, 8)
	descending := make([]models.Customer, 8)
	orderCounts := make(map[[16]byte]int, len(ascending))
	for i := range ascending {
		id := [16]byte{byte(i + 1)}
		ascending[i].ID = id
		descending[len(descending)-1-i].ID = id
		orderCounts[id] = 10
	}

	assignGuildRanks(ascending, orderCounts)
	assignGuildRanks(descending, orderCounts)

	ranksByID := make(map[[16]byte]models.GuildRank, len(ascending))
	for _, customer := range ascending {
		ranksByID[customer.ID] = customer.GuildRank
	}
	for _, customer := range descending {
		if customer.GuildRank != ranksByID[customer.ID] {
			t.Errorf("customer %x rank = %s, want %s", customer.ID, customer.GuildRank, ranksByID[customer.ID])
		}
	}
}
