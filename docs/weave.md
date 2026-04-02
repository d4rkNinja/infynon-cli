# infynon weave

If your team also uses Loom for shared coding memory, the recommended Claude Code companion skill is:
`https://github.com/d4rkNinja/code-guardian`

Node-based API flow testing TUI. Model your API as a directed graph of HTTP nodes, thread context variables between them, and run security scans — all from the terminal.

## Usage

```
infynon weave <subcommand>
```

---

## Variable Types

Weave has two distinct types of variables. Choosing correctly is the most important design decision.

### The rule of thumb

| | Env var `{$KEY}` | Prompt input `{var}` |
|--|--|--|
| **Set** | Once per project/environment | Fresh every test run |
| **Who sets it** | Developer configuring the project | Person running the test |
| **Examples** | `BASE_URL`, `API_VERSION`, `X_API_KEY` | `email`, `phone_number`, `otp_code`, `password` |
| **Where stored** | `.infynon/.env` | Declared on the node via `[[prompt_inputs]]` |

**Never put user-specific test data in `.infynon/.env`.** Email addresses, phone numbers, OTP codes, and passwords change per tester and per run. Storing them in env means they go stale immediately.

### Env Variables — `{$KEY}`

Static project-wide config only. Set once, used by all APIs in the project.

```bash
# Good candidates for env vars
infynon weave env set BASE_URL http://localhost:8001   # required — where is the server?
infynon weave env set API_VERSION v1                   # version prefix used across all paths
infynon weave env set SHARED_API_KEY abc123            # key shared by all requests
infynon weave env set API_TOKEN eyJhbGc...             # sensitive — auto-masked in TUI

# List, inspect, remove
infynon weave env list
infynon weave env get API_TOKEN --reveal
infynon weave env delete OLD_KEY
```

Or manage interactively in the TUI on **Tab 6 (Env / Ctx)**:
- `n` — add new variable
- `Enter` — edit selected variable
- `d` — delete selected variable
- `v` — reveal / hide sensitive values

Reference in any node field with `{$VAR_NAME}`:

```
Path:    /api/{$API_VERSION}/users
Header:  Authorization: Bearer {$API_TOKEN}
Body:    {"api_key": "{$SHARED_API_KEY}"}
```

**BASE_URL is required.** If not set, the flow/node will refuse to run with a clear error. Resolution order:
1. `--base-url` flag (CLI only, overrides for that run)
2. `base_url` stored on the flow
3. `BASE_URL` from `.infynon/.env`
4. Error — cannot run

### Runtime Variables — `{var}` + prompt inputs

Values that are user-specific or change every run: emails, phone numbers, OTPs, passwords. These are **asked at the moment the flow reaches that specific node**.

**In the TUI:** a popup modal appears right before the node fires. Fill in each field, press Tab to move between them, Enter on the last field to submit. The flow immediately continues.

**In the CLI:** the terminal pauses and prompts you for each value inline.

The value is injected into the node's path, headers, and body wherever `{var}` appears.

### Context propagation — ask once, use everywhere

When a value is entered (or extracted from a response) in an earlier node, it stays in the **flow context** and flows to all downstream nodes automatically — without re-prompting.

```
send-email asks for {email}
    ↓ email is now in context
verify-email uses {email} from context — NOT asked again
    ↓ email still in context
register uses {email} from context — NOT asked again
```

Design your flows so user-specific values are prompted on the **first node that needs them**. All downstream nodes just reference `{var_name}`.

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
| `1` | Overview | All nodes and flows — press `Enter` or `a` to run |
| `2` | Flow Graph | Visual DAG of the active flow's node connections |
| `3` | Live Execution | Real-time step-by-step output — auto-switches on run |
| `4` | Latency Profiler | Per-node timing breakdown |
| `5` | Security Probes | Active security probe results (SQLi, XSS, auth, etc.) |
| `6` | Env / Ctx | Manage `.infynon/.env` variables + view last-run context |
| `7` | State Inspector | Final context values after the last run + schema drift |
| `8` | Run Diff | Compare two runs side-by-side |
| `9` | Node Library | Browse all nodes — press `Enter` or `r` to run |
| `0` | Config | Toggle markdown/PDF output |

### TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `1`–`9`, `0` | Switch tabs |
| `q` | Quit |
| `Enter` / `r` | Run selected flow (tab 1) or node (tab 9) |
| `a` | Run all flows (tab 1) |
| `↑` / `↓` | Navigate list |
| `n` | Add new env variable (tab 6) |
| `d` | Delete selected env variable (tab 6) |
| `v` | Reveal / hide sensitive env values (tab 6) |
| `m` | Toggle markdown output (tab 0) |
| `p` | Toggle PDF output (tab 0) |

### Live Execution (Tab 3)

The live execution view shows a real-time feed of each step as it runs. After a run completes (or fails), you can interact with the results:

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate between steps |
| `Enter` / `Space` | Open step detail inspector — shows full request/response body, all assertion results, extracted variables, and error messages |
| `r` | **Retry** — re-run the current flow from the beginning |
| `b` | **Modify body** — open the inline body editor for the selected step's node. Edit the JSON body and save with `Ctrl+S`, then press `r` to retry |
| `Esc` | Close step detail |

> **Tip:** The typical debugging cycle is: run → see failure on a step → press `b` to fix the node body → press `r` to retry.

### Runtime Prompt Modal (TUI)

When the flow reaches a node that has runtime prompt inputs, execution **pauses** and a modal appears. Each prompt input can have a different type that changes how the modal behaves:

**`text` (default)** — free-text input, masked with `●` if `--secret`:
```
┌─ ◆ Input Required — api-v1-auth-otp-verify-mobile ──────────────┐
│  Provide values before the request fires:                        │
│  ┌─ Mobile OTP (check your SMS) ──────────────────────────────┐ │
│  │  847291▌                                                   │ │
│  └────────────────────────────────────────────────────────────┘ │
│  Tab next field  ↑↓ navigate  Enter confirm  Esc cancel          │
└──────────────────────────────────────────────────────────────────┘
```

**`boolean`** — yes/no toggle. Press `y`, `n`, or `Space` to toggle:
```
┌─ ◆ Input Required — delete-user ────────────────────────────────┐
│  ┌─ Confirm delete? ─────────────────────────────────────────┐  │
│  │  [ Yes ]  [ No ]   y/n or Space to toggle                 │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

**`select`** — single choice, navigate with `↑`/`↓`, confirm with `Enter`:
```
┌─ ◆ Input Required — create-order ───────────────────────────────┐
│  ┌─ Target environment ──────────────────────────────────────┐  │
│  │   ▶ staging                                               │  │
│  │     production                                            │  │
│  │     dev                                                   │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

**`multiselect`** — multiple choices. `↑`/`↓` to navigate, `Space` to check/uncheck, `Enter` to confirm:
```
┌─ ◆ Input Required — create-token ───────────────────────────────┐
│  ┌─ Token scopes ─────────────── Space: toggle  Enter: confirm ┐ │
│  │  [✔] read                                                   │ │
│  │  [✔] write                                                  │ │
│  │  [ ] admin                                                  │ │
│  └──────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```
`multiselect` values are stored as a comma-joined string: `"read,write"`.

**CLI modal (terminal):** uses the same types via `dialoguer`:
- `boolean` → `[y/n]` confirm prompt
- `select` → interactive arrow-key list
- `multiselect` → checkbox list with `Space` to toggle

---

## Nodes

A node is a single HTTP request definition — method, path, headers, body, variable extractions, assertions, and optional runtime prompt inputs.

### Create a Node

```bash
infynon weave node create              # interactive wizard
infynon weave node create --ai "create a user with name and email"
```

The wizard prompts for:
1. Node ID (kebab-case, e.g. `verify-otp`)
2. Name / description
3. HTTP method (`GET` / `POST` / `PUT` / `PATCH` / `DELETE` / `HEAD`)
4. Path — can contain `{$ENV_VAR}` and `{context_var}` placeholders
5. Request body (JSON) — can contain `{placeholder}` variables
6. Headers
7. Variable extractions (`name=body.data.id`, blank to finish)
8. Assertions (`status == 200`, blank to finish)

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
# BASE_URL must be set in .infynon/.env, or pass --base-url
infynon weave node run <node-id>
infynon weave node run <node-id> --base-url http://staging.example.com

