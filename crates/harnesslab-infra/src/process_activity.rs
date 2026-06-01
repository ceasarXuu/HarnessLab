use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct NoOutputActivityEvent {
    pub path: PathBuf,
    pub run_id: String,
    pub task_id: Option<String>,
    pub event_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessActivity {
    pub pid: i32,
    pub pattern: String,
    pub command_name: String,
}

pub fn process_group_activity(pgid: i32, patterns: &[String]) -> Result<Option<ProcessActivity>> {
    if pgid <= 0 || patterns.is_empty() {
        return Ok(None);
    }
    let output = Command::new("ps")
        .args(["-axo", "pid=,pgid=,stat=,command="])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().find_map(|line| {
        let process = parse_ps_line(line)?;
        if process.pgid != pgid || process.stat.contains('Z') {
            return None;
        }
        matching_pattern(process.command, patterns).map(|(pattern, command_name)| ProcessActivity {
            pid: process.pid,
            pattern,
            command_name,
        })
    }))
}

fn matching_pattern(command: &str, patterns: &[String]) -> Option<(String, String)> {
    patterns.iter().find_map(|pattern| {
        command_matches_activity_pattern(command, pattern).then(|| {
            (
                pattern.clone(),
                command_basename(command).unwrap_or_default(),
            )
        })
    })
}

fn command_matches_activity_pattern(command: &str, pattern: &str) -> bool {
    let mut command_tokens = command.split_whitespace();
    let Some(command_name) = command_tokens.next().and_then(token_basename) else {
        return false;
    };
    let pattern_tokens: Vec<&str> = pattern.split_whitespace().collect();
    match pattern_tokens.as_slice() {
        [single] => command_name == *single,
        [first, rest @ ..] => {
            command_name == *first
                && rest
                    .iter()
                    .all(|expected| command_tokens.next() == Some(*expected))
        }
        [] => false,
    }
}

struct PsProcess<'a> {
    pid: i32,
    pgid: i32,
    stat: &'a str,
    command: &'a str,
}

fn parse_ps_line(line: &str) -> Option<PsProcess<'_>> {
    let (pid, rest) = take_token(line)?;
    let (pgid, rest) = take_token(rest)?;
    let (stat, command) = take_token(rest)?;
    Some(PsProcess {
        pid: pid.parse().ok()?,
        pgid: pgid.parse().ok()?,
        stat,
        command: command.trim_start(),
    })
}

fn take_token(input: &str) -> Option<(&str, &str)> {
    let trimmed = input.trim_start();
    if trimmed.is_empty() {
        return None;
    }
    let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
    Some((&trimmed[..end], &trimmed[end..]))
}

fn command_basename(command: &str) -> Option<String> {
    command
        .split_whitespace()
        .next()
        .and_then(token_basename)
        .map(str::to_string)
}

fn token_basename(token: &str) -> Option<&str> {
    let clean = token.trim_matches(|ch| matches!(ch, '"' | '\''));
    clean.rsplit('/').next().filter(|name| !name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::command_matches_activity_pattern;
    use super::parse_ps_line;

    #[test]
    fn matches_docker_activity_on_executable_tokens() {
        assert!(command_matches_activity_pattern(
            "/opt/homebrew/bin/docker compose -p run build",
            "docker compose"
        ));
        assert!(command_matches_activity_pattern(
            "/opt/homebrew/bin/docker buildx bake",
            "docker buildx"
        ));
        assert!(command_matches_activity_pattern(
            "/opt/homebrew/lib/docker/cli-plugins/docker-buildx bake",
            "docker-buildx"
        ));
    }

    #[test]
    fn ignores_shell_text_that_mentions_docker_activity() {
        assert!(!command_matches_activity_pattern(
            "/bin/sh -c 'echo docker buildx; sleep 10'",
            "docker buildx"
        ));
    }

    #[test]
    fn parses_ps_lines_with_aligned_spacing() {
        let process = parse_ps_line("  12   34 Ss   /opt/bin/docker compose build").unwrap();

        assert_eq!(process.pid, 12);
        assert_eq!(process.pgid, 34);
        assert_eq!(process.stat, "Ss");
        assert_eq!(process.command, "/opt/bin/docker compose build");
    }
}
