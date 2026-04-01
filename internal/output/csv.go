package output

import (
	"bufio"
	"encoding/csv"
	"fmt"
	"os"
	"path/filepath"
)

// entityHeaders maps entity names to their CSV column headers.
var entityHeaders = map[string][]string{
	"stores":    {"id", "name", "opened_at", "tax_rate"},
	"customers": {"id", "name"},
	"orders":    {"id", "customer", "ordered_at", "store_id", "subtotal", "tax_paid", "order_total"},
	"items":     {"id", "order_id", "sku"},
	"products":  {"sku", "name", "type", "price", "description"},
	"supplies":  {"id", "name", "cost", "perishable", "sku"},
	"tweets":    {"id", "user_id", "tweeted_at", "content"},
}

// CSVWriter implements OutputWriter by writing to CSV files with buffered I/O.
type CSVWriter struct {
	dir     string
	prefix  string
	files   map[string]*os.File
	buffers map[string]*bufio.Writer
	writers map[string]*csv.Writer
}

// NewCSVWriter creates a new CSV writer that writes to the given directory.
// Creates the directory if it doesn't exist.
// Files are opened lazily on first write.
func NewCSVWriter(outputDir, prefix string) (*CSVWriter, error) {
	if err := os.MkdirAll(outputDir, 0o755); err != nil {
		return nil, fmt.Errorf("create output dir: %w", err)
	}
	return &CSVWriter{
		dir:     outputDir,
		prefix:  prefix,
		files:   make(map[string]*os.File),
		buffers: make(map[string]*bufio.Writer),
		writers: make(map[string]*csv.Writer),
	}, nil
}

// getWriter returns the csv.Writer for the given entity, creating the file
// and writing the header row on first access.
func (w *CSVWriter) getWriter(entity string) (*csv.Writer, error) {
	if cw, ok := w.writers[entity]; ok {
		return cw, nil
	}

	headers, ok := entityHeaders[entity]
	if !ok {
		return nil, fmt.Errorf("unknown entity: %s", entity)
	}

	filename := fmt.Sprintf("%s_%s.csv", w.prefix, entity)
	path := filepath.Join(w.dir, filename)

	f, err := os.Create(path)
	if err != nil {
		return nil, fmt.Errorf("create %s: %w", path, err)
	}

	buf := bufio.NewWriter(f)
	cw := csv.NewWriter(buf)

	if err := cw.Write(headers); err != nil {
		f.Close()
		return nil, fmt.Errorf("write header for %s: %w", entity, err)
	}

	w.files[entity] = f
	w.buffers[entity] = buf
	w.writers[entity] = cw

	return cw, nil
}

// writeRows writes rows for the given entity.
func (w *CSVWriter) writeRows(entity string, rows [][]string) error {
	cw, err := w.getWriter(entity)
	if err != nil {
		return err
	}
	for _, row := range rows {
		if err := cw.Write(row); err != nil {
			return fmt.Errorf("write %s row: %w", entity, err)
		}
	}
	return nil
}

func (w *CSVWriter) WriteStores(rows [][]string) error    { return w.writeRows("stores", rows) }
func (w *CSVWriter) WriteCustomers(rows [][]string) error { return w.writeRows("customers", rows) }
func (w *CSVWriter) WriteOrders(rows [][]string) error    { return w.writeRows("orders", rows) }
func (w *CSVWriter) WriteItems(rows [][]string) error     { return w.writeRows("items", rows) }
func (w *CSVWriter) WriteProducts(rows [][]string) error  { return w.writeRows("products", rows) }
func (w *CSVWriter) WriteSupplies(rows [][]string) error  { return w.writeRows("supplies", rows) }
func (w *CSVWriter) WriteTweets(rows [][]string) error    { return w.writeRows("tweets", rows) }

// Close flushes all buffered writers and closes all files.
// Returns the first error encountered.
func (w *CSVWriter) Close() error {
	var firstErr error
	for entity, cw := range w.writers {
		cw.Flush()
		if err := cw.Error(); err != nil && firstErr == nil {
			firstErr = fmt.Errorf("flush csv %s: %w", entity, err)
		}
	}
	for entity, buf := range w.buffers {
		if err := buf.Flush(); err != nil && firstErr == nil {
			firstErr = fmt.Errorf("flush buffer %s: %w", entity, err)
		}
	}
	for entity, f := range w.files {
		if err := f.Close(); err != nil && firstErr == nil {
			firstErr = fmt.Errorf("close %s: %w", entity, err)
		}
	}
	return firstErr
}
