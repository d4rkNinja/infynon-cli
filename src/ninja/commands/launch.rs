fn append_forwarded_args(command: &str, args: &[String]) -> String {
    if args.is_empty() {
        return command.to_string();
    }
    let rendered = args
        .iter()
        .map(|arg| shell_quote(arg))
        .collect::<Vec<_>>()
        .join(" ");
    format!("{} {}", command, rendered)
}

fn shell_quote(value: &str) -> String {
    if cfg!(windows) {
        format!("'{}'", value.replace('\'', "''"))
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn launch_agent_command(
    command: &str,
    cwd: &Option<String>,
    background: bool,
) -> Result<serde_json::Value, String> {
    if background {
        launch_background(command, cwd)
    } else {
        launch_new_terminal(command, cwd)
    }
}

fn schedule_close_invoking_terminal() -> serde_json::Value {
    let current_pid = std::process::id();
    #[cfg(windows)]
    {
        let script = format!(
            "$proc = Get-CimInstance Win32_Process -Filter 'ProcessId = {current_pid}'; \
             if ($proc -and $proc.ParentProcessId) {{ \
               Start-Sleep -Milliseconds 700; \
               taskkill /PID $proc.ParentProcessId /T /F | Out-Null \
             }}; Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue"
        );
        let spawned = spawn_detached_windows_powershell(&script);
        match spawned {
            Ok(child) => json!({
                "scheduled": true,
                "current_pid": current_pid,
                "closer_pid": child.id(),
                "method": "taskkill parent shell tree",
            }),
            Err(err) => json!({
                "scheduled": false,
                "current_pid": current_pid,
                "error": err.to_string(),
            }),
        }
    }

    #[cfg(not(windows))]
    {
        let script = format!(
            "ppid=$(ps -o ppid= -p {current_pid} | tr -d ' '); \
             if [ -n \"$ppid\" ] && [ \"$ppid\" != \"1\" ]; then sleep 0.7; kill -TERM \"$ppid\" 2>/dev/null; fi"
        );
        let spawned = Command::new("sh").args(["-lc", &script]).spawn();
        match spawned {
            Ok(child) => json!({
                "scheduled": true,
                "current_pid": current_pid,
                "closer_pid": child.id(),
                "method": "terminate parent shell process",
            }),
            Err(err) => json!({
                "scheduled": false,
                "current_pid": current_pid,
                "error": err.to_string(),
            }),
        }
    }
}

#[cfg(windows)]
fn spawn_detached_windows_powershell(script: &str) -> Result<std::process::Child, std::io::Error> {
    let stamp = timestamp_now()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>();
    let path = std::env::temp_dir().join(format!(
        "infynon-close-{}-{}.ps1",
        std::process::id(),
        stamp
    ));
    std::fs::write(&path, script)?;
    let launcher = format!(
        "Start-Process -WindowStyle Hidden -FilePath powershell -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File',{})",
        shell_quote(&path.display().to_string())
    );
    Command::new("powershell")
        .args([
            "-NoProfile",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &launcher,
        ])
        .spawn()
}

fn write_windows_agent_script(command: &str, cwd: &str) -> Result<std::path::PathBuf, String> {
    let stamp = timestamp_now()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>();
    let path = std::env::temp_dir().join(format!(
        "infynon-agent-{}-{}.ps1",
        std::process::id(),
        stamp
    ));
    let script = format!(
        "Set-Location -LiteralPath {}\r\ntry {{\r\n{}\r\n}} finally {{\r\n  Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue\r\n}}\r\n",
        shell_quote(cwd),
        command
    );
    std::fs::write(&path, script)
        .map(|_| path)
        .map_err(|e| format!("Failed to write Windows agent launcher script: {}", e))
}

fn write_unix_agent_script(command: &str, cwd: &Option<String>) -> Result<std::path::PathBuf, String> {
    let stamp = timestamp_now()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>();
    let path = std::env::temp_dir().join(format!(
        "infynon-agent-{}-{}.sh",
        std::process::id(),
        stamp
    ));
    let mut script = String::from("#!/bin/sh\n");
    if let Some(cwd) = cwd {
        script.push_str(&format!("cd {} || exit 1\n", shell_quote(cwd)));
    }
    script.push_str(command);
    script.push_str("\nstatus=$?\nrm -- \"$0\" 2>/dev/null\nexec \"${SHELL:-/bin/sh}\" -l\nexit \"$status\"\n");
    std::fs::write(&path, script)
        .map_err(|e| format!("Failed to write agent launcher script: {}", e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&path)
            .map_err(|e| format!("Failed to inspect agent launcher script: {}", e))?
            .permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&path, permissions)
            .map_err(|e| format!("Failed to mark agent launcher script executable: {}", e))?;
    }
    Ok(path)
}

fn launch_background(command: &str, cwd: &Option<String>) -> Result<serde_json::Value, String> {
    let mut process = if cfg!(windows) {
        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-Command", command]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-lc", command]);
        cmd
    };
    if let Some(cwd) = cwd {
        process.current_dir(cwd);
    }
    let child = process
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start background agent command: {}", e))?;
    Ok(json!({
        "ran": true,
        "background": true,
        "terminal_opened": false,
        "pid": child.id(),
    }))
}

