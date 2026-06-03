use harnesslab_infra::{DockerCliProvider, ExecSpec, SandboxHandle};
use std::collections::BTreeMap;
use std::fs;
use std::process::Command;

#[cfg(unix)]
#[test]
fn c_sbox_010_docker_exec_preserves_client_env_without_agent_env_leak() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    let capture = tmp.path().join("capture.txt");
    let docker = bin.join("docker");
    fs::write(
        &docker,
        format!(
            r#"#!/bin/sh
set -eu
{{
  printf 'args=%s\n' "$*"
  printf 'DOCKER_HOST=%s\n' "${{DOCKER_HOST:-}}"
  printf 'HARNESSLAB_AGENT_ONLY=%s\n' "${{HARNESSLAB_AGENT_ONLY:-}}"
}} > {}
test "${{DOCKER_HOST:-}}" = "tcp://docker.example.test:2376"
if [ "${{HARNESSLAB_AGENT_ONLY+x}}" = "x" ]; then
  exit 65
fi
"#,
            shell_quote(&capture.display().to_string())
        ),
    )
    .unwrap();
    make_executable(&docker);

    let mut paths = vec![bin.clone()];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let path = std::env::join_paths(paths).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .arg("docker_exec_public_wrapper_helper")
        .arg("--nocapture")
        .env("HARNESSLAB_RUN_DOCKER_EXEC_HELPER", "1")
        .env("DOCKER_HOST", "tcp://docker.example.test:2376")
        .env("HARNESSLAB_DOCKER_CAPTURE", &capture)
        .env("PATH", path)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "helper failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let captured = fs::read_to_string(capture).unwrap();
    assert!(captured.contains("args=exec -i --workdir /workspace container-1 sh -lc true"));
    assert!(captured.contains("DOCKER_HOST=tcp://docker.example.test:2376"));
    assert!(captured.contains("HARNESSLAB_AGENT_ONLY=\n"));
}

#[cfg(unix)]
#[test]
fn docker_exec_public_wrapper_helper() {
    if std::env::var("HARNESSLAB_RUN_DOCKER_EXEC_HELPER").as_deref() != Ok("1") {
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let mut env_vars = BTreeMap::new();
    env_vars.insert("HARNESSLAB_AGENT_ONLY".to_string(), "hidden".to_string());
    let handle = SandboxHandle {
        container_id: "container-1".to_string(),
        name: "name".to_string(),
        run_id: "run-1".to_string(),
        workspace_container_path: "/workspace".to_string(),
    };
    let result = DockerCliProvider::exec(
        &handle,
        &ExecSpec {
            command: "true".to_string(),
            stdin: None,
            working_dir: tmp.path().join("host"),
            timeout_sec: 5,
            no_output_timeout_sec: None,
            no_output_progress_paths: Vec::new(),
            no_output_activity_patterns: Vec::new(),
            no_output_activity_event: None,
            env_clear: true,
            env_vars,
            stdout_path: tmp.path().join("stdout.log"),
            stderr_path: tmp.path().join("stderr.log"),
        },
    )
    .unwrap();

    assert_eq!(result.exit_code, Some(0));
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(unix)]
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
