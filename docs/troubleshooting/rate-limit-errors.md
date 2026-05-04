# Rate Limit Errors

Use this page when package metadata, vulnerability lookups, or API-related commands fail because a remote service is rate limited.

## Retry later

Rate limits are often temporary.

```bash
infynon pkg scan
```

## Reduce repeated requests

Scan a specific file instead of repeatedly scanning the whole project:

```bash
infynon pkg scan --pkg-file package-lock.json
```

## Use CI carefully

If running in CI, avoid triggering many parallel jobs that query the same package metadata services at once.

## Notes

INFYNON depends on public package and vulnerability metadata where available. Those services may enforce limits.

## Related docs

- [pkg command](../commands/pkg.md)
- [Scan project dependencies](../guides/scan-project-dependencies.md)
