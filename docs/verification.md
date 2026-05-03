# Verify INFYNON Downloads

Each INFYNON GitHub Release includes a `checksums.txt` file containing SHA-256 hashes for the release assets.

Verification is recommended for managed environments, CI images, shared developer machines, and any manual binary download.

## Download the Release Files

Download the binary for your platform and the matching `checksums.txt` file from:

```text
https://github.com/d4rkNinja/infynon-cli/releases
```

## macOS and Linux

From the directory containing the downloaded binary and `checksums.txt`:

```bash
sha256sum -c checksums.txt
```

To verify only one asset:

```bash
sha256sum infynon-x86_64-unknown-linux-musl
```

Compare the output with the matching line in `checksums.txt`.

## Windows PowerShell

```powershell
Get-FileHash .\infynon-x86_64-pc-windows-msvc.exe -Algorithm SHA256
```

Compare the `Hash` value with the matching entry in `checksums.txt`.

## Expected Asset Names

| Platform | Asset |
|---|---|
| Windows x64 | `infynon-x86_64-pc-windows-msvc.exe` |
| Linux x64 | `infynon-x86_64-unknown-linux-musl` |
| Linux arm64 | `infynon-aarch64-unknown-linux-musl` |
| macOS Intel | `infynon-x86_64-apple-darwin` |
| macOS Apple Silicon | `infynon-aarch64-apple-darwin` |

## Integrity Notes

Checksum verification confirms that the downloaded file matches the release artifact published for that version. It does not replace normal endpoint security controls such as HTTPS, trusted release channels, endpoint protection, or internal software approval processes.

For locked-down environments, mirror only the specific release assets your organization has approved.
