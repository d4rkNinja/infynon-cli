use std::process::{Command, Stdio};

// ── Binary detection ──────────────────────────────────────────────────────────

/// Checks if a binary is available on PATH using the OS's native resolver.
/// Windows: `where <binary>` — resolves .cmd, .bat, .exe in PATH correctly.
/// macOS / Linux: `which <binary>` — standard POSIX resolver.
pub fn is_installed(binary: &str) -> bool {
    if native_which(binary) {
        return true;
    }
    // Also check known alternative names for the same tool
    for alt in alt_binaries(binary) {
        if native_which(alt) {
            return true;
        }
    }
    false
}

/// Returns the actual binary name that is available on PATH.
/// Tries the primary name first, then falls through alternatives.
/// Falls back to the primary name if nothing is found (command will fail with a clear OS error).
pub fn resolve_binary(primary: &str) -> String {
    if native_which(primary) {
        return primary.to_string();
    }
    for alt in alt_binaries(primary) {
        if native_which(alt) {
            return alt.to_string();
        }
    }
    primary.to_string()
}

/// Run the OS-native binary resolver.
fn native_which(binary: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        // `where` is built into Windows and correctly resolves .cmd/.bat/.exe/.ps1 in PATH
        Command::new("where")
            .arg(binary)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        // `which` is POSIX standard and available on macOS and all major Linux distros
        Command::new("which")
            .arg(binary)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

/// Alternative binary names to try when the primary name is not found.
/// Covers real-world variations across distros and install methods.
fn alt_binaries(binary: &str) -> &'static [&'static str] {
    match binary {
        // Python: distros often ship `python3` not `python`
        "pip" => &["pip3", "pip3.12", "pip3.11", "pip3.10"],
        "python" => &["python3"],
        // Node: some Linux distros package as `nodejs`
        "node" => &["nodejs"],
        // Go: sometimes installed as `golang`
        "go" => &["golang"],
        // Ruby gem
        "gem" => &["gem3", "gem2"],
        // dart sometimes accessed via flutter
        "dart" => &["flutter"],
        // dotnet sometimes `dotnet-host`
        "dotnet" => &["dotnet-host"],
        _ => &[],
    }
}

// ── Ecosystem info ────────────────────────────────────────────────────────────

pub struct EcosystemInfo {
    pub install_url: &'static str,
    pub note: &'static str,
    /// Install command appropriate for the detected OS (determined at runtime).
    pub install_cmd: String,
}

