use super::execute_attempts_with;
use crate::runner::schedule::AttemptWork;
use harnesslab_core::{
    AttemptProvenance, FailureClass, FailureCode, HealthImpact, NetworkPolicy, Outcome,
    TaskAttemptResult, TaskPlan, TaskState, UsageRecord,
};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

#[test]
fn run_004_attempt_scheduler_refills_slot_before_slow_task_finishes() {
    let tmp = tempfile::tempdir().unwrap();
    let attempts = ["slow", "fast", "refill"]
        .into_iter()
        .map(|task_id| AttemptWork {
            task: task_with_id(task_id),
            task_runtime_snapshot: None,
            attempt: 1,
            provenance: AttemptProvenance::Original,
        })
        .collect::<Vec<_>>();
    let (refill_started_tx, refill_started_rx) = mpsc::channel();
    let (release_slow_tx, release_slow_rx) = mpsc::channel();
    let release_slow_rx = Arc::new(Mutex::new(release_slow_rx));
    let executor = Arc::new(move |work: AttemptWork| {
        if work.task.task_id == "slow" {
            release_slow_rx
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(5))
                .expect("slow task should be released by test");
        }
        if work.task.task_id == "refill" {
            refill_started_tx
                .send(())
                .expect("test should receive refill start");
        }
        Ok(success_result(work))
    });
    let run_dir = tmp.path().to_path_buf();

    let handle =
        std::thread::spawn(move || execute_attempts_with(&run_dir, "run-1", attempts, 2, executor));
    if let Err(error) = refill_started_rx.recv_timeout(Duration::from_secs(2)) {
        let _ = release_slow_tx.send(());
        let _ = handle.join();
        panic!("refill task did not start while slow task was still active: {error}");
    }
    release_slow_tx.send(()).unwrap();
    let results = handle.join().unwrap().unwrap();

    assert_eq!(results.len(), 3);
    assert!(results.iter().any(|result| result.task_id == "refill"));
}

#[test]
fn run_004_attempt_scheduler_stops_refill_after_run_health_abort() {
    let tmp = tempfile::tempdir().unwrap();
    let attempts = [
        "stall-0", "stall-1", "stall-2", "stall-3", "stall-4", "active", "pending",
    ]
    .into_iter()
    .map(|task_id| AttemptWork {
        task: task_with_id(task_id),
        task_runtime_snapshot: None,
        attempt: 1,
        provenance: AttemptProvenance::Original,
    })
    .collect::<Vec<_>>();
    let started = Arc::new(Mutex::new(Vec::new()));
    let started_for_executor = Arc::clone(&started);
    let (active_started_tx, active_started_rx) = mpsc::channel();
    let (release_active_tx, release_active_rx) = mpsc::channel();
    let release_active_rx = Arc::new(Mutex::new(release_active_rx));
    let executor = Arc::new(move |work: AttemptWork| {
        started_for_executor
            .lock()
            .unwrap()
            .push(work.task.task_id.clone());
        if work.task.task_id == "active" {
            active_started_tx
                .send(())
                .expect("test should observe active start");
            release_active_rx
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(5))
                .expect("active task should be released by test");
            return Ok(success_result(work));
        }
        if work.task.task_id == "pending" {
            panic!("pending task should not be started after run-health abort");
        }
        Ok(stall_result(work))
    });
    let run_dir = tmp.path().to_path_buf();

    let handle =
        std::thread::spawn(move || execute_attempts_with(&run_dir, "run-1", attempts, 2, executor));
    active_started_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("active task should start before abort drains active workers");
    wait_for_run_health_abort(tmp.path());
    release_active_tx.send(()).unwrap();
    let results = handle.join().unwrap().unwrap();

    let started = started.lock().unwrap().clone();
    assert!(!started.iter().any(|task_id| task_id == "pending"));
    let pending = results
        .iter()
        .find(|result| result.task_id == "pending")
        .expect("pending attempt should be recorded as interrupted");
    assert_eq!(pending.state, TaskState::Interrupted);
    assert_eq!(pending.failure_code, Some(FailureCode::RunHealthAborted));
    assert!(
        tmp.path()
            .join("tasks/pending/attempts/1/result.json")
            .exists()
    );
}

#[test]
fn run_004_attempt_scheduler_stops_refill_after_worker_error() {
    let tmp = tempfile::tempdir().unwrap();
    let attempts = ["slow", "error", "pending"]
        .into_iter()
        .map(|task_id| AttemptWork {
            task: task_with_id(task_id),
            task_runtime_snapshot: None,
            attempt: 1,
            provenance: AttemptProvenance::Original,
        })
        .collect::<Vec<_>>();
    let started = Arc::new(Mutex::new(Vec::new()));
    let started_for_executor = Arc::clone(&started);
    let (slow_started_tx, slow_started_rx) = mpsc::channel();
    let (release_slow_tx, release_slow_rx) = mpsc::channel();
    let release_slow_rx = Arc::new(Mutex::new(release_slow_rx));
    let executor = Arc::new(move |work: AttemptWork| {
        started_for_executor
            .lock()
            .unwrap()
            .push(work.task.task_id.clone());
        match work.task.task_id.as_str() {
            "slow" => {
                slow_started_tx
                    .send(())
                    .expect("test should observe slow start");
                release_slow_rx
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(5))
                    .expect("slow task should be released by test");
                Ok(success_result(work))
            }
            "error" => Err(anyhow::anyhow!("fatal worker error")),
            "pending" => panic!("pending task should not be started after worker error"),
            _ => Ok(success_result(work)),
        }
    });
    let run_dir = tmp.path().to_path_buf();

    let handle =
        std::thread::spawn(move || execute_attempts_with(&run_dir, "run-1", attempts, 2, executor));
    slow_started_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("slow task should start");
    wait_until_started(&started, "error");
    std::thread::sleep(Duration::from_millis(100));
    assert!(
        !started
            .lock()
            .unwrap()
            .iter()
            .any(|task_id| task_id == "pending")
    );
    release_slow_tx.send(()).unwrap();
    let error = handle.join().unwrap().unwrap_err();

    assert!(error.to_string().contains("fatal worker error"));
    assert!(
        !started
            .lock()
            .unwrap()
            .iter()
            .any(|task_id| task_id == "pending")
    );
}

