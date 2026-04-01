package output

// OutputWriter defines the interface for writing simulation output.
type OutputWriter interface {
	// WriteStores writes store rows. Each row is a string slice.
	WriteStores(rows [][]string) error
	WriteCustomers(rows [][]string) error
	WriteOrders(rows [][]string) error
	WriteItems(rows [][]string) error
	WriteProducts(rows [][]string) error
	WriteSupplies(rows [][]string) error
	WriteSparrows(rows [][]string) error
	Close() error
}
