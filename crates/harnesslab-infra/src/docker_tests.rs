use super::*;
use harnesslab_core::NetworkPolicy;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;

#[test]
fn c_sbox_001_health_check_is_structured() {
    let result = DockerCliProvider::health_check();

    assert!(matches!(result.status.as_str(), "ok" | "error"));
    assert!(!result.message.is_empty());
}

#[test]
fn c_sbox_004_create_args_include_labels_mounts_and_network_policy() {
    let request = DockerCreateRequest {
        run_id: "run-1".to_string(),
        task_id: "task-1".to_string(),
        attempt: 2,
        image: "alpine:3.20".to_string(),
        workspace_host_path: PathBuf::from("/tmp/harnesslab-workspace"),
        workspace_container_path: "/workspace".to_string(),
        network: NetworkPolicy::None,
        env_vars: vec!["A=B".to_string()],
        mounts: vec!["/host/cache:/cache:ro".to_string()],
        privileged: false,
        cpu_cores: 2,
        memory_mb: 512,
    };

    let args = DockerCliProvider::create_args(&request);

    assert!(args.windows(2).any(|pair| pair == ["--network", "none"]));
    assert!(args.contains(&"--label".to_string()));
    assert!(args.contains(&"harnesslab.run_id=run-1".to_string()));
    assert!(args.contains(&"harnesslab.task_id=task-1".to_string()));
    assert!(args.contains(&"harnesslab.attempt=2".to_string()));
    assert!(args.contains(&"/tmp/harnesslab-workspace:/workspace".to_string()));
    assert!(args.contains(&"/host/cache:/cache:ro".to_string()));
    assert!(args.contains(&"A=B".to_string()));
    assert!(args.contains(&"--cpus".to_string()));
    assert!(args.contains(&"--memory".to_string()));
}

#[test]
fn c_sbox_005_exec_copy_destroy_and_cleanup_args_are_stable() {
    let handle = SandboxHandle {
        container_id: "abc123".to_string(),
        name: "harnesslab-run-task-1-1".to_string(),
        run_id: "run-1".to_string(),
        workspace_container_path: "/workspace".to_string(),
    };

    assert_eq!(
        DockerCliProvider::exec_args(&handle, "printf ok"),
        vec![
            "exec",
            "-i",
            "--workdir",
            "/workspace",
            "abc123",
            "sh",
            "-lc",
            "printf ok"
        ]
    );
    assert_eq!(
        DockerCliProvider::copy_out_args(&handle, "/workspace/out", "/tmp/out"),
        vec!["cp", "abc123:/workspace/out", "/tmp/out"]
    );
    assert_eq!(
        DockerCliProvider::destroy_args(&handle),
        vec!["rm", "-f", "abc123"]
    );
    assert_eq!(
        DockerCliProvider::ps_orphans_args("run-1"),
        vec!["ps", "-aq", "--filter", "label=harnesslab.run_id=run-1"]
    );
    assert_eq!(
        docker_shell_command(&DockerCliProvider::exec_args(&handle, "printf 'ok'")),
        "docker 'exec' '-i' '--workdir' '/workspace' 'abc123' 'sh' '-lc' 'printf '\\''ok'\\'''"
    );
}

#[test]
fn c_sbox_006_create_copy_and_destroy_use_runner_outputs() {
    let runner = FakeDockerRunner::new(vec![ok("container-1\n"), ok(""), ok("")]);
    let request = request();

    let handle = DockerCliProvider::create_with_runner(&request, &runner).unwrap();
    DockerCliProvider::copy_out_with_runner(
        &handle,
        "/workspace/out",
        PathBuf::from("/tmp/out").as_path(),
        &runner,
    )
    .unwrap();
    DockerCliProvider::destroy_with_runner(&handle, &runner).unwrap();

    assert_eq!(handle.container_id, "container-1");
    assert_eq!(runner.seen.borrow().len(), 3);
    assert_eq!(runner.seen.borrow()[0][0], "run");
    assert_eq!(runner.seen.borrow()[1][0], "cp");
    assert_eq!(runner.seen.borrow()[2][0], "rm");
}

#[test]
fn c_sbox_007_create_rejects_failed_or_empty_container_id() {
    let failed = FakeDockerRunner::new(vec![err("boom")]);
    let empty = FakeDockerRunner::new(vec![ok("  \n")]);

    let failed_error = DockerCliProvider::create_with_runner(&request(), &failed)
        .unwrap_err()
        .to_string();
    let empty_error = DockerCliProvider::create_with_runner(&request(), &empty)
        .unwrap_err()
        .to_string();

    assert!(failed_error.contains("docker run failed"));
    assert!(empty_error.contains("did not return a container id"));
}

