# infynon pkg &lt;ecosystem&gt; install

Secure proxy for package installation. Checks packages against OSV before passing through to the native package manager.

## Usage

```
infynon pkg <ecosystem> <command> <packages...> [--strict]
```

### Universal Spec Parser & Binary Resolution
INFYNON features a robust, cross-platform engine that:
- Uses a **universal package spec parser** to accurately handle different version formats (`name@version`, `name==version`, etc.) for all 14 supported ecosystems.
- Employs **native OS resolution** (`where` on Windows, `which` on Unix) to reliably detect and execute native binaries (like `npm`, `cargo`, `pip`) without relying on hardcoded paths.

## Ecosystems & Commands

| Ecosystem | Install Command |
|-----------|----------------|
| npm | `infynon pkg npm install <pkg>` |
| yarn | `infynon pkg yarn add <pkg>` |
| pnpm | `infynon pkg pnpm add <pkg>` |
| bun | `infynon pkg bun add <pkg>` |
| pip | `infynon pkg pip install <pkg>` |
| uv | `infynon pkg uv pip install <pkg>` |
| uv | `infynon pkg uv add <pkg>` |
| poetry | `infynon pkg poetry add <pkg>` |
| cargo | `infynon pkg cargo add <pkg>` |
| go | `infynon pkg go get <pkg>` |
| gem | `infynon pkg gem install <pkg>` |
| composer | `infynon pkg composer require <pkg>` |
| nuget | `infynon pkg nuget add <pkg>` |
| hex | `infynon pkg hex deps.get` |
| pub | `infynon pkg pub add <pkg>` |

## Auto-Detect

```bash
# Omit ecosystem — infynon detects from manifest files
infynon pkg install express       # detects npm from package.json
infynon pkg add serde             # detects cargo from Cargo.toml
```

## Strict Mode

```bash
infynon pkg --strict npm install express           # block all severities
infynon pkg --strict critical npm install express   # block critical only
infynon pkg --strict high npm install express       # block critical + high
infynon pkg --strict medium npm install express     # block critical + high + medium
```

Blocks vulnerable packages at or above the specified severity level and exits with code 1. Defaults to blocking all severities. Designed for CI pipelines.

## Interactive Prompts

When vulnerabilities are found, you get 4 choices:

1. **Install anyway** — proceed with the vulnerable version
2. **Skip all** — don't install any vulnerable packages
3. **Install recommended** — use the safe version from the vulnerability database
4. **Decide per package** — choose individually for each package
