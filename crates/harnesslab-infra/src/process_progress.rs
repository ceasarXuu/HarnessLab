use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ProgressWatcher {
    paths: Vec<PathBuf>,
    states: Vec<FileState>,
}

impl ProgressWatcher {
    pub fn new(paths: Vec<PathBuf>) -> Self {
        let states = paths.iter().map(FileState::read).collect();
        Self { paths, states }
    }

    pub fn changed_path(&mut self) -> Option<PathBuf> {
        for (index, path) in self.paths.iter().enumerate() {
            let current = FileState::read(path);
            if current.is_progress_since(self.states[index]) {
                self.states[index] = current;
                return Some(path.clone());
            }
            self.states[index] = current;
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileState {
    len: Option<u64>,
}

impl FileState {
    fn read(path: &PathBuf) -> Self {
        match fs::metadata(path) {
            Ok(metadata) => Self {
                len: Some(metadata.len()),
            },
            Err(_) => Self { len: None },
        }
    }

    fn is_progress_since(self, previous: Self) -> bool {
        match (previous.len, self.len) {
            (Some(before), Some(after)) if after > before => true,
            (None, Some(after)) if after > 0 => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProgressWatcher;
    use std::fs;

    #[test]
    fn detects_created_file_with_content() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("run.log");
        let mut watcher = ProgressWatcher::new(vec![path.clone()]);

        fs::write(&path, "started").unwrap();

        assert_eq!(watcher.changed_path(), Some(path));
    }

    #[test]
    fn ignores_empty_file_creation() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("run.log");
        let mut watcher = ProgressWatcher::new(vec![path.clone()]);

        fs::write(&path, "").unwrap();

        assert_eq!(watcher.changed_path(), None);
    }

    #[test]
    fn detects_content_growth() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("run.log");
        fs::write(&path, "started").unwrap();
        let mut watcher = ProgressWatcher::new(vec![path.clone()]);

        fs::write(&path, "started\nnext").unwrap();

        assert_eq!(watcher.changed_path(), Some(path));
    }

    #[test]
    fn ignores_same_size_content_rewrite() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("run.log");
        fs::write(&path, "aaaa").unwrap();
        let mut watcher = ProgressWatcher::new(vec![path.clone()]);

        fs::write(&path, "bbbb").unwrap();

        assert_eq!(watcher.changed_path(), None);
    }
}
