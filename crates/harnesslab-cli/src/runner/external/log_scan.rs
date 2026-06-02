use harnesslab_core::FailureCode;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

const DOCKER_NETWORK_POOL_EXHAUSTED: &str =
    "all predefined address pools have been fully subnetted";
const TERMINAL_BENCH_COMPOSE_BUILD_FAILED: &str = "Docker compose command failed with exit code";
const TERMINAL_BENCH_COMPOSE_CALLED_PROCESS_ERROR: &str = "Command '['docker', 'compose'";
const MAX_LOG_BYTES: u64 = 256 * 1024;

pub(super) fn terminal_bench_infra_failure(attempt_dir: &Path) -> Option<FailureCode> {
    find_log_files(attempt_dir).into_iter().find_map(|path| {
        let content = read_log_sample(&path).ok()?;
        if content.contains(DOCKER_NETWORK_POOL_EXHAUSTED) {
            return Some(FailureCode::DockerNetworkPoolExhausted);
        }
        if content.contains(TERMINAL_BENCH_COMPOSE_BUILD_FAILED)
            || (content.contains(TERMINAL_BENCH_COMPOSE_CALLED_PROCESS_ERROR)
                && content.contains("returned non-zero exit status"))
        {
            return Some(FailureCode::ExternalRunnerSetupFailed);
        }
        None
    })
}

fn find_log_files(root: &Path) -> Vec<PathBuf> {
    let mut logs = Vec::new();
    for relative in ["agent/stdout.log", "agent/stderr.log"] {
        let path = root.join(relative);
        if path.is_file() {
            logs.push(path);
        }
    }
    collect_official_run_logs(&root.join("official/terminal-bench"), &mut logs);
    logs
}

fn collect_official_run_logs(root: &Path, logs: &mut Vec<PathBuf>) {
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        if metadata.is_dir() {
            let Ok(entries) = fs::read_dir(&path) else {
                continue;
            };
            for entry in entries.flatten() {
                pending.push(entry.path());
            }
        } else if path.file_name().and_then(|name| name.to_str()) == Some("run.log") {
            logs.push(path);
        }
    }
}

fn read_log_sample(path: &Path) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    let len = file.metadata()?.len();
    if len > MAX_LOG_BYTES {
        file.seek(SeekFrom::Start(len - MAX_LOG_BYTES))?;
    }
    let mut bytes = Vec::new();
    file.take(MAX_LOG_BYTES).read_to_end(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_terminal_bench_docker_network_pool_exhaustion() {
        let tmp = tempfile::tempdir().unwrap();
        let log_dir = tmp.path().join("official/terminal-bench/run/task");
        fs::create_dir_all(&log_dir).unwrap();
        fs::write(
            log_dir.join("run.log"),
            "Error response from daemon: all predefined address pools have been fully subnetted",
        )
        .unwrap();

        assert_eq!(
            terminal_bench_infra_failure(tmp.path()),
            Some(FailureCode::DockerNetworkPoolExhausted)
        );
    }

    #[test]
    fn detects_terminal_bench_compose_setup_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let log_dir = tmp.path().join("official/terminal-bench/run/task");
        fs::create_dir_all(&log_dir).unwrap();
        fs::write(
            log_dir.join("run.log"),
            "Docker compose command failed with exit code 1\nfailed to solve: gcc: internal compiler error",
        )
        .unwrap();

        assert_eq!(
            terminal_bench_infra_failure(tmp.path()),
            Some(FailureCode::ExternalRunnerSetupFailed)
        );
    }

    #[test]
    fn ignores_non_log_files_and_clean_logs() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("result.json"),
            DOCKER_NETWORK_POOL_EXHAUSTED,
        )
        .unwrap();
        fs::write(tmp.path().join("run.log"), "clean").unwrap();

        assert_eq!(terminal_bench_infra_failure(tmp.path()), None);
    }

    #[test]
    fn scans_large_log_tail_for_late_docker_network_pool_exhaustion() {
        let tmp = tempfile::tempdir().unwrap();
        let mut content = "x".repeat((MAX_LOG_BYTES as usize) + 128);
        content.push_str(DOCKER_NETWORK_POOL_EXHAUSTED);
        let log_dir = tmp.path().join("official/terminal-bench/run/task");
        fs::create_dir_all(&log_dir).unwrap();
        fs::write(log_dir.join("run.log"), content).unwrap();

        assert_eq!(
            terminal_bench_infra_failure(tmp.path()),
            Some(FailureCode::DockerNetworkPoolExhausted)
        );
    }

    #[test]
    fn ignores_verifier_logs_with_docker_error_text() {
        let tmp = tempfile::tempdir().unwrap();
        let log_dir = tmp.path().join("verifier");
        fs::create_dir_all(&log_dir).unwrap();
        fs::write(
            log_dir.join("stderr.log"),
            "Docker compose command failed with exit code 1",
        )
        .unwrap();

        assert_eq!(terminal_bench_infra_failure(tmp.path()), None);
    }
}
