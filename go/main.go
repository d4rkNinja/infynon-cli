// Package main provides a Go-installable wrapper for the INFYNON CLI.
//
// Install:
//
//	go install github.com/d4rkNinja/infynon-cli/go@latest
//
// This downloads the pre-built binary from GitHub Releases for your
// platform and places it in your $GOPATH/bin.
package main

import (
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
)

const (
	repo    = "d4rkNinja/infynon-cli"
	version = "0.2.0-beta.9.0.2"
)

func targetTriple() (target, ext string, ok bool) {
	switch runtime.GOOS + "/" + runtime.GOARCH {
	case "windows/amd64":
		return "x86_64-pc-windows-msvc", ".exe", true
	case "linux/amd64":
		return "x86_64-unknown-linux-musl", "", true
	case "linux/arm64":
		return "aarch64-unknown-linux-musl", "", true
	case "darwin/amd64":
		return "x86_64-apple-darwin", "", true
	case "darwin/arm64":
		return "aarch64-apple-darwin", "", true
	default:
		return "", "", false
	}
}

func binDir() string {
	if gopath := os.Getenv("GOPATH"); gopath != "" {
		return filepath.Join(gopath, "bin")
	}
	home, _ := os.UserHomeDir()
	return filepath.Join(home, "go", "bin")
}

func download(url, dest string) error {
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != 200 {
		return fmt.Errorf("download failed: HTTP %d from %s", resp.StatusCode, url)
	}

	f, err := os.Create(dest)
	if err != nil {
		return err
	}
	defer f.Close()

	_, err = io.Copy(f, resp.Body)
	return err
}

func main() {
	target, ext, ok := targetTriple()
	if !ok {
		fmt.Fprintf(os.Stderr, "[infynon] Unsupported platform: %s/%s\n", runtime.GOOS, runtime.GOARCH)
		fmt.Fprintf(os.Stderr, "         Build from source: cargo install --git https://github.com/%s\n", repo)
		os.Exit(1)
	}

	dir := binDir()
	binPath := filepath.Join(dir, "infynon"+ext)

	// If binary already exists, just exec it with args
	if _, err := os.Stat(binPath); err == nil {
		cmd := exec.Command(binPath, os.Args[1:]...)
		cmd.Stdin = os.Stdin
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			if exitErr, ok := err.(*exec.ExitError); ok {
				os.Exit(exitErr.ExitCode())
			}
			os.Exit(1)
		}
		return
	}

	// Download binary from GitHub releases
	tag := "v" + version
	asset := "infynon-" + target + ext
	url := fmt.Sprintf("https://github.com/%s/releases/download/%s/%s", repo, tag, asset)

	fmt.Printf("[infynon] Downloading %s from %s release...\n", asset, tag)

	if err := os.MkdirAll(dir, 0o755); err != nil {
		fmt.Fprintf(os.Stderr, "[infynon] Failed to create bin dir: %v\n", err)
		os.Exit(1)
	}

	if err := download(url, binPath); err != nil {
		fmt.Fprintf(os.Stderr, "[infynon] Download failed: %v\n", err)
		fmt.Fprintf(os.Stderr, "[infynon] Manual install: https://github.com/%s/releases/tag/%s\n", repo, tag)
		os.Exit(1)
	}

	if runtime.GOOS != "windows" {
		os.Chmod(binPath, 0o755)
	}

	fmt.Printf("[infynon] Installed to %s\n", binPath)

	// Run with any passed args
	if len(os.Args) > 1 {
		cmd := exec.Command(binPath, os.Args[1:]...)
		cmd.Stdin = os.Stdin
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		if err := cmd.Run(); err != nil {
			if exitErr, ok := err.(*exec.ExitError); ok {
				os.Exit(exitErr.ExitCode())
			}
			os.Exit(1)
		}
	} else {
		// Show version info after fresh install
		fmt.Printf("[infynon] Run: infynon --help\n")
	}
}
