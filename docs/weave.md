# infynon weave

Node-based API flow testing and security probe TUI. Model your API as a directed graph of HTTP nodes, thread context variables between them, and run security scans — all from the terminal.

## Usage

```
infynon weave <subcommand>
```

---

## TUI

Open the interactive terminal dashboard.

```bash
infynon weave tui               # open overview (all nodes + flows)
infynon weave tui <flow-id>     # open a specific flow directly
```

### TUI Tabs

| Key | Tab | Description |
|-----|-----|-------------|
| `1` | Overview | All nodes and flows. Press `Enter` or `a` to run the selected flow |
| `2` | Flow Graph | Visual graph of the active flow's node connections |
| `3` | Live Execution | Real-time step-by-step run output — auto-switches when a run starts |
| `4` | Latency Profiler | Per-node timing and latency breakdown |
| `5` | Security Probes | Active security probe results (SQLi, XSS, auth, etc.) |
| `6` | Coverage Map | Which nodes have been exercised in this session |
| `7` | State Inspector | Current context variable values at each step |
| `8` | Run Diff | Compare two runs side-by-side |
| `9` | Node Library | Browse all nodes. Press `Enter` or `r` to run the selected node |
| `0` | Config | Toggle markdown/PDF output, set default base URL |

### TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `1`–`9`, `0` | Switch tabs |
| `q` | Quit TUI |
| `Enter` / `r` | Run selected flow (tab 1) or node (tab 9) |
| `a` | Run all flows (tab 1) |
| `↑` / `↓` | Navigate list |
| `m` | Toggle markdown output (Config tab) |
| `p` | Toggle PDF output (Config tab) |
| `e` | Edit default base URL (Config tab) |

---

## Nodes

A node is a single HTTP request definition — method, path, headers, body, variable extractions, and assertions.

### Create a Node

```bash
infynon weave node create              # interactive wizard
infynon weave node create --ai "create a user with name and email"
```

The interactive wizard prompts for:
1. Node ID (kebab-case, e.g. `create-user`)
2. Name / description
3. HTTP method (GET / POST / PUT / PATCH / DELETE / HEAD)
4. Path (e.g. `/api/v1/users`)
5. Request body (for POST/PUT/PATCH — JSON)
6. Headers (key: value, blank to finish)
7. Variable extractions (`name=body.data.id`, blank to finish)
8. Assertions (`status == 201`, blank to finish)

### List Nodes

```bash
infynon weave node list
```

### Get Node Details

```bash
infynon weave node get <node-id>
```

### Run a Node in Isolation

```bash
infynon weave node run <node-id>
infynon weave node run <node-id> --base-url http://staging.example.com
infynon weave node run <node-id> --set token=abc123 --set user_id=42
```

### Clone a Node

```bash
infynon weave node clone <node-id> <new-id>
```

### Export a Node

```bash
infynon weave node export <node-id>                    # curl command (default)
infynon weave node export <node-id> --format json      # JSON definition
infynon weave node export <node-id> --base-url https://api.example.com
```

### Remove a Node

```bash
infynon weave node remove <node-id>
```

---

## Assertions

Assertions verify the response after each node runs. Each assertion has an expression and an `on_fail` action (`stop` or `warn`). Assertions can be individually enabled or disabled without deleting them.

### List Assertions

```bash
infynon weave node assertion <node-id> list
```

Output shows index, enabled status, expression, and on_fail action:

```
  [0] ✔  status == 200   (stop)
  [1] ✔  body exists     (warn)
  [2] ✘  body.id != null (warn)   ← disabled
```

### Add an Assertion

```bash
infynon weave node assertion <node-id> add "status == 201"
infynon weave node assertion <node-id> add "body.token != null" --on-fail warn
```

`on-fail` values: `stop` (default — halts the flow) | `warn` (logs and continues)

### Enable / Disable / Toggle

