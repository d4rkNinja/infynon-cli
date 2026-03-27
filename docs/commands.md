# INFYNON Command Reference

## Package Security Mode

```
infynon pkg <subcommand>
```

### Scan

```bash
infynon pkg scan                              # scan all detected lock files
infynon pkg scan --pkg-file <PATH>            # scan specific file
infynon pkg scan --output markdown            # export Markdown report
infynon pkg scan --output pdf                 # export PDF report
infynon pkg scan --output both                # export both formats
infynon pkg scan --fix                        # auto-fix all vulnerabilities
infynon pkg scan --fix critical               # auto-fix critical only
infynon pkg scan --fix high                   # auto-fix critical + high
infynon pkg scan --fix medium                 # auto-fix critical + high + medium
infynon pkg scan --fix low                    # auto-fix all except informational
infynon pkg scan --fix informational          # auto-fix all (same as --fix)
```

### Secure Install

```bash
# npm
infynon pkg npm install <pkg>
infynon pkg npm install <pkg>@<version>

# yarn
infynon pkg yarn add <pkg>
infynon pkg yarn add <pkg>@<version>

# pnpm
infynon pkg pnpm add <pkg>
infynon pkg pnpm add <pkg>@<version>

# bun
infynon pkg bun add <pkg>
infynon pkg bun add <pkg>@<version>

# pip
infynon pkg pip install <pkg>
infynon pkg pip install <pkg>==<version>

# uv
infynon pkg uv pip install <pkg>
infynon pkg uv add <pkg>

# poetry
infynon pkg poetry add <pkg>
infynon pkg poetry add <pkg>==<version>

# cargo
infynon pkg cargo add <pkg>
infynon pkg cargo add <pkg>@<version>

# go
infynon pkg go get <module>
infynon pkg go get <module>@<version>

# gem
infynon pkg gem install <pkg>
infynon pkg gem install <pkg>:<version>

# composer
infynon pkg composer require <vendor/pkg>
infynon pkg composer require <vendor/pkg>:<version>

# nuget
infynon pkg nuget add <pkg>
infynon pkg nuget add <pkg> --version <version>

# hex (Elixir)
infynon pkg hex deps.get

# pub (Dart/Flutter)
infynon pkg pub add <pkg>
```

### Strict Mode (CI)

```bash
infynon pkg --strict npm install <pkg>                # block all severities
infynon pkg --strict critical npm install <pkg>       # block critical only
infynon pkg --strict high npm install <pkg>           # block critical + high
infynon pkg --strict medium npm install <pkg>         # block critical + high + medium
infynon pkg --strict low npm install <pkg>            # block all except informational
infynon pkg --strict pip install <pkg>
infynon pkg --strict cargo add <pkg>
```

### Auto-Detect Ecosystem

```bash
infynon pkg install <pkg>                     # detects from manifest files
infynon pkg add <pkg>
```

## Firewall Engine Mode (Upcoming)

```bash
infynon                                       # show project info
infynon daemon                                # start background CVE intelligence
infynon dashboard                             # open TUI security dashboard
infynon update-intel                          # manually refresh CVE intel
```
