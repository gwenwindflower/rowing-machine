package simulation

import (
	"bytes"
	"sort"

	"rowing-machine/internal/models"
)

func assignGuildRanks(customers []models.Customer, orderCounts map[[16]byte]int) {
	if len(customers) == 0 {
		return
	}

	indices := make([]int, len(customers))
	for i := range indices {
		indices[i] = i
	}
	sort.Slice(indices, func(i, j int) bool {
		left := customers[indices[i]]
		right := customers[indices[j]]
		leftOrders := orderCounts[left.ID]
		rightOrders := orderCounts[right.ID]
		if leftOrders != rightOrders {
			return leftOrders < rightOrders
		}
		return bytes.Compare(left.ID[:], right.ID[:]) < 0
	})

	rankCount := int(models.Master) + 1
	for position, customerIndex := range indices {
		customers[customerIndex].GuildRank = models.GuildRank(position * rankCount / len(customers))
	}
}