#[test]
fn c_sbox_008_cleanup_orphans_removes_listed_containers() {
    let runner = FakeDockerRunner::new(vec![ok("a\n\nb\n"), ok(""), ok("")]);

    let result = DockerCliProvider::cleanup_orphans_with_runner("run-1", &runner).unwrap();

    assert_eq!(result.removed, vec!["a", "b"]);
    assert_eq!(
        runner.seen.borrow()[0],
        DockerCliProvider::ps_orphans_args("run-1")
    );
    assert_eq!(runner.seen.borrow()[1], vec!["rm", "-f", "a"]);
    assert_eq!(runner.seen.borrow()[2], vec!["rm", "-f", "b"]);
}

#[test]
fn c_sbox_009_error_paths_are_structured() {
    let handle = SandboxHandle {
        container_id: "abc123".to_string(),
        name: "name".to_string(),
        run_id: "run-1".to_string(),
        workspace_container_path: "/workspace".to_string(),
    };

    let copy = DockerCliProvider::copy_out_with_runner(
        &handle,
        "/workspace/out",
        PathBuf::from("/tmp/out").as_path(),
        &FakeDockerRunner::new(vec![err("copy failed")]),
    )
    .unwrap_err()
    .to_string();
    let destroy = DockerCliProvider::destroy_with_runner(
        &handle,
        &FakeDockerRunner::new(vec![err("rm failed")]),
    )
    .unwrap_err()
    .to_string();
    let cleanup = DockerCliProvider::cleanup_orphans_with_runner(
        "run-1",
        &FakeDockerRunner::new(vec![err("ps failed")]),
    )
    .unwrap_err()
    .to_string();

    assert!(copy.contains("docker cp failed"));
    assert!(destroy.contains("docker rm failed"));
    assert!(cleanup.contains("docker ps failed"));
}

#[test]
fn c_sbox_010_exec_without_docker_returns_process_record() {
    let tmp = tempfile::tempdir().unwrap();
    let handle = SandboxHandle {
        container_id: "missing".to_string(),
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
            timeout_sec: 1,
            stdout_path: tmp.path().join("stdout.log"),
            stderr_path: tmp.path().join("stderr.log"),
        },
    )
    .unwrap();

    assert_ne!(result.exit_code, Some(0));
    assert!(tmp.path().join("stderr.log").exists());
}

#[test]
fn c_sbox_011_create_args_cover_privileged_full_network_and_sanitized_names() {
    let mut request = request();
    request.run_id = String::new();
    request.task_id = "task/slash".to_string();
    request.network = NetworkPolicy::Full;
    request.privileged = true;
    request.cpu_cores = 0;
    request.memory_mb = 0;

    let args = DockerCliProvider::create_args(&request);

    assert!(args.contains(&"harnesslab-x-task-slash-1".to_string()));
    assert!(args.windows(2).any(|pair| pair == ["--network", "bridge"]));
    assert!(args.contains(&"--privileged".to_string()));
    assert!(!args.contains(&"--cpus".to_string()));
    assert!(!args.contains(&"--memory".to_string()));
}

fn request() -> DockerCreateRequest {
    DockerCreateRequest {
        run_id: "run-1".to_string(),
        task_id: "task-1".to_string(),
        attempt: 1,
        image: "alpine:3.20".to_string(),
        workspace_host_path: PathBuf::from("/tmp/workspace"),
        workspace_container_path: "/workspace".to_string(),
        network: NetworkPolicy::None,
        env_vars: Vec::new(),
        mounts: Vec::new(),
        privileged: false,
        cpu_cores: 1,
        memory_mb: 128,
    }
}

fn ok(stdout: &str) -> DockerCommandOutput {
    DockerCommandOutput {
        success: true,
        stdout: stdout.as_bytes().to_vec(),
        stderr: Vec::new(),
    }
}

fn err(stderr: &str) -> DockerCommandOutput {
    DockerCommandOutput {
        success: false,
        stdout: Vec::new(),
        stderr: stderr.as_bytes().to_vec(),
    }
}

struct FakeDockerRunner {
    outputs: RefCell<VecDeque<DockerCommandOutput>>,
    seen: RefCell<Vec<Vec<String>>>,
}

impl FakeDockerRunner {
    fn new(outputs: Vec<DockerCommandOutput>) -> Self {
        Self {
            outputs: RefCell::new(outputs.into()),
            seen: RefCell::new(Vec::new()),
        }
    }
}

impl DockerCommandRunner for FakeDockerRunner {
    fn output(&self, args: &[String]) -> Result<DockerCommandOutput> {
        self.seen.borrow_mut().push(args.to_vec());
        self.outputs
            .borrow_mut()
            .pop_front()
            .context("fake docker output")
    }
}
