use anyhow::{Context, Result, anyhow};
use dialoguer::{Confirm, Input, Password, Select};
use std::ffi::OsString;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use zeroize::Zeroizing;

pub(crate) fn run() -> Result<()> {
    anyhow::ensure!(
        std::io::stdin().is_terminal()
            && std::io::stdout().is_terminal()
            && std::io::stderr().is_terminal(),
        "the interactive interface requires a terminal; run `hig --help` for non-interactive commands"
    );

    let categories = [
        "Archive",
        "Project snapshots",
        "Repository history",
        "Recovery Vault",
        "Cache maintenance",
        "Runtime and sessions",
        "Exit",
    ];

    loop {
        let Some(selection) = select("HIG", &categories)? else {
            break;
        };
        match selection {
            0 => archive_menu()?,
            1 => project_menu()?,
            2 => repository_menu()?,
            3 => recovery_menu()?,
            4 => cache_menu()?,
            5 => runtime_menu()?,
            _ => break,
        }
    }
    Ok(())
}

fn archive_menu() -> Result<()> {
    let actions = [
        "Create an archive",
        "Extract an archive",
        "Inspect an archive",
        "Back",
    ];
    loop {
        let Some(selection) = select("Archive", &actions)? else {
            return Ok(());
        };
        match selection {
            0 => create_archive()?,
            1 => extract_archive()?,
            2 => inspect_archive()?,
            _ => return Ok(()),
        }
    }
}

fn project_menu() -> Result<()> {
    let actions = [
        "Initialize project metadata",
        "Show project status",
        "Rebuild project snapshot",
        "Back",
    ];
    loop {
        let Some(selection) = select("Project snapshots", &actions)? else {
            return Ok(());
        };
        match selection {
            0 => {
                let dir = prompt_path("Project directory", current_dir_text())?;
                invoke(args_with_path(&["init"], dir), None)?;
            }
            1 => {
                let dir = prompt_path("Project directory", current_dir_text())?;
                invoke(args_with_path(&["project", "status"], dir), None)?;
            }
            2 => {
                let dir = prompt_path("Project directory", current_dir_text())?;
                let mut args = args_with_path(&["project", "rebuild"], dir);
                args.push("--wait".into());
                invoke(args, None)?;
            }
            _ => return Ok(()),
        }
    }
}

fn repository_menu() -> Result<()> {
    let actions = [
        "Initialize repository history",
        "Record a snapshot",
        "Show recent history",
        "Watch for automatic snapshots",
        "Verify repository",
        "Restore a revision",
        "Garbage collection preview",
        "Back",
    ];
    loop {
        let Some(selection) = select("Repository history", &actions)? else {
            return Ok(());
        };
        match selection {
            0 => {
                let dir = prompt_path("Project directory", current_dir_text())?;
                invoke(args_with_path(&["repo", "init"], dir), None)?;
            }
            1 => {
                let dir = prompt_path("Project directory", current_dir_text())?;
                let message = prompt_text("Snapshot message", "snapshot")?;
                let mut args = args_with_path(&["repo", "snapshot"], dir);
                args.extend(["--message".into(), message.into()]);
                invoke(args, None)?;
            }
            2 => {
                let dir = prompt_path("Project directory", current_dir_text())?;
                invoke(args_with_path(&["repo", "log"], dir), None)?;
            }
            3 => {
                let dir = prompt_path("Project directory", current_dir_text())?;
                let mut args = args_with_path(&["repo", "watch"], dir);
                args.extend(["--debounce-ms".into(), "750".into()]);
                invoke(args, None)?;
            }
            4 => {
                let dir = prompt_path("Project directory", current_dir_text())?;
                invoke(args_with_path(&["repo", "verify"], dir), None)?;
            }
            5 => restore_repository_revision()?,
            6 => {
                let dir = prompt_path("Project directory", current_dir_text())?;
                invoke(args_with_path(&["repo", "gc"], dir), None)?;
            }
            _ => return Ok(()),
        }
    }
}

fn recovery_menu() -> Result<()> {
    let actions = [
        "Initialize Recovery Vault",
        "Register a workspace",
        "Capture a recovery point",
        "Show Vault status",
        "List recovery points",
        "Verify a recovery point",
        "Restore a recovery point",
        "Garbage collection preview",
        "Back",
    ];
    loop {
        let Some(selection) = select("Recovery Vault", &actions)? else {
            return Ok(());
        };
        match selection {
            0 => initialize_recovery_vault()?,
            1 => {
                let dir = prompt_path("Project directory", current_dir_text())?;
                let mut args = args_with_path(&["recovery", "register"], dir);
                args.extend(prompt_vault_root_args()?);
                invoke(args, None)?;
            }
            2 => {
                let dir = prompt_path("Project directory", current_dir_text())?;
                let mut args = args_with_path(&["recovery", "capture"], dir);
                args.extend(prompt_vault_root_args()?);
                invoke(args, None)?;
            }
            3 => {
                let mut args = args(&["recovery", "status"]);
                args.extend(prompt_vault_root_args()?);
                invoke(args, None)?;
            }
            4 => {
                let mut args = args(&["recovery", "list"]);
                args.extend(prompt_vault_root_args()?);
                invoke(args, None)?;
            }
            5 => verify_recovery_point()?,
            6 => restore_recovery_point()?,
            7 => {
                let mut args = args(&["recovery", "gc"]);
                args.extend(prompt_vault_root_args()?);
                invoke(args, None)?;
            }
            _ => return Ok(()),
        }
    }
}

