//! Shell 命令执行：模板渲染、权限校验和命令运行。

use super::types::ActionOutcome;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tokio::process::Command;

pub(super) fn validate_request_permissions(
    cwd: Option<&str>,
    env: &HashMap<String, String>,
    allowed_working_dirs: &[String],
    allowed_env_keys: Option<&HashSet<String>>,
) -> Option<String> {
    if let Some(cwd) = cwd {
        if !allowed_working_dirs.is_empty()
            && !allowed_working_dirs
                .iter()
                .any(|allowed| Path::new(cwd).starts_with(Path::new(allowed)))
        {
            return Some(format!("working directory is not allowed: {cwd}"));
        }
    }

    if let Some(allowed) = allowed_env_keys {
        for key in env.keys() {
            if !allowed.contains(key) {
                return Some(format!("environment key is not allowed: {key}"));
            }
        }
    }

    None
}

pub(super) async fn run_shell_command(
    command: String,
    cwd: Option<String>,
    env: HashMap<String, String>,
) -> ActionOutcome {
    let mut child = Command::new("sh");
    child.arg("-lc").arg(&command);
    if let Some(cwd) = cwd {
        child.current_dir(cwd);
    }
    if !env.is_empty() {
        child.envs(env);
    }

    match child.output().await {
        Ok(output) => ActionOutcome {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        },
        Err(err) => ActionOutcome::failure(err.to_string()),
    }
}

pub(super) fn render_command_template(
    template: &str,
    params: &HashMap<String, String>,
    host: &HashMap<String, String>,
) -> String {
    let mut rendered = String::new();
    let mut cursor = 0usize;

    while let Some(start) = template[cursor..].find("{{") {
        let start = cursor + start;
        rendered.push_str(&template[cursor..start]);
        let value_start = start + 2;

        if let Some(end_rel) = template[value_start..].find("}}") {
            let end = value_start + end_rel;
            let key = template[value_start..end].trim();
            let (raw, key) = if let Some(key) = key.strip_prefix("raw:") {
                (true, key.trim())
            } else {
                (false, key)
            };

            let value = key
                .strip_prefix("host.")
                .and_then(|key| host.get(key))
                .or_else(|| params.get(key));

            if let Some(value) = value {
                if raw {
                    rendered.push_str(value);
                } else {
                    rendered.push_str(&shell_escape(value));
                }
            }
            cursor = end + 2;
        } else {
            rendered.push_str(&template[start..]);
            cursor = template.len();
            break;
        }
    }

    if cursor < template.len() {
        rendered.push_str(&template[cursor..]);
    }

    rendered
}

pub(super) fn shell_escape(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }

    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}
