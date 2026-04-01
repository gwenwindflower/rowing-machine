package simulation

import (
	"encoding/csv"
	"os"
	"path/filepath"
	"strconv"
	"testing"
	"time"
)

// testConfig returns a default Config suitable for integration tests.
func testConfig(t *testing.T) Config {
	t.Helper()
	return Config{
		Years:     1,
		Scale:     10,
		Seed:      42,
		StartDate: time.Date(2018, 9, 1, 0, 0, 0, 0, time.UTC),
		OutputDir: t.TempDir(),
		Prefix:    "raw",
		Quiet:     true,
	}
}

// --- Helper functions ---

// readCSV reads a CSV file and returns headers and data rows separately.
func readCSV(t *testing.T, path string) (headers []string, rows [][]string) {
	t.Helper()
	f, err := os.Open(path)
	if err != nil {
		t.Fatalf("open %s: %v", path, err)
	}
	defer f.Close()

	r := csv.NewReader(f)
	all, err := r.ReadAll()
	if err != nil {
		t.Fatalf("read csv %s: %v", path, err)
	}
	if len(all) == 0 {
		t.Fatalf("csv %s is empty", path)
	}
	return all[0], all[1:]
}

// csvPath builds the expected output file path for an entity.
func csvPath(dir, prefix, entity string) string {
	return filepath.Join(dir, prefix+"_"+entity+".csv")
}

// columnSet extracts a set of unique values from a specific column index.
func columnSet(rows [][]string, col int) map[string]bool {
	s := make(map[string]bool, len(rows))
	for _, row := range rows {
		s[row[col]] = true
	}
	return s
}

// colIndex returns the index of a column name in the headers slice.
func colIndex(t *testing.T, headers []string, name string) int {
	t.Helper()
	for i, h := range headers {
		if h == name {
			return i
		}
	}
	t.Fatalf("column %q not found in headers %v", name, headers)
	return -1
}

// parseTimestamp parses an ISO 8601 timestamp (no timezone) as used in the output.
func parseTimestamp(t *testing.T, s string) time.Time {
	t.Helper()
	ts, err := time.Parse("2006-01-02T15:04:05", s)
	if err != nil {
		t.Fatalf("parse timestamp %q: %v", s, err)
	}
	return ts
}

// --- Integration tests ---

func TestIntegration_EndToEnd(t *testing.T) {
	cfg := testConfig(t)
	if err := Run(cfg); err != nil {
		t.Fatalf("Run failed: %v", err)
	}

	entities := []string{"stores", "customers", "orders", "items", "products", "supplies", "tweets"}
	for _, entity := range entities {
		path := csvPath(cfg.OutputDir, cfg.Prefix, entity)
		_, rows := readCSV(t, path)
		if len(rows) == 0 {
			t.Errorf("%s.csv has no data rows (header only)", entity)
		}
	}
}

func TestIntegration_Determinism(t *testing.T) {
	cfg1 := testConfig(t)
	cfg2 := testConfig(t)
	// cfg2 gets a different TempDir automatically from testConfig

	if err := Run(cfg1); err != nil {
		t.Fatalf("Run 1 failed: %v", err)
	}
	if err := Run(cfg2); err != nil {
		t.Fatalf("Run 2 failed: %v", err)
	}

	entities := []string{"stores", "customers", "orders", "items", "products", "supplies", "tweets"}
	for _, entity := range entities {
		path1 := csvPath(cfg1.OutputDir, cfg1.Prefix, entity)
		path2 := csvPath(cfg2.OutputDir, cfg2.Prefix, entity)

		data1, err := os.ReadFile(path1)
		if err != nil {
			t.Fatalf("read %s: %v", path1, err)
		}
		data2, err := os.ReadFile(path2)
		if err != nil {
			t.Fatalf("read %s: %v", path2, err)
		}

		if string(data1) != string(data2) {
			t.Errorf("%s.csv: outputs differ between runs with same seed", entity)
		}
	}
}