fn cache_menu() -> Result<()> {
    let actions = [
        "Show cache status",
        "Garbage collection preview",
        "Compaction preview",
        "Back",
    ];
    loop {
        let Some(selection) = select("Cache maintenance", &actions)? else {
            return Ok(());
        };
        let args = match selection {
            0 => args(&["cache", "status"]),
            1 => args(&["cache", "gc", "--dry-run"]),
            2 => args(&["cache", "compact", "--dry-run"]),
            _ => return Ok(()),
        };
        invoke(args, None)?;
    }
}

fn runtime_menu() -> Result<()> {
    let actions = [
        "Start daemon",
        "Show daemon status",
        "Stop daemon",
        "Unlock secure session",
        "Show session status",
        "Clear session",
        "Back",
    ];
    loop {
        let Some(selection) = select("Runtime and sessions", &actions)? else {
            return Ok(());
        };
        match selection {
            0 => invoke(args(&["daemon", "start"]), None)?,
            1 => invoke(args(&["daemon", "status"]), None)?,
            2 => {
                if Confirm::new()
                    .with_prompt("Stop the daemon? Running tasks may be interrupted.")
                    .default(false)
                    .interact()?
                {
                    invoke(args(&["daemon", "stop"]), None)?;
                }
            }
            3 => {
                let mut command = args(&["session", "unlock", "--password-stdin"]);
                command.extend(prompt_cache_dir_args()?);
                invoke(command, Some(prompt_password(false)?))?;
            }
            4 => {
                let mut command = args(&["session", "status"]);
                command.extend(prompt_cache_dir_args()?);
                invoke(command, None)?;
            }
            5 => {
                if Confirm::new()
                    .with_prompt("Clear the in-memory session key?")
                    .default(false)
                    .interact()?
                {
                    let mut command = args(&["session", "clear"]);
                    command.extend(prompt_cache_dir_args()?);
                    invoke(command, None)?;
                }
            }
            _ => return Ok(()),
        }
    }
}

fn create_archive() -> Result<()> {
    let input_dir = prompt_path("Directory to archive", current_dir_text())?;
    let output_default = archive_output_default(&input_dir);
    let output = prompt_path("Output archive", output_default)?;
    let encryption = select(
        "Encryption",
        &[
            "Password-protected (recommended)",
            "No encryption (archive contents will be readable)",
        ],
    )?
    .ok_or_else(|| anyhow!("archive creation was cancelled"))?;

    let mut command = args(&["pack"]);
    command.push(input_dir.into_os_string());
    command.extend(["--output".into(), output.into_os_string()]);
    let password = if encryption == 0 {
        command.extend([
            "--encryption".into(),
            "password".into(),
            "--password-stdin".into(),
        ]);
        Some(prompt_password(true)?)
    } else {
        command.extend(["--encryption".into(), "none".into()]);
        None
    };
    invoke(command, password)
}

fn extract_archive() -> Result<()> {
    let archive = prompt_path("Archive to extract", "archive.hig")?;
    let output_dir = prompt_path("Destination directory", "restored")?;
    let mut command = args(&["unpack"]);
    command.push(archive.into_os_string());
    command.extend(["--output-dir".into(), output_dir.into_os_string()]);

    let password = if Confirm::new()
        .with_prompt("Is this archive password-protected?")
        .default(true)
        .interact()?
    {
        command.push("--password-stdin".into());
        Some(prompt_password(false)?)
    } else {
        None
    };
    if Confirm::new()
        .with_prompt("Allow overwriting existing files?")
        .default(false)
        .interact()?
    {
        command.push("--overwrite".into());
    }
    invoke(command, password)
}

fn inspect_archive() -> Result<()> {
    let archive = prompt_path("Archive to inspect", "archive.hig")?;
    let mut command = args(&["inspect"]);
    command.push(archive.into_os_string());
    let password = if Confirm::new()
        .with_prompt("Is this archive password-protected?")
        .default(false)
        .interact()?
    {
        command.push("--password-stdin".into());
        Some(prompt_password(false)?)
    } else {
        None
    };
    invoke(command, password)
}

fn restore_repository_revision() -> Result<()> {
    let dir = prompt_path("Repository directory", current_dir_text())?;
    let revision = prompt_text("Revision", "HEAD")?;
    let output_dir = prompt_path("Restore destination", "restored")?;
    let mut command = args_with_path(&["repo", "restore"], dir);
    command.extend([
        "--revision".into(),
        revision.into(),
        "--output-dir".into(),
        output_dir.into_os_string(),
    ]);
    if Confirm::new()
        .with_prompt("Allow overwriting existing files?")
        .default(false)
        .interact()?
    {
        command.push("--overwrite".into());
    }
    invoke(command, None)
}

