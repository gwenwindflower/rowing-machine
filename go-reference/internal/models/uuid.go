package models

import "math/rand/v2"

// UUIDFromRNG generates a v4 UUID using the provided PRNG for determinism.
func UUIDFromRNG(rng *rand.Rand) [16]byte {
	var id [16]byte
	for i := range id {
		id[i] = byte(rng.IntN(256))
	}
	// Set version 4
	id[6] = (id[6] & 0x0f) | 0x40
	// Set variant bits
	id[8] = (id[8] & 0x3f) | 0x80
	return id
}

// FormatUUID formats a 16-byte UUID as a standard string representation.
func FormatUUID(id [16]byte) string {
	buf := make([]byte, 36)
	hex := "0123456789abcdef"
	pos := 0
	for i, b := range id {
		if i == 4 || i == 6 || i == 8 || i == 10 {
			buf[pos] = '-'
			pos++
		}
		buf[pos] = hex[b>>4]
		buf[pos+1] = hex[b&0x0f]
		pos += 2
	}
	return string(buf)
}