func TestIntegration_ReferentialIntegrity(t *testing.T) {
	cfg := testConfig(t)
	if err := Run(cfg); err != nil {
		t.Fatalf("Run failed: %v", err)
	}

	// Build reference sets
	custH, custRows := readCSV(t, csvPath(cfg.OutputDir, cfg.Prefix, "customers"))
	custIDs := columnSet(custRows, colIndex(t, custH, "id"))

	storeH, storeRows := readCSV(t, csvPath(cfg.OutputDir, cfg.Prefix, "stores"))
	storeIDs := columnSet(storeRows, colIndex(t, storeH, "id"))

	prodH, prodRows := readCSV(t, csvPath(cfg.OutputDir, cfg.Prefix, "products"))
	skus := columnSet(prodRows, colIndex(t, prodH, "sku"))

	orderH, orderRows := readCSV(t, csvPath(cfg.OutputDir, cfg.Prefix, "orders"))
	orderIDs := columnSet(orderRows, colIndex(t, orderH, "id"))
	orderCustCol := colIndex(t, orderH, "customer")
	orderStoreCol := colIndex(t, orderH, "store_id")

	// Check orders -> customers, stores
	for i, row := range orderRows {
		if !custIDs[row[orderCustCol]] {
			t.Errorf("order row %d: customer %q not in customers", i, row[orderCustCol])
		}
		if !storeIDs[row[orderStoreCol]] {
			t.Errorf("order row %d: store_id %q not in stores", i, row[orderStoreCol])
		}
	}

	// Check items -> orders, products
	itemH, itemRows := readCSV(t, csvPath(cfg.OutputDir, cfg.Prefix, "items"))
	itemOrderCol := colIndex(t, itemH, "order_id")
	itemSKUCol := colIndex(t, itemH, "sku")

	for i, row := range itemRows {
		if !orderIDs[row[itemOrderCol]] {
			t.Errorf("item row %d: order_id %q not in orders", i, row[itemOrderCol])
		}
		if !skus[row[itemSKUCol]] {
			t.Errorf("item row %d: sku %q not in products", i, row[itemSKUCol])
		}
	}

	// Check tweets -> customers
	tweetH, tweetRows := readCSV(t, csvPath(cfg.OutputDir, cfg.Prefix, "tweets"))
	tweetUserCol := colIndex(t, tweetH, "user_id")

	for i, row := range tweetRows {
		if !custIDs[row[tweetUserCol]] {
			t.Errorf("tweet row %d: user_id %q not in customers", i, row[tweetUserCol])
		}
	}
}

func TestIntegration_OrderArithmetic(t *testing.T) {
	cfg := testConfig(t)
	if err := Run(cfg); err != nil {
		t.Fatalf("Run failed: %v", err)
	}

	headers, rows := readCSV(t, csvPath(cfg.OutputDir, cfg.Prefix, "orders"))
	subtotalCol := colIndex(t, headers, "subtotal")
	taxCol := colIndex(t, headers, "tax_paid")
	totalCol := colIndex(t, headers, "order_total")

	for i, row := range rows {
		subtotal, err := strconv.ParseInt(row[subtotalCol], 10, 64)
		if err != nil {
			t.Fatalf("order row %d: parse subtotal %q: %v", i, row[subtotalCol], err)
		}
		tax, err := strconv.ParseInt(row[taxCol], 10, 64)
		if err != nil {
			t.Fatalf("order row %d: parse tax_paid %q: %v", i, row[taxCol], err)
		}
		total, err := strconv.ParseInt(row[totalCol], 10, 64)
		if err != nil {
			t.Fatalf("order row %d: parse order_total %q: %v", i, row[totalCol], err)
		}

		if total != subtotal+tax {
			t.Errorf("order row %d: order_total(%d) != subtotal(%d) + tax_paid(%d)",
				i, total, subtotal, tax)
		}
	}
}