fn initialize_recovery_vault() -> Result<()> {
    let mut command = args(&["recovery", "init"]);
    command.extend(prompt_vault_root_args()?);
    if Confirm::new()
        .with_prompt("Configure a mirror now?")
        .default(false)
        .interact()?
    {
        let mirror = prompt_path("Mirror directory", "hig-recovery-mirror")?;
        command.extend(["--mirror".into(), mirror.into_os_string()]);
    }
    invoke(command, None)
}

fn verify_recovery_point() -> Result<()> {
    let repository_id = prompt_required_text("Repository ID")?;
    let point_id = prompt_required_text("Recovery point ID")?;
    let mut command = args(&["recovery", "verify"]);
    command.extend([repository_id.into(), point_id.into()]);
    command.extend(prompt_vault_root_args()?);
    invoke(command, None)
}

fn restore_recovery_point() -> Result<()> {
    let repository_id = prompt_required_text("Repository ID")?;
    let point_id = prompt_required_text("Recovery point ID")?;
    let output_dir = prompt_path("Restore destination", "recovered")?;
    let mut command = args(&["recovery", "restore"]);
    command.extend([
        repository_id.into(),
        point_id.into(),
        "--output-dir".into(),
        output_dir.into_os_string(),
    ]);
    command.extend(prompt_vault_root_args()?);
    if Confirm::new()
        .with_prompt("Allow overwriting existing files?")
        .default(false)
        .interact()?
    {
        command.push("--overwrite".into());
    }
    invoke(command, None)
}

fn prompt_vault_root_args() -> Result<Vec<OsString>> {
    let default = std::env::var("HIG_RECOVERY_VAULT").unwrap_or_default();
    let value = Input::<String>::new()
        .with_prompt("Vault root (blank uses the configured default)")
        .default(default)
        .allow_empty(true)
        .interact_text()?;
    if value.trim().is_empty() {
        Ok(Vec::new())
    } else {
        Ok(vec!["--vault-root".into(), value.into()])
    }
}

fn prompt_cache_dir_args() -> Result<Vec<OsString>> {
    let value = Input::<String>::new()
        .with_prompt("Cache directory (blank uses the default)")
        .allow_empty(true)
        .interact_text()?;
    if value.trim().is_empty() {
        Ok(Vec::new())
    } else {
        Ok(vec!["--cache-dir".into(), value.into()])
    }
}

fn prompt_password(confirm: bool) -> Result<Zeroizing<String>> {
    let prompt = Password::new().with_prompt("Password");
    let value = if confirm {
        prompt
            .with_confirmation("Confirm password", "Passwords do not match")
            .interact()?
    } else {
        prompt.interact()?
    };
    anyhow::ensure!(!value.is_empty(), "password must not be empty");
    Ok(Zeroizing::new(value))
}

fn prompt_path(label: &str, default: impl Into<String>) -> Result<PathBuf> {
    let value = Input::<String>::new()
        .with_prompt(label)
        .default(default.into())
        .interact_text()?;
    Ok(PathBuf::from(value))
}

fn prompt_text(label: &str, default: impl Into<String>) -> Result<String> {
    Input::<String>::new()
        .with_prompt(label)
        .default(default.into())
        .interact_text()
        .map_err(Into::into)
}

fn prompt_required_text(label: &str) -> Result<String> {
    Input::<String>::new()
        .with_prompt(label)
        .validate_with(|value: &String| {
            if value.trim().is_empty() {
                Err("This value is required".to_string())
            } else {
                Ok(())
            }
        })
        .interact_text()
        .map_err(Into::into)
}

fn select(prompt: &str, items: &[&str]) -> Result<Option<usize>> {
    Select::new()
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact_opt()
        .map_err(Into::into)
}

fn invoke(args: Vec<OsString>, secret: Option<Zeroizing<String>>) -> Result<()> {
    let executable = std::env::current_exe().context("resolve current HIG executable")?;
    let mut command = Command::new(executable);
    command.args(args);
    if secret.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut child = command.spawn().context("start HIG command")?;

    if let Some(secret) = secret {
        let mut child_stdin = child
            .stdin
            .take()
            .context("open child stdin for protected password input")?;
        let result = child_stdin
            .write_all(secret.as_bytes())
            .and_then(|()| child_stdin.write_all(b"\n"));
        drop(child_stdin);
        drop(secret);
        if let Err(error) = result {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error).context("send protected password to HIG command");
        }
    }

    let status = child.wait().context("wait for HIG command")?;
    if status.success() {
        eprintln!("\nHIG command completed.");
    } else {
        eprintln!("\nHIG command exited with {status}.");
    }
    Ok(())
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

fn args_with_path(values: &[&str], path: PathBuf) -> Vec<OsString> {
    let mut result = args(values);
    result.push(path.into_os_string());
    result
}

fn current_dir_text() -> String {
    std::env::current_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|_| ".".to_string())
}

fn archive_output_default(input: &Path) -> String {
    let name = input
        .file_name()
        .map(|value| value.to_string_lossy())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "archive".into());
    input
        .with_file_name(format!("{name}.hig"))
        .to_string_lossy()
        .into_owned()
}
