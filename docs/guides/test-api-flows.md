# Test API Flows

Use Weave to test chained backend behavior from the terminal.

## Set base URL

```bash
infynon weave env set BASE_URL http://localhost:8000
```

## Create nodes

```bash
infynon weave node create
infynon weave node create --ai "POST /auth/login extracts token"
```

## Create a flow

```bash
infynon weave flow create auth-flow --ai "login then get profile"
```

## Run a flow

```bash
infynon weave flow run auth-flow
infynon weave flow run auth-flow --format json --no-input
```

## Add assertions

```bash
infynon weave node assertion login add "status == 200"
```

## Related docs

- [weave command](../commands/weave.md)
- [Generated files](../reference/generated-files.md)
