# INFYNON Command Guide

INFYNON is organized into command areas. Each area supports a different part of the development workflow: package risk, API behavior, and repository context.

Use the built-in help for the exact command surface available in your installed version:

```bash
infynon --help
infynon pkg --help
infynon weave --help
infynon trace --help
```

## Package Intelligence: `infynon pkg`

Use `infynon pkg` when you need to understand dependency risk, inspect installed packages, or run package-related security workflows.

Common entry points:

```bash
infynon pkg scan
infynon pkg audit
infynon pkg explain <package>
infynon pkg outdated
infynon pkg diff
infynon pkg doctor
```

Typical workflow:

```bash
infynon pkg scan
infynon pkg audit
infynon pkg explain serde_json
```

Use this area when:

- you are reviewing a dependency change
- you need a local package risk report
- you want to understand why a package is present
- you want package security checks before continuing with development

## API Flow Testing: `infynon weave`

Use `infynon weave` when API validation depends on multiple connected requests rather than one isolated request.

Common entry points:

```bash
infynon weave env set BASE_URL http://localhost:8001
infynon weave node create
infynon weave flow create checkout
infynon weave flow run checkout
```

Typical workflow:

```bash
infynon weave env set BASE_URL http://localhost:8001
infynon weave flow run checkout --format json --no-input
```

Use this area when:

- a request depends on a previous login, setup, or resource creation step
- you need repeatable API validation from a terminal
- you want flow output suitable for automation
- you need request context threaded across steps

## Repository Memory: `infynon trace`

Use `infynon trace` when repository context needs to survive beyond chat messages, local notes, or one-off handoffs.

Common entry points:

```bash
infynon trace init
infynon trace note add repo-handoff --title "Auth changed" --body "Refresh moved into middleware"
infynon trace retrieve --scope branch --target main
infynon trace sync --direction both
infynon trace tui
```

Use this area when:

- you need structured handoff notes
- package ownership or branch context matters
- you want context available for later AI-assisted work
- you need a terminal interface for inspecting repo memory

## Suggested Team Workflows

### Before merging dependency changes

```bash
infynon pkg scan
infynon pkg audit
```

### Before shipping API changes

```bash
infynon weave flow run checkout --format json --no-input
```

### Before handing off a branch

```bash
infynon trace note add repo-handoff --title "Branch handoff" --body "Summarize the important implementation context here"
infynon trace sync --direction both
```

## Exit Codes and Automation

Treat INFYNON like a normal CLI in automation:

- inspect command-specific help before wiring a command into CI
- pin the INFYNON version for reproducible environments
- store generated reports as CI artifacts when needed
- fail builds only on command modes your team has agreed to enforce

## More Help

Run:

```bash
infynon <command> --help
```

for the exact flags supported by the installed version.
