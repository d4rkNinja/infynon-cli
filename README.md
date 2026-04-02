# INFYNON

CLI for package intelligence, API flow testing, and database-backed coding memory.

## Current Areas

### `infynon pkg`

- CVE scanning across 14 ecosystems
- secure install wrapper
- audit / why / outdated / diff / doctor / fix / clean / migrate
- scheduled monitoring with Eagle Eye

```bash
infynon pkg scan
infynon pkg audit
infynon pkg npm install express --strict high
infynon pkg fix --auto
```

### `infynon weave`

- node-based API flow testing
- context threading between requests
- OpenAPI import
- runtime prompt inputs
- live execution, run diff, and built-in security probes

```bash
infynon weave env set BASE_URL http://localhost:8001
infynon weave node create --ai "POST /auth/login extracts token"
infynon weave flow create "checkout" --ai "login then create order"
infynon weave flow run checkout
infynon weave ai probe checkout
```

### `infynon loom`

Shared coding memory with Redis or SQL backends.

- Redis for fast live retrieval and session-style coordination
- SQL for durable notes, structured queries, and long-term canonical memory
- repo-level default user for note attribution
- canonical / team / user memory layers
- PR / branch / file / package notes
- compaction and reconciliation
- TUI-based inspection, note browsing, and package risk ownership

```bash
infynon loom init --owner team --user alien
infynon loom source add-sql team-db --engine sqlite --url sqlite://.infynon/loom/loom.db --user alien --default
infynon loom note add repo-handoff --title "Auth changed" --body "Refresh moved into middleware"
infynon loom sync --direction both
```

## Root Command Style

```bash
infynon pkg <subcommand>
infynon weave <subcommand>
infynon loom <subcommand>
```

## Installation

### npm

```bash
npm install -g infynon
```

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.sh | bash
```

### Windows

```powershell
irm https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/scripts/install.ps1 | iex
```

### Source

```bash
cargo install --git https://github.com/d4rkNinja/infynon-cli
```

## Notes

- The repo is currently centered on `pkg`, `weave`, and `loom`.
- Loom docs: [docs/loom.md](docs/loom.md)
- Command reference: [docs/commands.md](docs/commands.md)
