package simulation

import (
	"fmt"
	"math/rand/v2"
	"strconv"
	"time"

	"rowing-machine/internal/catalog"
	"rowing-machine/internal/market"
	"rowing-machine/internal/models"
	"rowing-machine/internal/output"
)

// Config holds all CLI configuration for a simulation run.
type Config struct {
	Years     int
	Scale     int
	Seed      uint64
	StartDate time.Time
	OutputDir string
	Prefix    string
	Quiet     bool
}

// Run executes the full simulation with the given configuration.
func Run(cfg Config) error {
	numDays := cfg.Years * 365

	// Pre-compute day states
	days := PrecomputeDays(cfg.StartDate, numDays)

	// Progress tracking
	progress := NewProgress(cfg.Quiet, numDays)
	progress.PrintSeed(cfg.Seed)

	// Create stores with deterministic UUIDs
	storeRNG := rand.New(rand.NewPCG(cfg.Seed, 0))
	storeConfigs := models.StoreConfigs()
	stores := make([]models.Store, len(storeConfigs))
	for i, sc := range storeConfigs {
		sc.ID = models.UUIDFromRNG(storeRNG)
		stores[i] = sc
	}

	// Create markets (one per store, each with own deterministic PRNG)
	markets := make([]*market.Market, len(stores))
	for i, store := range stores {
		marketRNG := rand.New(rand.NewPCG(cfg.Seed+uint64(i)+1, 0))
		markets[i] = market.NewMarket(store, marketRNG, cfg.Scale)
	}

	// Initialize output
	writer, err := output.NewCSVWriter(cfg.OutputDir, cfg.Prefix)
	if err != nil {
		return fmt.Errorf("creating output: %w", err)
	}
	defer writer.Close()

	// Main simulation loop
	seenCustomers := make(map[[16]byte]bool)
	var allCustomers []models.Customer
	var allOrderRows, allItemRows, allTweetRows [][]string

	for _, day := range days {
		dayInfo := market.DayInfo{
			Index:     day.Index,
			Date:      day.Date,
			IsWeekend: day.IsWeekend,
			Season:    day.Season,
			Effect:    day.Effect,
			OpensAt:   day.OpensAt,
			ClosesAt:  day.ClosesAt,
		}
		for _, m := range markets {
			result := m.SimDay(dayInfo, seenCustomers)

			for _, c := range result.NewCustomers {
				allCustomers = append(allCustomers, c)
			}

			for _, owi := range result.Orders {
				allOrderRows = append(allOrderRows, orderToRow(owi.Order))
				for j, item := range owi.Order.Items {
					allItemRows = append(allItemRows, itemToRow(owi.ItemIDs[j], owi.Order.ID, item))
				}
			}

			for _, tweet := range result.Tweets {
				allTweetRows = append(allTweetRows, tweetToRow(tweet))
			}
		}
		progress.Update(day.Index)
	}

	// Write all output

	// Stores
	storeRows := make([][]string, len(stores))
	for i, s := range stores {
		storeRows[i] = storeToRow(s, cfg.StartDate)
	}
	if err := writer.WriteStores(storeRows); err != nil {
		return fmt.Errorf("writing stores: %w", err)
	}

	// Customers (only those who placed orders)
	customerRows := make([][]string, len(allCustomers))
	for i, c := range allCustomers {
		customerRows[i] = customerToRow(c)
	}
	if err := writer.WriteCustomers(customerRows); err != nil {
		return fmt.Errorf("writing customers: %w", err)
	}

	// Orders
	if err := writer.WriteOrders(allOrderRows); err != nil {
		return fmt.Errorf("writing orders: %w", err)
	}

	// Items
	if err := writer.WriteItems(allItemRows); err != nil {
		return fmt.Errorf("writing items: %w", err)
	}

	// Products (static catalog)
	productRows := make([][]string, len(catalog.MenuItems))
	for i, p := range catalog.MenuItems {
		productRows[i] = productToRow(p)
	}
	if err := writer.WriteProducts(productRows); err != nil {
		return fmt.Errorf("writing products: %w", err)
	}

	// Supplies (denormalized)
	supplyRows := catalog.DenormalizedSupplyRows()
	sRows := make([][]string, len(supplyRows))
	for i, sr := range supplyRows {
		sRows[i] = supplyToRow(sr)
	}
	if err := writer.WriteSupplies(sRows); err != nil {
		return fmt.Errorf("writing supplies: %w", err)
	}

	// Tweets
	if err := writer.WriteTweets(allTweetRows); err != nil {
		return fmt.Errorf("writing tweets: %w", err)
	}

	// Summary
	progress.PrintSummary(map[string]int{
		"stores":    len(stores),
		"customers": len(allCustomers),
		"orders":    len(allOrderRows),
		"items":     len(allItemRows),
		"products":  len(catalog.MenuItems),
		"supplies":  len(supplyRows),
		"tweets":    len(allTweetRows),
	}, cfg.OutputDir)

	return nil
}

// --- Row conversion helpers ---

const timeFormat = "2006-01-02T15:04:05"

func storeToRow(s models.Store, epoch time.Time) []string {
	openedAt := epoch.AddDate(0, 0, s.OpenedDay)
	return []string{
		models.FormatUUID(s.ID),
		s.Name,
		openedAt.Format(timeFormat),
		strconv.FormatFloat(s.TaxRate, 'f', -1, 64),
	}
}

func customerToRow(c models.Customer) []string {
	return []string{
		models.FormatUUID(c.ID),
		c.Name,
	}
}

func orderToRow(o models.Order) []string {
	return []string{
		models.FormatUUID(o.ID),
		models.FormatUUID(o.CustomerID),
		o.OrderedAt.Format(timeFormat),
		models.FormatUUID(o.StoreID),
		strconv.FormatInt(o.Subtotal, 10),
		strconv.FormatInt(o.TaxPaid, 10),
		strconv.FormatInt(o.OrderTotal, 10),
	}
}

func itemToRow(itemID [16]byte, orderID [16]byte, item models.Item) []string {
	return []string{
		models.FormatUUID(itemID),
		models.FormatUUID(orderID),
		item.SKU,
	}
}

func productToRow(p models.Item) []string {
	return []string{
		p.SKU,
		p.Name,
		p.Type.String(),
		strconv.FormatInt(p.Price, 10),
		p.Description,
	}
}

func supplyToRow(sr catalog.SupplyRow) []string {
	perishable := "False"
	if sr.Perishable {
		perishable = "True"
	}
	return []string{
		sr.ID,
		sr.Name,
		strconv.FormatInt(sr.Cost, 10),
		perishable,
		sr.SKU,
	}
}

func tweetToRow(t models.Tweet) []string {
	return []string{
		models.FormatUUID(t.ID),
		models.FormatUUID(t.UserID),
		t.TweetedAt.Format(timeFormat),
		t.Content,
	}
}
