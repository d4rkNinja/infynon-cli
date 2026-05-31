# Agent Release Rules

## Release Tag Guardrails

Follow this checklist every time you cut a release tag.

1. Pick the release version, for example `0.2.0-beta.9.0.9`.
2. Update all release metadata files to the same version:
   - `Cargo.toml` (`[package].version`)
   - `npm/package.json` (`version`)
   - `go/internal/installer/installer.go` (`const version = "..."`)
3. Validate versions before tagging:
   - `python scripts/verify-release-versions.py v<version>`
4. Commit the version and workflow changes to `main`.
5. Create an annotated tag using the `v` prefix:
   - `git tag -a v<version> -m "Release v<version>"`
6. Push branch first, then push the tag:
   - `git push origin main`
   - `git push origin v<version>`

## Why this is required

- `.github/workflows/release.yml` triggers on tags matching `v*`.
- The release workflow runs `scripts/verify-release-versions.py`.
- If Go/Cargo/npm versions are not synced before tagging, release fails.
