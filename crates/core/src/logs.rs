use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    Launcher,
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
    Unknown,
}

/// Classifies a game output line. `previous` is the level of the line before,
/// so stack traces stay attached to the error that produced them.
pub fn detect_level(line: &str, previous: LogLevel) -> LogLevel {
    if let Some(level) = bracketed_level(line) {
        return level;
    }
    let trimmed = line.trim_start();
    let is_continuation = line.starts_with('\t')
        || trimmed.starts_with("at ")
        || trimmed.starts_with("Caused by:")
        || trimmed.starts_with("... ");
    if is_continuation
        && matches!(
            previous,
            LogLevel::Error | LogLevel::Fatal | LogLevel::Warning
        )
    {
        return previous;
    }
    if trimmed.contains("Exception") || trimmed.contains("Error:") {
        return LogLevel::Error;
    }
    LogLevel::Unknown
}

/// Reads the level from `[12:00:00] [Render thread/WARN]: ...`.
fn bracketed_level(line: &str) -> Option<LogLevel> {
    let tag = line.split("]: ").next()?;
    let level = tag.rsplit('/').next()?.trim_end_matches(']');
    match level {
        "TRACE" | "DEBUG" => Some(LogLevel::Debug),
        "INFO" => Some(LogLevel::Info),
        "WARN" | "WARNING" => Some(LogLevel::Warning),
        "ERROR" | "SEVERE" => Some(LogLevel::Error),
        "FATAL" => Some(LogLevel::Fatal),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_bracketed_levels() {
        let line = "[12:41:05] [main/WARN]: Unable to parse version";
        assert_eq!(detect_level(line, LogLevel::Unknown), LogLevel::Warning);
        let line = "[12:41:08] [Download-2/ERROR]: Failed to fetch user properties";
        assert_eq!(detect_level(line, LogLevel::Unknown), LogLevel::Error);
    }

    #[test]
    fn stack_traces_inherit_the_error_level() {
        assert_eq!(
            detect_level("\tat com.mojang.Foo.bar(Foo.java:1)", LogLevel::Error),
            LogLevel::Error
        );
        assert_eq!(
            detect_level("Caused by: java.io.IOException", LogLevel::Error),
            LogLevel::Error
        );
        assert_eq!(detect_level("\tat x", LogLevel::Info), LogLevel::Unknown);
    }

    #[test]
    fn unstructured_exceptions_are_errors() {
        assert_eq!(
            detect_level("java.lang.NullPointerException: boom", LogLevel::Info),
            LogLevel::Error
        );
        assert_eq!(
            detect_level("plain output", LogLevel::Info),
            LogLevel::Unknown
        );
    }
}
