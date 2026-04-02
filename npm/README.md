# INFYNON

INFYNON is a CLI for:

- package intelligence with `infynon pkg`
- API flow testing with `infynon weave`
- shared coding memory with `infynon loom`

[![npm](https://img.shields.io/npm/v/infynon?style=flat-square&logo=npm)](https://www.npmjs.com/package/infynon)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/d4rkNinja/infynon-cli/blob/main/LICENSE)
[![GitHub](https://img.shields.io/badge/source-GitHub-black?style=flat-square&logo=github)](https://github.com/d4rkNinja/infynon-cli)

## Install

```bash
npm install -g infynon
```

This package downloads the matching native binary for your OS and architecture.

## Command Areas

### `infynon pkg`

- scan lockfiles for vulnerable packages
- secure install wrapper for multiple ecosystems
- audit, why, outdated, diff, doctor, fix, clean, migrate
- Eagle Eye scheduled package monitoring

```bash
infynon pkg scan
infynon pkg audit
infynon pkg npm install express --strict high
```

### `infynon weave`

- create API nodes and flows
- run connected request chains
- import OpenAPI
- prompt for runtime values
- run AI-assisted security probes

```bash
infynon weave env set BASE_URL http://localhost:8001
infynon weave flow create "checkout" --ai "login then create order"
infynon weave flow run checkout
```

### `infynon loom`

- canonical, team, and user memory layers
- Redis or SQL backends
- package notes that can identify who introduced a compromised dependency
- sync, retrieve, compact, and TUI inspection

```bash
infynon loom init --owner team --user alien
infynon loom source add-sql team-db --engine sqlite --url sqlite://.infynon/loom/loom.db --user alien --default
infynon loom note add repo-handoff --title "Auth changed" --body "Refresh moved into middleware"
infynon loom sync --direction both
```

## Backend Choice

Use Redis when you want:

- fast live retrieval
- active session state
- lower-latency coordination

Use SQL when you want:

- durable structured history
- stronger filtering and reports
- long-term canonical memory

## Documentation

- Root README: `README.md`
- Command reference: `docs/commands.md`
- Loom guide: `docs/loom.md`
- Weave guide: `docs/weave.md`

## License

MIT