func TestIntegration_NoOrdersBeforeStoreOpens(t *testing.T) {
	cfg := testConfig(t)
	if err := Run(cfg); err != nil {
		t.Fatalf("Run failed: %v", err)
	}

	// Parse stores: map store ID -> opened_at date
	storeH, storeRows := readCSV(t, csvPath(cfg.OutputDir, cfg.Prefix, "stores"))
	storeIDCol := colIndex(t, storeH, "id")
	openedAtCol := colIndex(t, storeH, "opened_at")

	storeOpenDates := make(map[string]time.Time, len(storeRows))
	for _, row := range storeRows {
		storeOpenDates[row[storeIDCol]] = parseTimestamp(t, row[openedAtCol])
	}

	// Check each order
	orderH, orderRows := readCSV(t, csvPath(cfg.OutputDir, cfg.Prefix, "orders"))
	orderedAtCol := colIndex(t, orderH, "ordered_at")
	orderStoreCol := colIndex(t, orderH, "store_id")

	for i, row := range orderRows {
		orderedAt := parseTimestamp(t, row[orderedAtCol])
		storeID := row[orderStoreCol]
		openedAt, ok := storeOpenDates[storeID]
		if !ok {
			t.Errorf("order row %d: store_id %q not found in stores", i, storeID)
			continue
		}
		// Compare dates only (truncate to midnight)
		orderDate := time.Date(orderedAt.Year(), orderedAt.Month(), orderedAt.Day(), 0, 0, 0, 0, time.UTC)
		openDate := time.Date(openedAt.Year(), openedAt.Month(), openedAt.Day(), 0, 0, 0, 0, time.UTC)
		if orderDate.Before(openDate) {
			t.Errorf("order row %d: ordered_at %s is before store %q opened_at %s",
				i, orderedAt.Format("2006-01-02"), storeID, openedAt.Format("2006-01-02"))
		}
	}
}

func TestIntegration_OperatingHours(t *testing.T) {
	cfg := testConfig(t)
	if err := Run(cfg); err != nil {
		t.Fatalf("Run failed: %v", err)
	}

	headers, rows := readCSV(t, csvPath(cfg.OutputDir, cfg.Prefix, "orders"))
	orderedAtCol := colIndex(t, headers, "ordered_at")

	for i, row := range rows {
		orderedAt := parseTimestamp(t, row[orderedAtCol])
		wd := orderedAt.Weekday()
		isWeekend := wd == time.Saturday || wd == time.Sunday
		hour := orderedAt.Hour()
		minute := orderedAt.Minute()
		totalMinutes := hour*60 + minute

		if isWeekend {
			// Weekend: 08:00 (480) to 15:00 (900)
			if totalMinutes < 480 || totalMinutes >= 900 {
				t.Errorf("order row %d: weekend order at %s (minute %d) outside 08:00-15:00",
					i, orderedAt.Format("15:04"), totalMinutes)
			}
		} else {
			// Weekday: 07:00 (420) to 20:00 (1200)
			if totalMinutes < 420 || totalMinutes >= 1200 {
				t.Errorf("order row %d: weekday order at %s (minute %d) outside 07:00-20:00",
					i, orderedAt.Format("15:04"), totalMinutes)
			}
		}
	}
}

func TestIntegration_RowCountSmoke(t *testing.T) {
	cfg := testConfig(t)
	if err := Run(cfg); err != nil {
		t.Fatalf("Run failed: %v", err)
	}

	type countCheck struct {
		entity  string
		min     int
		max     int // 0 means no upper bound
		exact   int // 0 means no exact check (use useExact flag)
		isExact bool
	}

	_, orderRows := readCSV(t, csvPath(cfg.OutputDir, cfg.Prefix, "orders"))
	orderCount := len(orderRows)

	checks := []countCheck{
		{entity: "stores", exact: 6, isExact: true},
		{entity: "products", exact: 10, isExact: true},
		{entity: "customers", min: 50},
		{entity: "orders", min: 1000},
		{entity: "items", min: orderCount + 1}, // items > orders (most orders have >= 1 item)
		{entity: "tweets", min: 100, max: orderCount},
	}

	for _, c := range checks {
		_, rows := readCSV(t, csvPath(cfg.OutputDir, cfg.Prefix, c.entity))
		count := len(rows)

		if c.isExact {
			if count != c.exact {
				t.Errorf("%s: expected exactly %d rows, got %d", c.entity, c.exact, count)
			}
			continue
		}

		if count < c.min {
			t.Errorf("%s: expected at least %d rows, got %d", c.entity, c.min, count)
		}
		if c.max > 0 && count > c.max {
			t.Errorf("%s: expected at most %d rows, got %d", c.entity, c.max, count)
		}
	}
}
