#[path = "support/terminal_bench.rs"]
mod terminal_bench_support;

use std::fs;
use terminal_bench_support::{
    fake_uvx, harnesslab, init_home, path_with, terminal_bench_root,
    write_agent_with_labels_and_run_as,
};

#[test]
fn agt_reg_012_terminal_bench_import_agent_run_as_blocks_before_run_dir() {
    let home = tempfile::tempdir().unwrap();
    init_home(home.path());
    write_agent_with_labels_and_run_as(
        home.path(),
        r#"terminal_bench_agent_import_path = "pkg.agent:Agent"
"#,
        "harnesslab",
    );
    let root = terminal_bench_root();
    let bin = fake_uvx("exit 99\n");

    let output = harnesslab()
        .env("HARNESSLAB_BENCHMARKS_DIR", root.path())
        .env("PATH", path_with(bin.path()))
        .args([
            "--home",
            home.path().to_str().unwrap(),
            "run",
            "--agent",
            "fake",
            "--benchmark",
            "terminal-bench",
            "--split",
            "smoke",
            "--json",
        ])
        .assert()
        .code(3)
        .get_output()
        .stderr
        .clone();

    let stderr = String::from_utf8(output).unwrap();
    assert!(stderr.contains("setup.run_as"));
    assert!(stderr.contains("terminal-bench import agent host path"));
    assert!(stderr.contains("current"));
    assert_eq!(fs::read_dir(home.path().join("runs")).unwrap().count(), 0);
}
