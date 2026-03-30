# infynon weave — Runtime Prompt Inputs

Some API endpoints require values that can't be known in advance: OTP codes, 2FA tokens, CAPTCHA responses, confirmation codes sent via email or SMS, or any other dynamic input that only the user can supply at run time.

Prompt inputs let a node declare variables that are **requested from the user interactively** the moment the node is about to fire. The user types a value, it is injected into the request as a `{var}` placeholder, and execution continues.

---

## How It Works

1. Define one or more prompt inputs on a node (via CLI or TUI).
2. When the flow reaches that node, execution **pauses**.
3. The user is shown a labeled input form.
4. The user enters the values and presses Enter.
5. The values are injected into the request path, headers, and body as `{var_name}` placeholders.
6. The node fires with those values in context — extraction and assertions run normally.

---

## Managing Prompt Inputs via CLI

### List prompt inputs on a node

```bash
infynon weave node prompt <node-id> list
```

Output:

```
  ◆ Prompt Inputs — verify-otp

  [0]  otp_code     "OTP Code"     secret: false   default: —
  [1]  session_id   "Session ID"   secret: false   default: abc123
```

### Add a prompt input

```bash
# Basic — var name becomes the label
infynon weave node prompt verify-otp add otp_code

# Custom label
infynon weave node prompt verify-otp add otp_code --label "OTP Code from SMS"

# Secret — masks input with * characters
infynon weave node prompt verify-otp add admin_password --label "Admin Password" --secret

# With a pre-filled default the user can accept or override
infynon weave node prompt verify-otp add session_id --label "Session ID" --default "abc123"
```

`--label` — human-readable text shown to the user at runtime. Defaults to the variable name if omitted.
`--secret` — masks the typed value with `*` characters. Use for passwords and sensitive tokens.
`--default` — pre-filled value. The user sees it and can press Enter to accept, or type to override.

### Remove a prompt input

```bash
# Use `list` first to find the index
infynon weave node prompt verify-otp remove 0
```

---

## Using Prompt Values in a Node

Reference prompt variable names with `{var_name}` exactly like any other context variable:

**Path:**
```
/api/v1/verify/{otp_code}
```

**Body:**
```json
{
  "otp": "{otp_code}",
  "session": "{session_id}"
}
```

**Headers:**
```
X-Auth-Token: {admin_password}
```

Prompt values are injected into context before the request is built. They are also available to downstream nodes via edge `--carry` just like extracted variables.

---

## CLI Run Behavior

When you run a node or flow via CLI and a node has prompt inputs, execution pauses and prompts appear inline in the terminal:

```
  Node 'verify-otp' needs input:

  OTP Code from SMS: _
  Session ID [abc123]: _
```

- **Secret fields** use a hidden input (no echo).
- **Fields with a default** show the default in brackets — press Enter to accept it.
- **Non-secret fields** show typed characters normally.

---

## TUI Run Behavior

When running a flow or single node from the TUI and execution reaches a node with prompt inputs, a **modal overlay** appears over the Live Execution tab:

```
╔═══════════════════════════════════════╗
║ ◆ Input Required — verify-otp        ║
║                                       ║
║  This node needs values before it     ║
║  can send the request.                ║
║                                       ║
║  OTP Code from SMS                    ║
║  › 847291▌                            ║
║                                       ║
║  Session ID                           ║
║  › abc123 (default)                   ║
║                                       ║
║  Tab/↓ next  ↑ prev  Enter submit     ║
║  Esc cancel                           ║
╚═══════════════════════════════════════╝
```

**TUI modal keys:**

| Key | Action |
|-----|--------|
| Any character | Type value into current field |
| `Backspace` | Delete last character |
| `Tab` / `↓` | Move to next field |
| `↑` | Move to previous field |
| `Enter` | Advance to next field; submit on last field |
| `Esc` | Cancel — sends empty values and skips the node |

Secret fields display typed characters as `*`. Fields with a default show the default in dim text until the user starts typing.

---

## Interactive Body Editor (TUI)

The Node Library tab (tab 9) includes an interactive body editor for editing a node's JSON request body directly from the TUI without leaving the terminal.

### Opening the Editor

Navigate to tab 9 (Node Library), select a node with `↑`/`↓`, then press `b`.

The editor opens as a full-screen overlay with line numbers and a block cursor:

```
╔═══════════════════════════════════════════╗
║ ◆ Edit Body — create-user (6 lines)      ║
║                                           ║
║   1  {                                   ║
║   2    "name": "▌",                      ║
║   3    "email": "{email}",               ║
║   4    "role": "user",                   ║
║   5    "active": true                    ║
║   6  }                                   ║
║                                           ║
║  Ctrl+S save  Esc cancel  ↑↓←→ move     ║
╚═══════════════════════════════════════════╝
```

### Body Editor Keys

| Key | Action |
|-----|--------|
| Any character | Insert at cursor |
| `Tab` | Insert 2 spaces |
| `Enter` | Insert newline (split line) |
| `Backspace` | Delete character before cursor; if at line start, merge with previous line |
| `Delete` | Delete character at cursor; if at line end, merge next line |
| `↑` / `↓` | Move cursor up/down one line |
| `←` / `→` | Move cursor left/right; wraps across lines |
| `Home` | Move to start of line |
| `End` | Move to end of line |
| `Ctrl+S` | Save body to node file and close editor |
| `Esc` | Close editor without saving |

### Save Behavior

- On save (`Ctrl+S`), the edited text is validated as JSON.
- If valid, it is compacted (minified) for storage in the node file and pretty-printed when the editor reopens.
- If invalid JSON, the raw text is saved as-is — the node will still run but `validate` will flag it.
- A notification appears confirming the save.

### When a Node Has No Body

Opening the body editor on a GET or other bodyless node shows `{}` as the starting content. You can add a body and save it — the node will gain a `body_json` field.

---

## Example: OTP Verification Flow

```bash
# Create the OTP verification node
infynon weave node create
# → ID: verify-otp
# → Method: POST
# → Path: /api/v1/auth/verify
# → Body: {"otp": "{otp_code}", "session_id": "{session_id}"}

# Add prompt inputs
infynon weave node prompt verify-otp add otp_code --label "OTP Code" --secret
infynon weave node prompt verify-otp add session_id --label "Session Token"

# Connect to the login node (which extracts session_id)
infynon weave attach login verify-otp --carry session_id

# Run the flow — pauses at verify-otp and asks for the OTP
infynon weave flow run onboarding-flow
```

---

## Tips

- **Chain with extracted values:** The `session_id` in the example above is extracted from the login response and carried to `verify-otp`. Users only need to type the OTP — everything else flows automatically.
- **Use `--secret` for anything sensitive:** Even in CI environments where no human sees the terminal, `--secret` prevents the value from appearing in logs or process arguments.
- **Defaults reduce typing in dev:** Set `--default` to a known-good test value so local runs are faster. Override it by typing a new value.
- **Prompt inputs in CI:** In CI pipelines, supply values via environment variables. Define them as `{$ENV_VAR}` placeholders in the node body/headers and use `prompt_inputs` only for values that truly require human input. Alternatively, pre-seed context via `--set` in `node run`.
