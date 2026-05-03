# Verify INFYNON Downloads

Each GitHub Release includes a `checksums.txt` file.

## macOS and Linux

```bash
sha256sum -c checksums.txt
```

## Windows PowerShell

```powershell
Get-FileHash .\infynon-x86_64-pc-windows-msvc.exe -Algorithm SHA256
```

Compare the resulting hash with the matching entry in `checksums.txt`.