```bash
infynon weave node assertion <node-id> enable  <index>
infynon weave node assertion <node-id> disable <index>
infynon weave node assertion <node-id> toggle  <index>
```

### Remove an Assertion

```bash
infynon weave node assertion <node-id> remove <index>
```

### Assertion Expressions

| Expression | Meaning |
|------------|---------|
| `status == 200` | HTTP status equals 200 |
| `status != 404` | HTTP status is not 404 |
| `status >= 200` | HTTP status in range |
| `body exists` | Response body is non-empty |
| `body.field == "value"` | JSON field equals value |
| `body.count > 0` | JSON numeric field comparison |
| `header.content-type contains application/json` | Header value contains string |

---

## Flows

A flow is a directed graph of nodes. Nodes are connected by edges; context variables can be carried between them so later nodes can use values extracted from earlier responses.

### Create a Flow

```bash
infynon weave flow create "My Flow"
infynon weave flow create "Onboarding Flow" --ai "register user, verify email, login, get profile"
```

### List Flows

```bash
infynon weave flow list
```

### Show Flow Graph

```bash
infynon weave flow show <flow-id>
```

### Run a Flow

```bash
infynon weave flow run <flow-id>
infynon weave flow run <flow-id> --base-url http://staging.example.com
infynon weave flow run <flow-id> --output markdown
infynon weave flow run <flow-id> --output pdf
infynon weave flow run <flow-id> --output both
```

Reports are saved to `./reports/` as `<flow-id>-<timestamp>.md` / `.pdf`.

### Run All Flows

```bash
infynon weave flow run-all
infynon weave flow run-all --base-url http://staging.example.com
infynon weave flow run-all --output both
```

### Remove a Flow

```bash
infynon weave flow remove <flow-id>
```

Nodes are **not** deleted when a flow is removed.

### Merge Two Flows

```bash
infynon weave flow merge <flow1-id> <flow2-id> --join-at <node-id>
infynon weave flow merge <flow1-id> <flow2-id> --join-at <node-id> --name "combined-flow"
```

Attaches `flow2` to `flow1` at the specified node.

---

## Attaching and Detaching Nodes

Connect nodes to form a flow graph. Edges define execution order and carry context variables forward.

```bash
# Basic edge
infynon weave attach <from-node-id> <to-node-id>

# Carry specific variables across the edge
infynon weave attach login get-profile --carry token,user_id

# Conditional edge (only follow if expression is true)
infynon weave attach create-user send-email --condition "status == 201"

# Let AI infer what to carry
infynon weave attach login get-profile --ai

# Remove an edge
infynon weave detach <from-node-id> <to-node-id>
```

---

## Importing from OpenAPI / Swagger

Generate nodes automatically from an existing API spec.

```bash
# Preview what would be imported (no files written)
infynon weave import openapi.yaml --dry-run

# Import all endpoints
infynon weave import openapi.yaml

# Import and create a flow from the imported nodes
infynon weave import openapi.yaml --flow "My API Flow"

# Import only endpoints under a path prefix
infynon weave import openapi.yaml --prefix /api/v1

# Override the base URL from the spec
infynon weave import openapi.yaml --base-url http://localhost:4000

# Combine flags
infynon weave import openapi.json --prefix /api/v1 --flow "V1 Flow" --base-url http://staging.example.com
```

**Supported formats:** OpenAPI 3.x (`.yaml`, `.yml`, `.json`) and Swagger 2.x.

**What gets generated per endpoint:**
- Node ID from `operationId` (camelCase → kebab-case) or `METHOD-path`
- Body template with `{field_name}` placeholders for all request schema properties
- Variable extractions for `id`, `token`, `*_id`, `*_token`, `*_url` fields in the response schema
- `status == <2xx>` assertion and `body exists` assertion
- `Authorization: Bearer {$AUTH_TOKEN}` header on non-auth endpoints

---

## Environment Variables

Reference environment variables in any node field using `{$VAR_NAME}` syntax. Variables are resolved from a `.env` file in the current directory first, then from the process environment.

