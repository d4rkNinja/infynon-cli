# Environment Variables

INFYNON can use environment variables for CI and integrations.

## Common variables

| Variable | Purpose |
|---|---|
| `CI` | Enables non-interactive package behavior in CI environments |
| `GEMINI_SYSTEM_MD` | Used when launching Gemini with INFYNON context |

## Weave variables

Weave project variables are managed with:

```bash
infynon weave env set BASE_URL http://localhost:8000
infynon weave env list
```

Common Weave variable:

| Variable | Purpose |
|---|---|
| `BASE_URL` | Base URL used for API node and flow execution |
