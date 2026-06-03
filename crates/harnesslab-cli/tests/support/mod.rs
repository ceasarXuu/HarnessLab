#![allow(dead_code)]

pub mod agent_schema_docs;
pub mod swe;

use std::fs;
use std::path::Path;

pub fn assert_public_artifacts_do_not_contain(run_dir: &Path, secret: &str) {
    let mut stack = vec![run_dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("agent-profile.runtime.json")
            {
                continue;
            }
            let content = String::from_utf8_lossy(&fs::read(&path).unwrap()).to_string();
            assert!(
                !content.contains(secret),
                "public artifact {} leaked secret",
                path.display()
            );
        }
    }
}
