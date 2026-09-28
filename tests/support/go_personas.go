package main

import (
	"encoding/csv"
	"math/rand/v2"
	"os"

	"rowing-machine/internal/market"
	"rowing-machine/internal/models"
)

func main() {
	writer := csv.NewWriter(os.Stdout)
	if err := writer.Write([]string{"customer", "persona"}); err != nil {
		panic(err)
	}
	storeRNG := rand.New(rand.NewPCG(42, 0))
	for i, store := range models.StoreConfigs() {
		store.ID = models.UUIDFromRNG(storeRNG)
		m := market.NewMarket(store, rand.New(rand.NewPCG(42+uint64(i)+1, 0)), 10)
		m.ActivateCustomers(store.OpenedDay + 365)
		for _, customer := range m.ActiveCustomers {
			if err := writer.Write([]string{models.FormatUUID(customer.ID), customer.Persona.Name()}); err != nil {
				panic(err)
			}
		}
	}
	writer.Flush()
	if err := writer.Error(); err != nil {
		panic(err)
	}
}
