package installer

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"time"
)

const (
	repo    = "d4rkNinja/infynon-cli"
	version = "0.2.12"
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
	if dir := os.Getenv("INFYNON_INSTALL_DIR"); dir != "" {
		return dir, nil
	}
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(home, ".infynon", "bin"), nil
}

func download(url, dest string, executable bool) error {
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
	if executable && runtime.GOOS != "windows" {
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

func isSHA256Hex(value string) bool {
	if len(value) != 64 {
		return false
	}
	_, err := hex.DecodeString(value)
	return err == nil
}

func checksumForAsset(checksumsPath, asset string) (string, error) {
	data, err := os.ReadFile(checksumsPath)
	if err != nil {
		return "", err
	}
	for _, line := range strings.Split(string(data), "\n") {
		fields := strings.Fields(strings.TrimSpace(line))
		if len(fields) < 2 {
			continue
		}
		hash := strings.ToLower(fields[0])
		name := strings.TrimPrefix(fields[1], "*")
		if isSHA256Hex(hash) && filepath.Base(name) == asset {
			return hash, nil
		}
	}
	return "", fmt.Errorf("checksums.txt does not include %s", asset)
}

func sha256File(path string) (string, error) {
	file, err := os.Open(path)
	if err != nil {
		return "", err
	}
	defer file.Close()

	hash := sha256.New()
	if _, err := io.Copy(hash, file); err != nil {
		return "", err
	}
	return hex.EncodeToString(hash.Sum(nil)), nil
}

func verifyChecksum(checksumsPath, filePath, asset string) error {
	expected, err := checksumForAsset(checksumsPath, asset)
	if err != nil {
		return err
	}
	actual, err := sha256File(filePath)
	if err != nil {
		return err
	}
	if actual != expected {
		return fmt.Errorf("SHA-256 mismatch for %s", asset)
	}
	return nil
}

func runBinary(binPath string, args []string) {
	if runtime.GOOS == "windows" {
		estimatedLen := len(binPath)
		for _, arg := range args {
			estimatedLen += len(arg) + 3
		}
		if estimatedLen > 30000 {
			fmt.Fprintln(os.Stderr, "[infynon] Command line is too long for Windows process creation; reduce arguments or use file-based package-manager input.")
			os.Exit(1)
		}
	}
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

func binaryMatchesVersion(binPath string) bool {
	output, err := exec.Command(binPath, "--version").CombinedOutput()
	if err != nil {
		return false
	}
	for _, field := range strings.Fields(string(output)) {
		if strings.TrimPrefix(field, "v") == version {
			return true
		}
	}
	return false
}

func Main() {
	target, ext, ok := targetTriple()
	if !ok {
		fmt.Fprintf(os.Stderr, "[infynon] Unsupported platform: %s/%s\n", runtime.GOOS, runtime.GOARCH)
		fmt.Fprintf(os.Stderr, "[infynon] Manual install: https://github.com/%s/releases\n", repo)
		os.Exit(1)
	}

	dir, err := binDir()
	if err != nil {
		fmt.Fprintf(os.Stderr, "[infynon] Could not resolve install directory: %v\n", err)
		os.Exit(1)
	}
	binPath := filepath.Join(dir, "infynon-native"+ext)

	if _, err := os.Stat(binPath); err == nil {
		if binaryMatchesVersion(binPath) {
			runBinary(binPath, os.Args[1:])
			return
		}
		fmt.Printf("[infynon] Existing native binary at %s is not version %s; reinstalling.\n", binPath, version)
	}

	tag := "v" + version
	asset := "infynon-" + target + ext
	url := fmt.Sprintf("https://github.com/%s/releases/download/%s/%s", repo, tag, asset)
	checksumsURL := fmt.Sprintf("https://github.com/%s/releases/download/%s/checksums.txt", repo, tag)
	tempSuffix := fmt.Sprintf(".%d", os.Getpid())
	tmpBinPath := binPath + ".download" + tempSuffix
	checksumsPath := binPath + ".checksums" + tempSuffix + ".txt"
	_ = os.Remove(tmpBinPath)
	_ = os.Remove(checksumsPath)
	defer os.Remove(tmpBinPath)
	defer os.Remove(checksumsPath)

	fmt.Printf("[infynon] Downloading %s from %s release...\n", asset, tag)
	if err := download(url, tmpBinPath, true); err != nil {
		fmt.Fprintf(os.Stderr, "[infynon] Download failed: %v\n", err)
		fmt.Fprintf(os.Stderr, "[infynon] Manual install: https://github.com/%s/releases/tag/%s\n", repo, tag)
		os.Exit(1)
	}
	if err := download(checksumsURL, checksumsPath, false); err != nil {
		fmt.Fprintf(os.Stderr, "[infynon] Checksum download failed: %v\n", err)
		fmt.Fprintf(os.Stderr, "[infynon] Manual install: https://github.com/%s/releases/tag/%s\n", repo, tag)
		os.Exit(1)
	}
	if err := verifyChecksum(checksumsPath, tmpBinPath, asset); err != nil {
		fmt.Fprintf(os.Stderr, "[infynon] Binary verification failed: %v\n", err)
		fmt.Fprintf(os.Stderr, "[infynon] Reinstall after the release asset is corrected.\n")
		os.Exit(1)
	}
	_ = os.Remove(binPath)
	if err := os.Rename(tmpBinPath, binPath); err != nil {
		fmt.Fprintf(os.Stderr, "[infynon] Install failed: %v\n", err)
		os.Exit(1)
	}
	if !binaryMatchesVersion(binPath) {
		_ = os.Remove(binPath)
		fmt.Fprintf(os.Stderr, "[infynon] Installed binary did not report version %s.\n", version)
		os.Exit(1)
	}

	fmt.Printf("[infynon] Installed to %s\n", binPath)
	if len(os.Args) > 1 {
		runBinary(binPath, os.Args[1:])
		return
	}
	fmt.Println("[infynon] Run: infynon --help")
}
