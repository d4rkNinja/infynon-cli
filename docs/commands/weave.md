# weave

Use `weave` to create, run, and replay API workflows.

## When to use

Use this command when you need to test chained API behavior, pass values between requests, validate responses, or run API flows in CI.

## Basic usage

```bash
infynon weave <subcommand>
```

## Common commands

| Command | Description |
|---|---|
| `infynon weave tui [flow-id]` | Open API flow TUI |
| `infynon weave env list` | List flow environment variables |
| `infynon weave env set <key> <value>` | Set environment variable |
| `infynon weave env get <key> [--reveal]` | Get environment variable |
| `infynon weave env delete <key>` | Delete environment variable |
| `infynon weave node create` | Create API node interactively |
| `infynon weave node create --ai <description>` | Create node from AI description |
| `infynon weave node list` | List nodes |
| `infynon weave node get <node-id>` | Show node |
| `infynon weave node clone <node-id> <new-id>` | Clone node |
| `infynon weave node run <node-id>` | Run one node |
| `infynon weave node export <node-id>` | Export node as curl or JSON |
| `infynon weave node remove <node-id>` | Remove node |
| `infynon weave node assertion <node-id> list` | List node assertions |
| `infynon weave node assertion <node-id> add <check>` | Add assertion |
| `infynon weave node assertion <node-id> enable <index>` | Enable assertion |
| `infynon weave node assertion <node-id> disable <index>` | Disable assertion |
| `infynon weave node assertion <node-id> toggle <index>` | Toggle assertion |
| `infynon weave node assertion <node-id> remove <index>` | Remove assertion |
| `infynon weave node prompt <node-id> list` | List runtime prompts |
| `infynon weave node prompt <node-id> add <var>` | Add runtime prompt |
| `infynon weave node prompt <node-id> remove <index>` | Remove runtime prompt |
| `infynon weave flow create <name>` | Create flow |
| `infynon weave flow list` | List flows |
| `infynon weave flow show <flow-id>` | Show flow |
| `infynon weave flow run <flow-id>` | Run one flow |
| `infynon weave flow run-all` | Run all flows |
| `infynon weave flow merge <flow-a> <flow-b> --join-at <node-id>` | Merge flows |
| `infynon weave flow remove <flow-id>` | Remove flow |
| `infynon weave attach <from> <to>` | Attach node to node |
| `infynon weave detach <from> <to>` | Detach node from node |
| `infynon weave import <spec>` | Import OpenAPI or compatible spec |
| `infynon weave validate` | Validate saved flow definitions |
| `infynon weave ai probe <flow-id>` | Run AI-assisted probes |
| `infynon weave ai explain <flow-id>` | Explain last flow run |
| `infynon weave ai build-flow --nodes <nodes>` | Build flow from nodes |

## Examples

### Set base URL

```bash
infynon weave env set BASE_URL http://localhost:8000
```

### Run a flow

```bash
infynon weave flow run auth-flow --format json --no-input
```

### Add prompt input

```bash
infynon weave node prompt login add otp --label "OTP code" --type text
```

## Notes

Flow run exit codes are documented in [Exit codes](../reference/exit-codes.md).

## Related docs

- [Test API flows](../guides/test-api-flows.md)
- [Generated files](../reference/generated-files.md)
