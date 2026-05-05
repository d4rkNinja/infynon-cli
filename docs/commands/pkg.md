# pkg

Use `pkg` to search packages, verify installs, and scan dependency files.

## When to use

Use this command before installing packages or when checking dependency risk in a project.

## Basic usage

```bash
infynon pkg <subcommand>
```

## Common commands

| Command | Description |
|---|---|
| `infynon pkg search <query>` | Search package registries |
| `infynon pkg npm install <package>` | Verify and install npm package |
| `infynon pkg yarn add <package>` | Verify and install with Yarn |
| `infynon pkg pnpm add <package>` | Verify and install with pnpm |
| `infynon pkg bun add <package>` | Verify and install with Bun |
| `infynon pkg pip install <package>` | Verify and install Python package with pip |
| `infynon pkg uv add <package>` | Verify and install Python package with uv |
| `infynon pkg poetry add <package>` | Verify and install Python package with Poetry |
| `infynon pkg cargo add <package>` | Verify and install Rust package |
| `infynon pkg scan` | Scan detected dependency files |
| `infynon pkg scan --pkg-file <file>` | Scan a specific dependency file |
| `infynon pkg scan --output markdown|pdf|both` | Export a scan report |
| `infynon pkg scan --fix [level]` | Scan and run fix workflow |
| `infynon pkg audit` | Audit project dependencies |
| `infynon pkg outdated` | Show outdated dependencies |
| `infynon pkg why <package>` | Explain why a package exists |
| `infynon pkg explain <package>` | Explain package risk and next steps |
| `infynon pkg diff <package> <v1> <v2>` | Compare package versions |
| `infynon pkg size <package...>` | Estimate package size |
| `infynon pkg doctor` | Diagnose package setup issues |
| `infynon pkg fix --auto` | Run automatic remediation |
| `infynon pkg clean` | Clean package state where supported |
| `infynon pkg migrate <from> <to>` | Help migrate package managers |
| `infynon pkg eagle-eye setup` | Configure scheduled monitoring |
| `infynon pkg eagle-eye start` | Start monitoring |
| `infynon pkg eagle-eye status` | Show monitoring status |
| `infynon pkg eagle-eye enable` | Enable monitoring |
| `infynon pkg eagle-eye disable` | Disable monitoring |

## Global options

| Option | Description |
|---|---|
| `--strict [level]` | Block or gate installs by severity level |
| `--pkg-file <file>` | Use a specific package file |
| `--json` | Emit machine-readable JSON |
| `--no-input` | Disable interactive prompts |
| `--yes` | Non-interactive install-all mode |
| `--skip-vulnerable` | Skip vulnerable packages |
| `--auto-fix` | Install fixed versions where possible |

## Examples

### Search packages

```bash
infynon pkg search fastapi
```

### Install with verification

```bash
infynon pkg npm install axios
infynon pkg uv add requests
infynon pkg cargo add serde
```

### Scan dependency files

```bash
infynon pkg scan --pkg-file package-lock.json
infynon pkg scan --json
```

## Notes

Package checks are based on available public vulnerability and package metadata. Always review results before accepting changes.

## Related docs

- [Secure package install](../guides/secure-package-install.md)
- [Scan project dependencies](../guides/scan-project-dependencies.md)
- [Supported package managers](../reference/supported-package-managers.md)