/// Returns install instructions for a given package manager, OS-aware at runtime.
pub fn install_instructions(pm: &str) -> Option<EcosystemInfo> {
    let os = Os::detect();

    Some(match pm {
        "npm" => EcosystemInfo {
            install_url: "https://nodejs.org/en/download",
            note:        "npm ships bundled with Node.js — install Node.js to get npm.",
            install_cmd: os.cmd(
                "winget install OpenJS.NodeJS   OR   choco install nodejs",
                "brew install node              OR   nvm install --lts",
                "sudo apt install nodejs npm    OR   nvm install --lts",
            ),
        },
        "yarn" => EcosystemInfo {
            install_url: "https://yarnpkg.com/getting-started",
            note:        "Requires Node.js + npm installed first.",
            install_cmd: "npm install -g yarn   OR   corepack enable yarn".to_string(),
        },
        "pnpm" => EcosystemInfo {
            install_url: "https://pnpm.io/installation",
            note:        "Requires Node.js + npm installed first.",
            install_cmd: "npm install -g pnpm".to_string(),
        },
        "bun" => EcosystemInfo {
            install_url: "https://bun.sh",
            note:        "Bun is a fast all-in-one JS runtime & package manager.",
            install_cmd: os.cmd(
                "powershell -c \"irm bun.sh/install.ps1 | iex\"",
                "brew install bun   OR   curl -fsSL https://bun.sh/install | bash",
                "curl -fsSL https://bun.sh/install | bash",
            ),
        },
        "pip" | "pip3" => EcosystemInfo {
            install_url: "https://python.org/downloads",
            note:        "pip ships with Python 3.4+. Install Python to get pip.",
            install_cmd: os.cmd(
                "winget install Python.Python.3   OR   choco install python",
                "brew install python              OR   pyenv install 3.13.0",
                "sudo apt install python3-pip     OR   sudo dnf install python3-pip",
            ),
        },
        "uv" => EcosystemInfo {
            install_url: "https://github.com/astral-sh/uv",
            note:        "uv is an ultra-fast Python package manager by Astral.",
            install_cmd: os.cmd(
                "pip install uv   OR   winget install astral-sh.uv",
                "pip install uv   OR   brew install uv",
                "pip install uv   OR   curl -Ls https://astral.sh/uv/install.sh | sh",
            ),
        },
        "poetry" => EcosystemInfo {
            install_url: "https://python-poetry.org/docs/#installation",
            note:        "Requires Python + pip installed first.",
            install_cmd: "pip install poetry   OR   pipx install poetry".to_string(),
        },
        "cargo" => EcosystemInfo {
            install_url: "https://rustup.rs",
            note:        "cargo ships with Rust. Install via rustup.",
            install_cmd: os.cmd(
                "winget install Rustlang.Rustup   OR   download from https://rustup.rs",
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
            ),
        },
        "go" => EcosystemInfo {
            install_url: "https://go.dev/dl",
            note:        "The `go` tool ships with the Go SDK.",
            install_cmd: os.cmd(
                "winget install GoLang.Go   OR   choco install golang",
                "brew install go",
                "sudo apt install golang    OR   download from https://go.dev/dl",
            ),
        },
        "gem" => EcosystemInfo {
            install_url: "https://www.ruby-lang.org/en/downloads",
            note:        "gem ships with Ruby. Install Ruby to get gem.",
            install_cmd: os.cmd(
                "winget install RubyInstallerTeam.RubyWithDevKit",
                "brew install ruby",
                "sudo apt install ruby-full   OR   rbenv install 3.4.0",
            ),
        },
        "composer" => EcosystemInfo {
            install_url: "https://getcomposer.org/download",
            note:        "Requires PHP installed first.",
            install_cmd: os.cmd(
                "winget install Composer.Composer   (PHP required)",
                "brew install composer",
                "sudo apt install composer   OR   php -r \"copy('https://getcomposer.org/installer','cs.php');\" && php cs.php",
            ),
        },
        "nuget" | "dotnet" => EcosystemInfo {
            install_url: "https://learn.microsoft.com/en-us/dotnet/core/install",
            note:        "Use `dotnet add package` for NuGet packages in .NET projects.",
            install_cmd: os.cmd(
                "winget install Microsoft.DotNet.SDK.9",
                "brew install --cask dotnet",
                "sudo apt install dotnet-sdk-9.0   OR   see https://learn.microsoft.com/dotnet/core/install/linux",
            ),
        },
        "hex" | "mix" => EcosystemInfo {
            install_url: "https://elixir-lang.org/install.html",
            note:        "hex/mix ship with Elixir. Elixir requires Erlang/OTP.",
            install_cmd: os.cmd(
                "winget install Erlang.OTPwin64 && winget install ElixirLang.Elixir",
                "brew install elixir",
                "sudo apt install elixir   OR   asdf install elixir latest",
            ),
        },
        "pub" | "dart" => EcosystemInfo {
            install_url: "https://dart.dev/get-dart",
            note:        "pub ships with the Dart SDK. Flutter also includes Dart.",
            install_cmd: os.cmd(
                "winget install Dart.Dart   OR   winget install Google.FlutterSDK",
                "brew install dart",
                "sudo apt install dart   OR   see https://dart.dev/get-dart",
            ),
        },
        _ => return None,
    })
}

// ── OS abstraction ────────────────────────────────────────────────────────────

enum Os {
    Windows,
    MacOs,
    Linux,
    Unknown,
}

impl Os {
    fn detect() -> Self {
        #[cfg(target_os = "windows")]
        {
            return Os::Windows;
        }
        #[cfg(target_os = "macos")]
        {
            return Os::MacOs;
        }
        #[cfg(target_os = "linux")]
        {
            return Os::Linux;
        }
        #[allow(unreachable_code)]
        Os::Unknown
    }

    /// Pick the right string for the current OS. `unknown` falls back to linux.
    fn cmd(&self, win: &str, mac: &str, linux: &str) -> String {
        match self {
            Os::Windows => win.to_string(),
            Os::MacOs => mac.to_string(),
            Os::Linux | Os::Unknown => linux.to_string(),
        }
    }
}
