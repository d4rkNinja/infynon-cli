# Supported Package Managers

INFYNON supports package scanning and safe install workflows across common ecosystems.

## Common install wrappers

| Command | Ecosystem |
|---|---|
| `infynon pkg npm install <pkg>` | npm |
| `infynon pkg yarn add <pkg>` | Yarn |
| `infynon pkg pnpm add <pkg>` | pnpm |
| `infynon pkg bun add <pkg>` | Bun |
| `infynon pkg pip install <pkg>` | pip |
| `infynon pkg uv add <pkg>` | uv |
| `infynon pkg poetry add <pkg>` | Poetry |
| `infynon pkg cargo add <pkg>` | Cargo |

## Common files

| File | Ecosystem |
|---|---|
| `package-lock.json` | npm |
| `yarn.lock` | Yarn |
| `pnpm-lock.yaml` | pnpm |
| `requirements.txt` | pip |
| `uv.lock` | uv |
| `poetry.lock` | Poetry |
| `pyproject.toml` | Python |
| `Cargo.lock` | Rust |
| `go.mod`, `go.sum` | Go |
| `Gemfile.lock` | Ruby |
| `composer.lock` | PHP |
| `packages.lock.json` | NuGet |
| `mix.lock` | Hex |
| `pubspec.lock` | Dart |
