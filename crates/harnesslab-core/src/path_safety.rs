use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PathSafetyError {
    #[error("task_id must not be empty")]
    EmptyTaskId,
    #[error("unsafe relative artifact path {0}")]
    UnsafeArtifactPath(String),
}

pub fn task_dir_name(task_id: &str) -> Result<String, PathSafetyError> {
    if task_id.trim().is_empty() {
        return Err(PathSafetyError::EmptyTaskId);
    }
    Ok(percent_encode_segment(task_id))
}

pub fn report_artifact_path(path: &str) -> Result<&str, PathSafetyError> {
    if is_safe_relative_artifact_path(path) {
        Ok(path)
    } else {
        Err(PathSafetyError::UnsafeArtifactPath(path.to_string()))
    }
}

fn percent_encode_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        let ch = byte as char;
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            encoded.push(ch);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn is_safe_relative_artifact_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && path
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
        && !path.contains('\\')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_001_task_dir_encodes_separator_and_keeps_display_id_usable() {
        assert_eq!(task_dir_name("owner/repo#42").unwrap(), "owner%2Frepo%2342");
    }

    #[test]
    fn path_002_task_dir_rejects_empty_id() {
        assert_eq!(task_dir_name(" "), Err(PathSafetyError::EmptyTaskId));
    }

    #[test]
    fn path_003_report_artifact_path_rejects_escape_segments() {
        assert!(report_artifact_path("patch.diff").is_ok());
        assert!(report_artifact_path("nested/patch.diff").is_ok());
        assert!(report_artifact_path("../patch.diff").is_err());
        assert!(report_artifact_path("/patch.diff").is_err());
        assert!(report_artifact_path(r"nested\\patch.diff").is_err());
    }
}