# Inject known context values upfront
infynon weave node run <node-id> --set token=abc123 --set user_id=42

# Prompt interactively for any {placeholder} not in env or --set
infynon weave node run <node-id> --prompt
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

## Runtime Prompt Inputs

Add runtime prompt inputs to any node that needs values you can only know at execution time — OTP codes, 2FA tokens, dynamic passwords, captcha answers.

### How it works

1. You define a prompt input on the node: a variable name, a label, and whether to mask the input.
2. In the node's body/path/headers you use `{var}` as a placeholder.
3. When the flow reaches that node, **before** the HTTP request fires, the user is asked to enter the value.
4. The value replaces `{var}` in the request.

### List Prompt Inputs

```bash
infynon weave node prompt <node-id> list
```

### Add a Prompt Input

| Flag | Description |
|------|-------------|
| `--label "..."` | Human-readable text shown to the user. Defaults to the variable name. |
| `--secret` | Masks input with `●` characters. Use for passwords and tokens. |
| `--default "value"` | Pre-filled value the user can accept (press Enter) or override. Essential for CI. |
| `--type` | Interaction style: `text` (default), `boolean`, `select`, `multiselect` |
| `--options "a,b,c"` | Comma-separated choices. Required for `select` and `multiselect`. |

```bash
# Basic — asks "code" at runtime, value goes into {code} in the body
infynon weave node prompt <node-id> add code

# With a friendly label shown to the user
infynon weave node prompt <node-id> add code --label "Mobile OTP (check your SMS)"

# Mask input with * (for passwords / tokens)
infynon weave node prompt <node-id> add admin_password --label "Admin password" --secret

# With a default value the user can accept or override
infynon weave node prompt <node-id> add env_name --label "Environment" --default staging

# With an explicit type
infynon weave node prompt <node-id> add env --label "Environment" --type select --options "staging,production,dev" --default staging
infynon weave node prompt <node-id> add confirm --label "Confirm delete?" --type boolean --default false
infynon weave node prompt <node-id> add scopes --label "Token scopes" --type multiselect --options "read,write,admin" --default "read,write"
```

### Prompt Input Types

| Type | Description | `options` needed? |
|------|-------------|-------------------|
| `text` | Free-text input (default) | No |
| `boolean` | Yes / No toggle | No |
| `select` | Single choice from list | Yes |
| `multiselect` | Multiple choices from list | Yes |

**Examples in TOML** (for reference — always use the CLI to manage these):
```toml
[[prompt_inputs]]
var = "env"
label = "Target environment"
type = "select"
options = ["staging", "production", "dev"]
default = "staging"

[[prompt_inputs]]
var = "confirm_delete"
label = "Confirm delete operation?"
type = "boolean"
default = "false"

[[prompt_inputs]]
var = "scopes"
label = "API scopes to include"
type = "multiselect"
options = ["read", "write", "admin"]
default = "read"

[[prompt_inputs]]
var = "otp_code"
label = "OTP from SMS"
type = "text"
```

**CLI examples:**
```bash
# text (default)
infynon weave node prompt login add email --label "Test email"

# boolean
infynon weave node prompt delete-user add confirm --label "Confirm delete?" --type boolean --default false

# select
infynon weave node prompt create-order add env --label "Environment" --type select --options "staging,production,dev" --default staging

# multiselect
infynon weave node prompt create-token add scopes --label "Token scopes" --type multiselect --options "read,write,admin" --default "read,write"
```

### Remove a Prompt Input

```bash
infynon weave node prompt <node-id> list    # find the index first
infynon weave node prompt <node-id> remove 0
```

### Full Example — OTP Flow (send + verify)

This example shows the correct pattern: `email`/`phone_number` are prompted once on the send nodes and carried automatically to verify nodes and registration.

