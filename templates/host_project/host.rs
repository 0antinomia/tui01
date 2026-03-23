use tui01::host::{HostEvent, HostLogLevel, RuntimeHost, ShellPolicy};

use crate::actions;

pub fn build_host() -> RuntimeHost {
    let mut host = RuntimeHost::new()
        .set_context("project_root", ".")
        .set_working_dir(".")
        .allow_working_dir(".")
        .insert_env("APP_ENV", "dev")
        .allow_env_key("APP_ENV")
        .set_shell_policy(ShellPolicy::RegisteredOnly)
        .on_log(|record| match record.level {
            HostLogLevel::Debug => eprintln!("[debug] {}", record.message),
            HostLogLevel::Info => eprintln!("[info] {}", record.message),
            HostLogLevel::Warn => eprintln!("[warn] {}", record.message),
            HostLogLevel::Error => eprintln!("[error] {}", record.message),
        })
        .on_event(|event| match event {
            HostEvent::OperationStarted { source, .. } => eprintln!("started: {source}"),
            HostEvent::OperationFinished { source, success, .. } => {
                eprintln!("finished: {source} success={success}")
            }
        });

    actions::register_actions(&mut host);
    host
}
