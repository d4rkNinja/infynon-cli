// Package main keeps the legacy /go install path working.
//
// Prefer:
//
//	go install github.com/d4rkNinja/infynon-cli/go/cmd/infynon@latest
package main

import "github.com/d4rkNinja/infynon-cli/go/internal/installer"

func main() {
	installer.Main()
}
