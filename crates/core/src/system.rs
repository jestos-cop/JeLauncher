use tokio::process::Command;

/// Physical memory of this computer in MiB, if the platform reports it.
pub async fn total_memory_mb() -> Option<u64> {
    total_memory_bytes()
        .await
        .map(|bytes| bytes / (1024 * 1024))
}

#[cfg(target_os = "macos")]
async fn total_memory_bytes() -> Option<u64> {
    let output = Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .await
        .ok()?;
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

#[cfg(target_os = "linux")]
async fn total_memory_bytes() -> Option<u64> {
    let meminfo = tokio::fs::read_to_string("/proc/meminfo").await.ok()?;
    parse_meminfo(&meminfo)
}

#[cfg(windows)]
async fn total_memory_bytes() -> Option<u64> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory",
        ])
        .output()
        .await
        .ok()?;
    String::from_utf8_lossy(&output.stdout).trim().parse().ok()
}

#[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
async fn total_memory_bytes() -> Option<u64> {
    None
}

/// Reads `MemTotal` (in KiB) from `/proc/meminfo`.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn parse_meminfo(meminfo: &str) -> Option<u64> {
    meminfo
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))?
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()
        .map(|kib| kib * 1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_meminfo() {
        let meminfo = "MemTotal:       16303720 kB\nMemFree:         1234 kB\n";
        assert_eq!(parse_meminfo(meminfo), Some(16_303_720 * 1024));
    }

    #[tokio::test]
    async fn reports_memory_on_this_machine() {
        if cfg!(any(target_os = "macos", target_os = "linux")) {
            assert!(total_memory_mb().await.is_some_and(|mb| mb > 256));
        }
    }
}