```bash
# Step 1: Add email prompt to the send-email node
infynon weave node prompt api-v1-auth-otp-send-email add email \
  --label "Email address to send OTP to"

# The body uses {email}: {"email":"{email}"}

# Step 2: verify-email gets {email} from context (no prompt needed)
# Only prompt for the OTP code itself
infynon weave node prompt api-v1-auth-otp-verify-email add email_code \
  --label "Email OTP (check your inbox)"

# Step 3: Add phone prompts to the send-mobile node
infynon weave node prompt api-v1-auth-otp-send-mobile add country_code \
  --label "Country code (e.g. +91)" \
  --default "+91"
infynon weave node prompt api-v1-auth-otp-send-mobile add phone_number \
  --label "Phone number (digits only)"

# Step 4: verify-mobile gets {country_code} and {phone_number} from context
# Only prompt for the SMS OTP
infynon weave node prompt api-v1-auth-otp-verify-mobile add code \
  --label "Mobile OTP (check your SMS)"

# Step 5: Run the flow — it prompts for user data at the right moments
infynon weave flow run karnsha-merchant-onboarding
```

At runtime (TUI or CLI):
```
→  Running karnsha-merchant-onboarding

◆ Input Required — api-v1-auth-otp-send-email
  Email address to send OTP to
  › test@example.com▌

✔  send-email  POST  200  89ms

◆ Input Required — api-v1-auth-otp-verify-email
  Email OTP (check your inbox)
  › 847291▌

✔  verify-email  POST  200  45ms
...
```

The body that fires for verify-mobile: `{"country_code":"+91","number":"9876543210","code":"847291"}`

### Prompt Inputs vs `--prompt` flag

| | Prompt inputs (`node prompt add`) | `--prompt` flag |
|---|---|---|
| **Scope** | Defined on the node permanently | CLI flag for one-time run |
| **Trigger** | Always asks when that node runs | Asks for any unresolved `{var}` not in env or `--set` |
| **Label** | Custom label you set | Uses variable name as label |
| **Secret masking** | Configurable per-input | No masking |
| **Works in TUI** | Yes — popup modal | No (CLI only) |
| **Use for** | OTPs, passwords, anything that changes every run | Quick one-off debugging |

**Use `node prompt add` for anything you want to work in the TUI flow.**

---

## Assertions

Assertions verify the response after each node runs.

### List Assertions

```bash
infynon weave node assertion <node-id> list
```

Output:
```
  [0] ✔  status == 200                              (stop)
  [1] ✔  header.content-type contains application/json  (warn)
  [2] ✘  body.data.verified == true                 (warn)   ← disabled
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
| `header.content-type contains application/json` | Header contains string |

---

## Flows

A flow is a directed graph of nodes. Nodes connect via edges; context variables are carried forward so later nodes can use values extracted from earlier responses.

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
# BASE_URL must be in .infynon/.env, or pass --base-url
infynon weave flow run <flow-id>
infynon weave flow run <flow-id> --base-url http://staging.example.com

# Seed context variables before the first node runs
infynon weave flow run <flow-id> --set user_id=42 --set role=admin

# Save a report
infynon weave flow run <flow-id> --output markdown
infynon weave flow run <flow-id> --output pdf
infynon weave flow run <flow-id> --output both
```

Reports are saved to `./reports/` as `<flow-id>-<timestamp>.md` / `.pdf`.

When a flow reaches a node with prompt inputs defined, **it pauses and asks for the value** before continuing. You do not need any flag for this — it happens automatically.

### Run All Flows

