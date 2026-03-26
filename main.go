// Package rowing-machine generates an arbitrary amount of simulated data based on a fictional store called Rowing Outfitters.
package main

import (
	"github.com/fatih/color"
)

func main() {
	pinkUnderline := color.New(color.FgCyan, color.BgMagenta, color.Bold)
	println(pinkUnderline.Sprintf("Welcome to the Jaffle Shop!"))
}