```bash
# .env file
AUTH_TOKEN=eyJhbGciOiJIUzI1NiJ9...
API_KEY=sk-prod-abc123
BASE_URL=http://localhost:3000
```

Use in node headers, body, or path:

```
Authorization: Bearer {$AUTH_TOKEN}
X-Api-Key: {$API_KEY}
```

**Precedence:** `.env` file → process environment → left as-is if not found.

---

## Validate

Check all nodes and flows for correctness before running.

```bash
infynon weave validate
```

Checks performed:

**Nodes:**
- ID is not empty
- HTTP method is valid (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`)
- Path starts with `/`
- `body_json` is valid JSON (if present)
- Extractions use valid `from` prefixes (`body.*`, `header.*`, `status`)

**Flows:**
- Entry node is not empty
- Entry node exists in the node library
- All edge nodes (from/to) exist in the node library
- No circular references (cycle detection)

Example output:

```
  ◆ Validate — 7 nodes, 1 flow

  Nodes
  ✔  login
  ✔  get-profile
  ✔  create-post
  ⚠  update-post  body_json is not valid JSON
  ✔  delete-post

  Flows
  ✔  onboarding-flow

  1 warning(s) found
```

Exits with code `1` if any errors are found — suitable for CI pipelines.

---

## AI Commands

```bash
# Suggest the next node to add after a given node
infynon weave ai suggest --after <node-id>

# Auto-attach the best next node
infynon weave ai attach --after <node-id>
infynon weave ai attach --after <node-id> --flow <flow-id>

# Add all unconnected nodes to a flow
infynon weave ai complete <flow-id>

# Run security probes on a flow (SQLi, XSS, auth bypass, etc.)
infynon weave ai probe <flow-id>
infynon weave ai probe <flow-id> --base-url http://staging.example.com

# Build a flow from a list of node IDs
infynon weave ai build-flow --nodes login,get-profile,create-post --name "user-journey"

# Explain why the last flow run failed
infynon weave ai explain <flow-id>
infynon weave ai explain <flow-id> --run 1    # explain a specific run (0 = most recent)

# Generate assertions for a node based on its response schema
infynon weave ai assert <node-id>

# Suggest conditional branches (e.g. 201 vs 409 handling)
infynon weave ai branch <node-id>
```

---

## File Structure

Weave stores all data under `.infynon/api/` in your project directory.

```
.infynon/
└── api/
    ├── nodes/          ← one file per node (.toml or .yaml)
    │   ├── login.toml
    │   └── get-profile.yaml
    └── flows/          ← one file per flow (.toml or .yaml)
        └── onboarding-flow.toml
```

**Format detection:** Weave reads both `.toml` and `.yaml`/`.yml` files. When saving, it auto-detects the project's existing format (YAML if any `.yaml` nodes exist, TOML otherwise).

**Never create or edit these files manually.** Always use `infynon weave` commands. Manually written files may fail to load if the schema doesn't match.

---

## Variable Extraction

Extractions pull values out of a response and store them in the flow context so subsequent nodes can use them.

Define extractions when creating a node:

```
name=body.data.id          → stores response JSON body.data.id as "id"
token=body.access_token    → stores body.access_token as "token"
location=header.location   → stores the Location response header as "location"
code=status                → stores the HTTP status code as "code"
```

Reference extracted values in later nodes:

```
Path:    /api/v1/users/{user_id}
Header:  Authorization: Bearer {token}
Body:    { "owner_id": "{user_id}" }
```

---

## CI Usage

```bash
# Validate all nodes and flows — exits 1 on error
infynon weave validate

# Run a flow in CI — exits 1 if any assertion fails
infynon weave flow run <flow-id> --base-url $API_URL

# Run all flows and save a report
infynon weave flow run-all --base-url $API_URL --output markdown
```

`{$ENV_VAR}` placeholders resolve from CI environment variables automatically — no `.env` file needed in CI.