fn launch_new_terminal(command: &str, cwd: &Option<String>) -> Result<serde_json::Value, String> {
    if cfg!(windows) {
        launch_windows_terminal(command, cwd)
    } else if cfg!(target_os = "macos") {
        launch_macos_terminal(command, cwd)
    } else {
        launch_linux_terminal(command, cwd)
    }
}

fn launch_windows_terminal(command: &str, cwd: &Option<String>) -> Result<serde_json::Value, String> {
    let cwd = cwd
        .clone()
        .or_else(|| std::env::current_dir().ok().map(|p| p.display().to_string()))
        .unwrap_or_else(|| ".".to_string());
    let script_path = write_windows_agent_script(command, &cwd)?;
    let script = format!(
        "$p = Start-Process powershell -WorkingDirectory {} -ArgumentList @('-NoExit','-NoProfile','-ExecutionPolicy','Bypass','-File',{}) -PassThru; $p.Id",
        shell_quote(&cwd),
        shell_quote(&script_path.display().to_string())
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .map_err(|e| format!("Failed to open Windows terminal: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(json!({
        "ran": true,
        "background": false,
        "terminal_opened": true,
        "terminal": "powershell",
        "pid": pid,
        "script_path": script_path,
    }))
}

fn launch_macos_terminal(command: &str, cwd: &Option<String>) -> Result<serde_json::Value, String> {
    let script_path = write_unix_agent_script(command, cwd)?;
    let script_command = format!("sh {}", shell_quote(&script_path.display().to_string()));
    let escaped = script_command.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("tell application \"Terminal\" to do script \"{}\"", escaped);
    Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .map_err(|e| format!("Failed to open macOS Terminal: {}", e))?;
    Ok(json!({
        "ran": true,
        "background": false,
        "terminal_opened": true,
        "terminal": "Terminal",
        "script_path": script_path,
    }))
}

fn launch_linux_terminal(command: &str, cwd: &Option<String>) -> Result<serde_json::Value, String> {
    let script_path = write_unix_agent_script(command, cwd)?;
    let command_text = format!("sh {}", shell_quote(&script_path.display().to_string()));
    let candidates: [(&str, Vec<&str>); 5] = [
        ("x-terminal-emulator", vec!["-e", "sh", "-lc"]),
        ("gnome-terminal", vec!["--", "sh", "-lc"]),
        ("konsole", vec!["-e", "sh", "-lc"]),
        ("xfce4-terminal", vec!["-e", "sh", "-lc"]),
        ("xterm", vec!["-e", "sh", "-lc"]),
    ];
    for (terminal, args) in candidates {
        let mut cmd = Command::new(terminal);
        cmd.args(args).arg(&command_text);
        if let Ok(child) = cmd.spawn() {
            return Ok(json!({
                "ran": true,
                "background": false,
                "terminal_opened": true,
                "terminal": terminal,
                "pid": child.id(),
                "script_path": script_path,
            }));
        }
    }
    Err("No supported Linux terminal was found for foreground launch.".to_string())
}
