package installer

import (
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"time"
)

const (
	repo    = "d4rkNinja/infynon-cli"
	version = "0.2.0-beta.9.0.7"
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

func binDir() (string, error) {
	if exe, err := os.Executable(); err == nil {
		if dir := filepath.Dir(exe); dir != "." && dir != "" {
			return dir, nil
		}
	}
	if gobin := os.Getenv("GOBIN"); gobin != "" {
		return gobin, nil
	}
	if gopath := os.Getenv("GOPATH"); gopath != "" {
		return filepath.Join(gopath, "bin"), nil
	}
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(home, "go", "bin"), nil
}

func download(url, dest string) error {
	if err := os.MkdirAll(filepath.Dir(dest), 0o755); err != nil {
		return err
	}

	tmp, err := os.CreateTemp(filepath.Dir(dest), ".infynon-download-*")
	if err != nil {
		return err
	}
	tmpPath := tmp.Name()
	removeTmp := true
	defer func() {
		if removeTmp {
			_ = os.Remove(tmpPath)
		}
	}()

	client := http.Client{Timeout: 60 * time.Second}
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		_ = tmp.Close()
		return err
	}
	req.Header.Set("User-Agent", "infynon-go-installer")

	resp, err := client.Do(req)
	if err != nil {
		_ = tmp.Close()
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		_ = tmp.Close()
		return fmt.Errorf("download failed: HTTP %d from %s", resp.StatusCode, url)
	}

	if _, err := io.Copy(tmp, resp.Body); err != nil {
		_ = tmp.Close()
		return err
	}
	if err := tmp.Close(); err != nil {
		return err
	}
	if runtime.GOOS != "windows" {
		if err := os.Chmod(tmpPath, 0o755); err != nil {
			return err
		}
	}
	if err := os.Rename(tmpPath, dest); err != nil {
		return err
	}
	removeTmp = false
	return nil
}

func runBinary(binPath string, args []string) {
	cmd := exec.Command(binPath, args...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			os.Exit(exitErr.ExitCode())
		}
		fmt.Fprintf(os.Stderr, "[infynon] Failed to run binary: %v\n", err)
		os.Exit(1)
	}
}

func Main() {
	target, ext, ok := targetTriple()
	if !ok {
		fmt.Fprintf(os.Stderr, "[infynon] Unsupported platform: %s/%s\n", runtime.GOOS, runtime.GOARCH)
		fmt.Fprintf(os.Stderr, "         Build from source: cargo install --git https://github.com/%s\n", repo)
		os.Exit(1)
	}

	dir, err := binDir()
	if err != nil {
		fmt.Fprintf(os.Stderr, "[infynon] Could not resolve install directory: %v\n", err)
		os.Exit(1)
	}
	binPath := filepath.Join(dir, "infynon"+ext)

	if _, err := os.Stat(binPath); err == nil {
		runBinary(binPath, os.Args[1:])
		return
	}

	tag := "v" + version
	asset := "infynon-" + target + ext
	url := fmt.Sprintf("https://github.com/%s/releases/download/%s/%s", repo, tag, asset)

	fmt.Printf("[infynon] Downloading %s from %s release...\n", asset, tag)
	if err := download(url, binPath); err != nil {
		fmt.Fprintf(os.Stderr, "[infynon] Download failed: %v\n", err)
		fmt.Fprintf(os.Stderr, "[infynon] Manual install: https://github.com/%s/releases/tag/%s\n", repo, tag)
		os.Exit(1)
	}

	fmt.Printf("[infynon] Installed to %s\n", binPath)
	if len(os.Args) > 1 {
		runBinary(binPath, os.Args[1:])
		return
	}
	fmt.Println("[infynon] Run: infynon --help")
}