```bash
infynon weave flow run-all
infynon weave flow run-all --base-url http://staging.example.com
infynon weave flow run-all --set env=staging --output both
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

---

## Attaching and Detaching Nodes

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

## Variable Extraction

Extractions pull values from a response into the flow context so subsequent nodes can reference them.

Define extractions when creating a node:

```
token=body.data.access_token    → stores body.data.access_token as "token"
user_id=body.data.id            → stores body.data.id as "user_id"
location=header.location        → stores the Location response header
code=status                     → stores the HTTP status code
```

Reference extracted values in later nodes:

```
Path:    /api/v1/users/{user_id}
Header:  Authorization: Bearer {token}
Body:    {"owner_id": "{user_id}"}
```

---

## Importing from OpenAPI / Swagger

```bash
infynon weave import openapi.yaml --dry-run           # preview, no files written
infynon weave import openapi.yaml                     # import all endpoints
infynon weave import openapi.yaml --flow "My Flow"    # import + create flow
infynon weave import openapi.yaml --prefix /api/v1   # only this path prefix
infynon weave import openapi.yaml --base-url http://localhost:4000
```

**Supported:** OpenAPI 3.x (`.yaml`, `.yml`, `.json`) and Swagger 2.x.

Each endpoint generates:
- Node ID from `operationId` or `METHOD-path`
- Body template with `{field_name}` placeholders
- Variable extractions for `id`, `token`, `*_id`, `*_token`, `*_url` response fields
- `status == 2xx` and `body exists` assertions
- `Authorization: Bearer {$AUTH_TOKEN}` header on protected endpoints

---

## Env Command Reference

```bash
infynon weave env list                    # list all variables (sensitive masked)
infynon weave env set <KEY> <VALUE>       # add or update a variable
infynon weave env get <KEY>               # show a variable (sensitive masked)
infynon weave env get <KEY> --reveal      # show full value
infynon weave env delete <KEY>            # remove a variable
```

Variables are stored in `.infynon/.env`. Sensitive keys (containing `TOKEN`, `SECRET`, `KEY`, `PASSWORD`, `AUTH`, etc.) are automatically masked in all output.

---

## Validate

```bash
infynon weave validate
```

Checks:
- Node: ID not empty, valid HTTP method, path starts with `/`, body is valid JSON, extractions use valid prefixes
- Flow: entry node exists, all edge nodes exist, no circular references

Exits with code `1` if any errors found — suitable for CI.

---

## AI Commands

```bash
infynon weave ai suggest --after <node-id>         # suggest next node
infynon weave ai attach --after <node-id>          # auto-attach best next node
infynon weave ai attach --after <node-id> --flow <flow-id>
infynon weave ai complete <flow-id>                # fill all unconnected nodes
infynon weave ai probe <flow-id>                   # run security probes
infynon weave ai probe <flow-id> --base-url http://staging.example.com
infynon weave ai build-flow --nodes login,get-profile,create-post --name "user-journey"
infynon weave ai explain <flow-id>                 # explain last run failure
infynon weave ai explain <flow-id> --run 1
infynon weave ai assert <node-id>                  # generate assertions from schema
infynon weave ai branch <node-id>                  # suggest conditional branches
```

---

## File Structure

```
.infynon/
├── .env                    ← env variables (BASE_URL, API_TOKEN, etc.)
└── api/
    ├── nodes/              ← one .toml file per node
    │   ├── login.toml
    │   └── verify-otp.toml
    ├── flows/              ← one .toml file per flow
    │   └── onboarding.toml
    └── runs/               ← run result JSON files
```

---

## CI Usage

```bash
# Validate — exits 1 on error
infynon weave validate

# Run a flow — exits 1 if any assertion fails
infynon weave flow run <flow-id> --base-url $API_URL

# Run all flows and save a report
infynon weave flow run-all --base-url $API_URL --output markdown
```

`{$ENV_VAR}` placeholders resolve from CI environment variables automatically — no `.env` file needed in CI.

Nodes with `prompt_inputs` work in CI in two ways:

**Option 1 — `--default` on each prompt input (recommended)**
Set a `--default` value on every prompt input. The CLI uses it automatically in non-interactive mode — the run never blocks. The human running interactively can override it.

```bash
infynon weave node prompt register add full_name --label "Full name" --default "CI Test User"
infynon weave node prompt register add password  --label "Password"  --secret --default "Test@1234"
```

**Option 2 — `--set` to pre-seed all vars**
Pass every prompt var as `--set KEY=VALUE`. Weave skips the prompt for any var already in context.

```bash
infynon weave flow run auth-flow \
  --set email=ci@example.com \
  --set phone_number=9999999999 \
  --set country_code=+91 \
  --set full_name="CI Bot" \
  --set password=Test@1234
# Note: OTP nodes (email_code, code) still block — use a test inbox or mock SMS service for those
```
