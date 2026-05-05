# Windows Troubleshooting

## Install Command

Use the root-level PowerShell installer:

```powershell
iwr https://raw.githubusercontent.com/d4rkNinja/infynon-cli/main/install.ps1 -useb | iex
```

The installer downloads the Windows x64 release binary into:

```text
%USERPROFILE%\.infynon\bin
```

It also adds that directory to the user `PATH` when needed.

## Command Not Found

Restart PowerShell or Windows Terminal after install. Then verify:

```powershell
$env:Path -split ';' | Select-String '\.infynon\\bin'
Get-Command infynon
infynon --help
```

If the directory is missing from `PATH`, add `%USERPROFILE%\.infynon\bin` to the user environment variables.

## Download Blocks

Allow these hosts through proxies or endpoint filters:

```text
github.com
api.github.com
objects.githubusercontent.com
registry.npmjs.org
```

If PowerShell script execution is restricted, run the installer from a PowerShell session allowed by your organization, or download the release asset manually and verify it with `checksums.txt`.

## Claude Code Launches

INFYNON uses Claude Code's file-based system-prompt flag for task starts:

```bash
claude {model_arg} --append-system-prompt-file {quoted_task_start_system_prompt_path}
```

This avoids sending the large INFYNON task system prompt through argv on Windows, where long command lines and shell quoting are less reliable. The prompt file is written under `~/.infynon/ninja/` and refreshed by INFYNON when agent tasks start.

## npm Installs

For npm installs:

```powershell
npm install -g infynon
```

Use Node.js 18 or newer. If optional platform packages are blocked, the npm wrapper may fall back to the GitHub Release asset for the same version.
