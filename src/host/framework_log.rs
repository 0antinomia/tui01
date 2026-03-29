//! 框架内部运行日志记录器。

use super::host_types::{HostLogLevel, HostLogRecord};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub enum FrameworkLogger {
    Enabled { path: PathBuf, file: Arc<Mutex<File>> },
    Disabled,
}

impl FrameworkLogger {
    pub fn new(base_dir: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = base_dir.as_ref().join(".tui01").join("logs").join("framework.log");
        Self::from_path(path)
    }

    pub fn from_path(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let path = path.into();
        let parent =
            path.parent().ok_or_else(|| std::io::Error::other("invalid framework log path"))?;
        fs::create_dir_all(parent)?;
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self::Enabled { path, file: Arc::new(Mutex::new(file)) })
    }

    pub fn fallback() -> Self {
        let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new(base).unwrap_or_else(|err| {
            panic!("failed to initialize framework logger: {err}");
        })
    }

    pub fn disabled() -> Self {
        Self::Disabled
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::Enabled { path, .. } => path.as_path(),
            Self::Disabled => Path::new(""),
        }
    }

    pub fn log(&self, record: &HostLogRecord) {
        let Self::Enabled { file, .. } = self else {
            return;
        };
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let line = format!(
            "{}.{:03} {:<5} {} {}\n",
            timestamp.as_secs(),
            timestamp.subsec_millis(),
            level_name(record.level),
            record.target,
            record.message.replace('\n', " | ")
        );

        if let Ok(mut file) = file.lock() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }
}

fn level_name(level: HostLogLevel) -> &'static str {
    match level {
        HostLogLevel::Debug => "DEBUG",
        HostLogLevel::Info => "INFO",
        HostLogLevel::Warn => "WARN",
        HostLogLevel::Error => "ERROR",
    }
}

#[cfg(test)]
mod tests {
    use super::FrameworkLogger;
    use crate::host::host_types::{HostLogLevel, HostLogRecord};
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    fn framework_logger_writes_to_file() {
        let base = std::env::temp_dir().join(format!("tui01-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let logger = FrameworkLogger::new(&base).unwrap();

        logger.log(&HostLogRecord {
            level: HostLogLevel::Info,
            target: "tui01.test".to_string(),
            message: "hello".to_string(),
        });

        let content = fs::read_to_string(PathBuf::from(logger.path())).unwrap();
        assert!(content.contains("INFO"));
        assert!(content.contains("tui01.test"));
        assert!(content.contains("hello"));

        let _ = fs::remove_dir_all(base.join(".tui01"));
    }

    #[test]
    fn disabled_framework_logger_writes_nothing() {
        let logger = FrameworkLogger::disabled();
        logger.log(&HostLogRecord {
            level: HostLogLevel::Info,
            target: "tui01.test".to_string(),
            message: "ignored".to_string(),
        });

        assert_eq!(logger.path(), Path::new(""));
    }
}
