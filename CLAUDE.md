# Repository Guidelines

## Project Structure & Module Organization
- `src/` contains the Rust CLI and product modules: `cli/` for argument routing, `api/` for Weave flows, `trace/` for repo memory, `engine/` for package scanning, `tui/` for terminal UI, and `web/` for the embedded web server.
- `front-layer/` is the Vite + React frontend that is built and embedded into the Rust binary at compile time.
- `docs/` holds command and feature guides. `scripts/` contains install/build helpers. `npm/` and `go/` provide wrapper distribution targets.
- Build-time embedding is controlled by `build.rs`; treat it as part of the app runtime path, not as optional tooling.

## Build, Test, and Development Commands
- `cargo build` builds the Rust CLI and triggers frontend embedding when needed.
- `cargo install --path . --force` installs the local build onto the machine for real CLI testing.
- `cargo test` runs Rust tests.
- `npm run build` from `front-layer/` builds the web UI bundle explicitly.
- `npm run lint` and `npm run typecheck` from `front-layer/` validate frontend code.
- `infynon --host 127.0.0.1 --port 4173` runs the bundled web server locally.

## Coding Style & Naming Conventions
- Use Rust idioms: `snake_case` for functions/modules, `PascalCase` for types, and small focused modules.
- Keep frontend components in `PascalCase` files when they are app-level components; shared UI primitives follow the existing `front-layer/src/components/ui/` naming.
- For frontend feature work, prefer composing dedicated subcomponents instead of keeping large pages in a single file.
- Use the existing shadcn UI primitives from `front-layer/src/components/ui/` wherever possible instead of introducing ad-hoc replacements.
- Do not add hardcoded colors in frontend code. Use theme tokens and semantic utility classes such as `bg-card`, `text-muted-foreground`, `border-border`, `bg-primary`, and related token-based variants.
- Prefer ASCII source files. Match existing formatting; use `cargo fmt` only on files you intend to change.
- Avoid path-dependent runtime behavior. New web/runtime features should work from any current working directory.

## Testing Guidelines
- Add Rust unit tests next to the module they cover when logic is non-trivial.
- For frontend changes, run `npm run build`, `npm run lint`, and `npm run typecheck` before shipping.
- For root CLI/web changes, verify both `infynon --help` and a live health check such as `GET /health`.

## Commit & Pull Request Guidelines
- Follow the existing history style: `feat: ...`, `refactor: ...`, `ci: ...`, `release: ...`, or scoped forms like `feat(trace): ...`.
- Keep commits narrow and explain behavior changes, not just file edits.
- PRs should include: purpose, affected commands/modules, verification steps, and screenshots for visible frontend changes.

## Security & Packaging Notes
- Do not rely on machine-specific install paths for frontend assets; the supported pattern is compile-time embedding.
- If you add SSL or packaging changes, verify behavior on Windows, Linux, and macOS assumptions before merging.
