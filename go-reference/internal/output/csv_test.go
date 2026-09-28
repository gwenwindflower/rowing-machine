package output

import (
	"encoding/csv"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestNewCSVWriterCreatesDirectory(t *testing.T) {
	dir := filepath.Join(t.TempDir(), "nested", "output")
	_, err := NewCSVWriter(dir, "raw")
	if err != nil {
		t.Fatalf("NewCSVWriter: %v", err)
	}
	info, err := os.Stat(dir)
	if err != nil {
		t.Fatalf("directory not created: %v", err)
	}
	if !info.IsDir() {
		t.Fatal("expected a directory")
	}
}

func TestWriteStores(t *testing.T) {
	dir := t.TempDir()
	w, err := NewCSVWriter(dir, "raw")
	if err != nil {
		t.Fatalf("NewCSVWriter: %v", err)
	}

	rows := [][]string{
		{"1", "Portland", "2018-09-01", "5"},
		{"2", "Denver", "2019-01-15", "7"},
	}
	if err := w.WriteStores(rows); err != nil {
		t.Fatalf("WriteStores: %v", err)
	}
	if err := w.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}

	records := readCSV(t, filepath.Join(dir, "raw_stores.csv"))
	if len(records) != 3 { // header + 2 data rows
		t.Fatalf("expected 3 records, got %d", len(records))
	}
	// Verify header
	expectedHeader := []string{"id", "name", "opened_at", "tax_rate"}
	for i, h := range expectedHeader {
		if records[0][i] != h {
			t.Errorf("header[%d]: got %q, want %q", i, records[0][i], h)
		}
	}
	// Verify first data row
	if records[1][1] != "Portland" {
		t.Errorf("first store name: got %q, want %q", records[1][1], "Portland")
	}
}

func TestFileNaming(t *testing.T) {
	dir := t.TempDir()
	w, err := NewCSVWriter(dir, "test_prefix")
	if err != nil {
		t.Fatalf("NewCSVWriter: %v", err)
	}

	if err := w.WriteProducts([][]string{{"SKU1", "Tent", "gear", "common", "5000", "A tent"}}); err != nil {
		t.Fatalf("WriteProducts: %v", err)
	}
	if err := w.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}

	expected := filepath.Join(dir, "test_prefix_products.csv")
	if _, err := os.Stat(expected); err != nil {
		t.Fatalf("expected file %s: %v", expected, err)
	}
}

func TestCloseFlushes(t *testing.T) {
	dir := t.TempDir()
	w, err := NewCSVWriter(dir, "raw")
	if err != nil {
		t.Fatalf("NewCSVWriter: %v", err)
	}

	if err := w.WriteSupplies([][]string{{"1", "Rope", "1200", "False", "Embervault Mines", "SUP01"}}); err != nil {
		t.Fatalf("WriteSupplies: %v", err)
	}
	if err := w.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}

	path := filepath.Join(dir, "raw_supplies.csv")
	info, err := os.Stat(path)
	if err != nil {
		t.Fatalf("stat: %v", err)
	}
	if info.Size() == 0 {
		t.Fatal("expected non-empty file after Close")
	}
}

func TestCSVEscaping(t *testing.T) {
	dir := t.TempDir()
	w, err := NewCSVWriter(dir, "raw")
	if err != nil {
		t.Fatalf("NewCSVWriter: %v", err)
	}

	// Field with a comma should be properly quoted.
	rows := [][]string{{"SKU1", "Tent, Large", "gear", "common", "5000", "A large, roomy tent"}}
	if err := w.WriteProducts(rows); err != nil {
		t.Fatalf("WriteProducts: %v", err)
	}
	if err := w.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}

	records := readCSV(t, filepath.Join(dir, "raw_products.csv"))
	if len(records) != 2 {
		t.Fatalf("expected 2 records, got %d", len(records))
	}
	if records[1][1] != "Tent, Large" {
		t.Errorf("escaped field: got %q, want %q", records[1][1], "Tent, Large")
	}
}

func TestEmptyWrite(t *testing.T) {
	dir := t.TempDir()
	w, err := NewCSVWriter(dir, "raw")
	if err != nil {
		t.Fatalf("NewCSVWriter: %v", err)
	}

	if err := w.WriteSparrows([][]string{}); err != nil {
		t.Fatalf("WriteSparrows: %v", err)
	}
	if err := w.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}

	records := readCSV(t, filepath.Join(dir, "raw_sparrows.csv"))
	if len(records) != 1 { // header only
		t.Fatalf("expected 1 record (header only), got %d", len(records))
	}
	expectedHeader := []string{"id", "user_id", "sent_at", "content"}
	for i, h := range expectedHeader {
		if records[0][i] != h {
			t.Errorf("header[%d]: got %q, want %q", i, records[0][i], h)
		}
	}
}

func TestMultipleCalls(t *testing.T) {
	dir := t.TempDir()
	w, err := NewCSVWriter(dir, "raw")
	if err != nil {
		t.Fatalf("NewCSVWriter: %v", err)
	}

	batch1 := [][]string{
		{"o1", "c1", "2024-01-01T00:00:00Z", "s1", "1000", "50", "1050"},
	}
	batch2 := [][]string{
		{"o2", "c2", "2024-01-02T00:00:00Z", "s2", "2000", "100", "2100"},
		{"o3", "c3", "2024-01-03T00:00:00Z", "s3", "3000", "150", "3150"},
	}

	if err := w.WriteOrders(batch1); err != nil {
		t.Fatalf("WriteOrders batch1: %v", err)
	}
	if err := w.WriteOrders(batch2); err != nil {
		t.Fatalf("WriteOrders batch2: %v", err)
	}
	if err := w.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}

	records := readCSV(t, filepath.Join(dir, "raw_orders.csv"))
	// 1 header + 3 data rows
	if len(records) != 4 {
		t.Fatalf("expected 4 records, got %d", len(records))
	}
	// Verify header appears only once (first record).
	if records[0][0] != "id" {
		t.Errorf("first record should be header, got %q", records[0][0])
	}
	if records[1][0] != "o1" {
		t.Errorf("second record: got %q, want %q", records[1][0], "o1")
	}
}

func TestWriteRejectsRaggedRows(t *testing.T) {
	w, err := NewCSVWriter(t.TempDir(), "raw")
	if err != nil {
		t.Fatalf("NewCSVWriter: %v", err)
	}

	err = w.WriteOrders([][]string{{"id", "customer"}})
	if err == nil {
		t.Fatal("WriteOrders accepted a row with too few fields")
	}
	if !strings.Contains(err.Error(), "orders row has 2 fields; want 7") {
		t.Fatalf("WriteOrders error = %q", err)
	}
}

// readCSV is a test helper that reads all records from a CSV file.
func readCSV(t *testing.T, path string) [][]string {
	t.Helper()
	f, err := os.Open(path)
	if err != nil {
		t.Fatalf("open %s: %v", path, err)
	}
	defer f.Close()
	records, err := csv.NewReader(f).ReadAll()
	if err != nil {
		t.Fatalf("read csv %s: %v", path, err)
	}
	return records
}
