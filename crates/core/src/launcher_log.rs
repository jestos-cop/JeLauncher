//! Writes the launcher's own log to a file, keeping the previous run's log next to it.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use log::{LevelFilter, Log, Metadata, Record};

use crate::{Error, Result};

pub const LOG_FILE: &str = "launcher.log";
const PREVIOUS_LOG_FILE: &str = "launcher.1.log";

struct FileLogger {
    file: Mutex<File>,
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target().starts_with("jelauncher")
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!(
            "{} {:<5} {}: {}\n",
            utc_timestamp(SystemTime::now()),
            record.level(),
            record.target(),
            record.args()
        );
        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(line.as_bytes());
        }
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock() {
            let _ = file.flush();
        }
    }
}

/// Starts logging into `dir` and returns the log file path.
pub fn init(dir: &Path, level: LevelFilter) -> Result<PathBuf> {
    std::fs::create_dir_all(dir).map_err(|err| Error::io(dir, err))?;
    let path = dir.join(LOG_FILE);
    if path.exists() {
        let previous = dir.join(PREVIOUS_LOG_FILE);
        std::fs::rename(&path, &previous).map_err(|err| Error::io(&path, err))?;
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|err| Error::io(&path, err))?;
    let logger = FileLogger {
        file: Mutex::new(file),
    };
    log::set_boxed_logger(Box::new(logger))
        .map_err(|err| Error::InvalidInput(format!("logging is already set up: {err}")))?;
    log::set_max_level(level);
    Ok(path)
}

/// ISO 8601 UTC time, e.g. `2025-10-19T15:00:00Z`.
pub(crate) fn utc_timestamp(time: SystemTime) -> String {
    let secs = time
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let (days, rest) = (secs / 86_400, secs % 86_400);
    let (year, month, day) = civil_from_days(i64::try_from(days).unwrap_or(0));
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

/// Gregorian date of a day count since 1970-01-01 (Howard Hinnant's algorithm).
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn formats_utc_timestamps() {
        assert_eq!(utc_timestamp(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        let time = UNIX_EPOCH + Duration::from_secs(1_760_886_000);
        assert_eq!(utc_timestamp(time), "2025-10-19T15:00:00Z");
        let leap_day = UNIX_EPOCH + Duration::from_secs(1_709_164_800);
        assert_eq!(utc_timestamp(leap_day), "2024-02-29T00:00:00Z");
    }
}
