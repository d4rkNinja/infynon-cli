# Scan Project Dependencies

Use dependency scans to check lockfiles and manifests for known risks.

## Scan current project

```bash
infynon pkg scan
```

## Scan one file

```bash
infynon pkg scan --pkg-file package-lock.json
infynon pkg scan --pkg-file Cargo.lock
infynon pkg scan --pkg-file uv.lock
```

## Export results

```bash
infynon pkg scan --json
infynon pkg scan --output markdown
infynon pkg scan --output pdf
infynon pkg scan --output both
```

## Fix vulnerable packages

```bash
infynon pkg scan --fix
infynon pkg scan --fix high
infynon pkg fix --auto
```

## Related docs

- [pkg command](../commands/pkg.md)
- [Exit codes](../reference/exit-codes.md)
