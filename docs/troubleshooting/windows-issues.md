# Windows Issues

Common Windows issues include path length limits, spaces in usernames, PowerShell wrapper problems, and global npm path conflicts.

## Check INFYNON path

```powershell
where infynon
infynon --version
```

## Check npm global path

```powershell
npm root -g
npm bin -g
```

## Reinstall INFYNON

```powershell
npm uninstall -g infynon
npm install -g infynon
```

## Long path issue

If you see errors related to long filenames or extensions, try moving the project to a shorter path.

Example:

```txt
D:\infyn
```

instead of:

```txt
C:\Users\<name>\Documents\very\long\project\path
```

## PowerShell installer

```powershell
irm https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.ps1 | iex
```

## Related docs

- [Installation issues](./installation-issues.md)
- [Permission errors](./permission-errors.md)
