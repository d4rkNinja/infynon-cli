# INFYNON Command Reference

## Root

```bash
infynon <command>
```

Top-level commands:

- `infynon pkg`
- `infynon weave`
- `infynon loom`

## Package Intelligence

```bash
infynon pkg <subcommand>
```

### Scan

```bash
infynon pkg scan
infynon pkg scan --pkg-file <PATH>
infynon pkg scan --output markdown
infynon pkg scan --output pdf
infynon pkg scan --output both
infynon pkg scan --fix
infynon pkg scan --fix high
```

### Secure Install

```bash
infynon pkg npm install <pkg>
infynon pkg yarn add <pkg>
infynon pkg pnpm add <pkg>
infynon pkg bun add <pkg>
infynon pkg pip install <pkg>
infynon pkg uv add <pkg>
infynon pkg poetry add <pkg>
infynon pkg cargo add <pkg>
infynon pkg go get <module>
infynon pkg gem install <pkg>
infynon pkg composer require <vendor/pkg>
infynon pkg nuget add <pkg>
infynon pkg hex deps.get
infynon pkg pub add <pkg>
```

### Strict Mode

```bash
infynon pkg --strict npm install <pkg>
infynon pkg --strict high npm install <pkg>
infynon pkg --strict pip install <pkg>
infynon pkg --strict cargo add <pkg>
```

### Other Package Commands

```bash
infynon pkg audit
infynon pkg why <package>
infynon pkg outdated
infynon pkg diff <pkg> <v1> <v2>
infynon pkg doctor
infynon pkg size <pkg>
infynon pkg search <query>
infynon pkg fix --auto
infynon pkg clean
infynon pkg migrate npm pnpm
```

### Eagle Eye

```bash
infynon pkg eagle-eye setup
infynon pkg eagle-eye start
infynon pkg eagle-eye status
infynon pkg eagle-eye enable
infynon pkg eagle-eye disable
```

## Weave

```bash
infynon weave <subcommand>
```

### Common Commands

```bash
infynon weave tui
infynon weave env set BASE_URL http://localhost:8001
infynon weave node create
infynon weave node create --ai "POST /auth/login extracts token"
infynon weave node run <node-id> --prompt
infynon weave flow create "checkout" --ai "login then create order"
infynon weave flow run <flow-id>
infynon weave flow run-all
infynon weave ai probe <flow-id>
infynon weave import openapi.yaml --flow "My Flow"
infynon weave validate
```

## Loom

```bash
infynon loom overview
```

### Setup

```bash
infynon loom init --owner team --user alien
infynon loom source add-redis team-redis --url redis://localhost:6379/0 --namespace infynon --user alien --default
infynon loom source add-sql team-db --engine sqlite --url sqlite://.infynon/loom/loom.db --user alien --default
infynon loom source list
infynon loom source default team-db
```

### Notes And Retrieval

```bash
infynon loom note add repo-handoff --title "Auth changed" --body "Refresh moved to middleware" --layer team --scope branch --target feature/auth-refresh --files src/auth.rs --tags auth,handoff
infynon loom note update repo-handoff --status stale
infynon loom note remove repo-handoff
infynon loom note list
infynon loom retrieve --scope branch --target auth
infynon loom retrieve --scope package --target chrono
```

### Sync And Operations

```bash
infynon loom sync --direction push
infynon loom sync --direction pull
infynon loom sync --direction both
infynon loom compact
infynon loom schema sql
infynon loom schema redis
infynon loom tui
```