#[test]
fn run_004_attempt_scheduler_stops_refill_after_worker_panic() {
    let tmp = tempfile::tempdir().unwrap();
    let attempts = ["slow", "panic", "pending"]
        .into_iter()
        .map(|task_id| AttemptWork {
            task: task_with_id(task_id),
            task_runtime_snapshot: None,
            attempt: 1,
            provenance: AttemptProvenance::Original,
        })
        .collect::<Vec<_>>();
    let started = Arc::new(Mutex::new(Vec::new()));
    let started_for_executor = Arc::clone(&started);
    let (slow_started_tx, slow_started_rx) = mpsc::channel();
    let (release_slow_tx, release_slow_rx) = mpsc::channel();
    let release_slow_rx = Arc::new(Mutex::new(release_slow_rx));
    let executor = Arc::new(move |work: AttemptWork| {
        started_for_executor
            .lock()
            .unwrap()
            .push(work.task.task_id.clone());
        match work.task.task_id.as_str() {
            "slow" => {
                slow_started_tx
                    .send(())
                    .expect("test should observe slow start");
                release_slow_rx
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(5))
                    .expect("slow task should be released by test");
                Ok(success_result(work))
            }
            "panic" => panic!("fatal worker panic"),
            "pending" => panic!("pending task should not be started after worker panic"),
            _ => Ok(success_result(work)),
        }
    });
    let run_dir = tmp.path().to_path_buf();

    let handle =
        std::thread::spawn(move || execute_attempts_with(&run_dir, "run-1", attempts, 2, executor));
    slow_started_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("slow task should start");
    wait_until_started(&started, "panic");
    std::thread::sleep(Duration::from_millis(100));
    assert!(
        !started
            .lock()
            .unwrap()
            .iter()
            .any(|task_id| task_id == "pending")
    );
    release_slow_tx.send(()).unwrap();
    let error = handle.join().unwrap().unwrap_err();

    assert!(
        error
            .to_string()
            .contains("task panicked: fatal worker panic")
    );
    assert!(
        !started
            .lock()
            .unwrap()
            .iter()
            .any(|task_id| task_id == "pending")
    );
}

fn success_result(work: AttemptWork) -> TaskAttemptResult {
    TaskAttemptResult {
        schema_version: 1,
        task_id: work.task.task_id,
        attempt: work.attempt,
        provenance: work.provenance,
        state: TaskState::Success,
        outcome: Outcome::Success,
        failure_class: FailureClass::None,
        failure_code: None,
        health_impact: HealthImpact::None,
        benchmark_score: 1.0,
        duration_ms: 1,
        agent: None,
        evaluation: None,
        patch: None,
        usage: UsageRecord::Unknown,
        warnings: Vec::new(),
    }
}

fn stall_result(work: AttemptWork) -> TaskAttemptResult {
    TaskAttemptResult {
        schema_version: 1,
        task_id: work.task.task_id,
        attempt: work.attempt,
        provenance: work.provenance,
        state: TaskState::Failure,
        outcome: Outcome::Failure,
        failure_class: FailureClass::Execution,
        failure_code: Some(FailureCode::ExternalRunnerNoProgress),
        health_impact: HealthImpact::Stall,
        benchmark_score: 0.0,
        duration_ms: 1,
        agent: None,
        evaluation: None,
        patch: None,
        usage: UsageRecord::Unknown,
        warnings: Vec::new(),
    }
}

fn wait_for_run_health_abort(run_dir: &std::path::Path) {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if let Ok(content) = std::fs::read_to_string(run_dir.join("run-health.json")) {
            let health = serde_json::from_str::<serde_json::Value>(&content).unwrap();
            if health["status"] == "invalid" {
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("run health did not abort");
}

fn wait_until_started(started: &Arc<Mutex<Vec<String>>>, task_id: &str) {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if started
            .lock()
            .unwrap()
            .iter()
            .any(|started| started == task_id)
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("{task_id} did not start");
}

fn task_with_id(task_id: &str) -> TaskPlan {
    TaskPlan {
        task_id: task_id.to_string(),
        instruction: "instruction".to_string(),
        workspace_spec: harnesslab_core::WorkspaceSpec {
            workspace_type: harnesslab_core::WorkspaceType::Empty,
            target_path: "workspace".to_string(),
            clean: true,
        },
        sandbox_spec: harnesslab_core::SandboxSpec {
            image: "host".to_string(),
            mounts: Vec::new(),
            env_vars: Vec::new(),
            network: NetworkPolicy::None,
            privileged: false,
            resource_limits: harnesslab_core::ResourceHint {
                cpu_cores: 1,
                memory_mb: 128,
            },
        },
        verifier_spec: harnesslab_core::VerifierSpec {
            command: "true".to_string(),
            working_dir: "workspace".to_string(),
            timeout_sec: 1,
            expected_exit_codes: vec![0],
            environment_mode: harnesslab_core::VerifierEnvironment::HostProcess,
            output_parser: "exit_code".to_string(),
        },
        artifact_spec: harnesslab_core::ArtifactSpec {
            base_dir: "workspace".to_string(),
            globs: Vec::new(),
            required_paths: Vec::new(),
            max_size_bytes: 1,
        },
        patch_spec: None,
        external_runner: None,
    }
}
