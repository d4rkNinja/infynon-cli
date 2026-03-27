# INFYNON Command Reference

## Package Security Mode

```
infynon pkg <subcommand>
```

### Scan

```bash
infynon pkg scan                              # scan all detected lock files
infynon pkg scan --pkg-file <PATH>            # scan specific file
infynon pkg scan --output markdown            # export Markdown report
infynon pkg scan --output pdf                 # export PDF report
infynon pkg scan --output both                # export both formats
infynon pkg scan --fix                        # auto-fix all vulnerabilities
infynon pkg scan --fix critical               # auto-fix critical only
infynon pkg scan --fix high                   # auto-fix critical + high
infynon pkg scan --fix medium                 # auto-fix critical + high + medium
infynon pkg scan --fix low                    # auto-fix all except informational
infynon pkg scan --fix informational          # auto-fix all (same as --fix)
```

### Secure Install

```bash
# npm
infynon pkg npm install <pkg>
infynon pkg npm install <pkg>@<version>

# yarn
infynon pkg yarn add <pkg>
infynon pkg yarn add <pkg>@<version>

# pnpm
infynon pkg pnpm add <pkg>
infynon pkg pnpm add <pkg>@<version>

# bun
infynon pkg bun add <pkg>
infynon pkg bun add <pkg>@<version>

# pip
infynon pkg pip install <pkg>
infynon pkg pip install <pkg>==<version>

# uv
infynon pkg uv pip install <pkg>
infynon pkg uv add <pkg>

# poetry
infynon pkg poetry add <pkg>
infynon pkg poetry add <pkg>==<version>

# cargo
infynon pkg cargo add <pkg>
infynon pkg cargo add <pkg>@<version>

# go
infynon pkg go get <module>
infynon pkg go get <module>@<version>

# gem
infynon pkg gem install <pkg>
infynon pkg gem install <pkg>:<version>

# composer
infynon pkg composer require <vendor/pkg>
infynon pkg composer require <vendor/pkg>:<version>

# nuget
infynon pkg nuget add <pkg>
infynon pkg nuget add <pkg> --version <version>

# hex (Elixir)
infynon pkg hex deps.get

# pub (Dart/Flutter)
infynon pkg pub add <pkg>
```

### Strict Mode (CI)

```bash
infynon pkg --strict npm install <pkg>                # block all severities
infynon pkg --strict critical npm install <pkg>       # block critical only
infynon pkg --strict high npm install <pkg>           # block critical + high
infynon pkg --strict medium npm install <pkg>         # block critical + high + medium
infynon pkg --strict low npm install <pkg>            # block all except informational
infynon pkg --strict pip install <pkg>
infynon pkg --strict cargo add <pkg>
```

### Auto-Detect Ecosystem

```bash
infynon pkg install <pkg>                     # detects from manifest files
infynon pkg add <pkg>
```

### Audit

Deep recursive dependency audit with visual tree and CVE highlights.

```bash
infynon pkg audit                          # audit detected lock files
infynon pkg audit --pkg-file Cargo.lock    # audit specific file
```

### Why

Trace why a package is present in your dependency tree.

```bash
infynon pkg why <package>
infynon pkg why serde
infynon pkg why lodash --pkg-file package-lock.json
```

### Outdated

Check for outdated dependencies across all detected ecosystems.

```bash
infynon pkg outdated
infynon pkg outdated --pkg-file Cargo.lock
```

Shows a table with current version, latest version, and update type (MAJOR / MINOR / PATCH).

### Diff

Compare two versions of a package — size, dependencies, install scripts, and CVEs.

```bash
infynon pkg diff <pkg> <v1> <v2>
infynon pkg diff express 4.17.1 4.18.2
infynon pkg diff serde 1.0.150 1.0.196 --ecosystem cargo
infynon pkg diff requests 2.28.0 2.31.0 --ecosystem pypi
```

### Doctor

Health check for your dependency tree.

```bash
infynon pkg doctor
infynon pkg doctor --pkg-file Cargo.lock
```

Checks:
- Duplicate package versions
- Unused dependencies (imported but not in lock file)
- Phantom dependencies (used in source but not declared)
- Missing lock files
- Risky install/preinstall scripts

### Size

Show install weight, bundle size, and transitive dependency count for packages.

```bash
infynon pkg size express
infynon pkg size serde tokio --ecosystem cargo
infynon pkg size requests --ecosystem pypi
```

### Search

Cross-ecosystem package search. Queries npm, crates.io, PyPI, RubyGems, Packagist, and pub.dev.

```bash
infynon pkg search <query>
infynon pkg search http-client
infynon pkg search json --ecosystem cargo
```

### Fix

Auto-fix all vulnerable dependencies to their nearest safe version.

```bash
infynon pkg fix --auto
infynon pkg fix --auto --pkg-file Cargo.lock
```

Delegates to `infynon pkg scan --fix all` internally.

### Clean

Find and remove unused dependencies interactively.

```bash
infynon pkg clean
infynon pkg clean --pkg-file package-lock.json
```

Detects unused deps, shows a list, prompts for confirmation, then runs the appropriate uninstall command per ecosystem.

### Migrate

Migrate your project between package managers.

```bash
infynon pkg migrate npm pnpm
infynon pkg migrate yarn bun
infynon pkg migrate pip uv
infynon pkg migrate pip poetry
```

Supported JS migrations: npm ↔ yarn ↔ pnpm ↔ bun
Supported Python migrations: pip ↔ uv ↔ poetry

Runs a vulnerability scan after migration completes.

---

## Firewall Engine Mode (Upcoming)

```bash
infynon                                       # show project info
infynon daemon                                # start background CVE intelligence
infynon dashboard                             # open TUI security dashboard
infynon update-intel                          # manually refresh CVE intel
```
