use anyhow::Result;
use std::fs::{self, OpenOptions};
#[cfg(unix)]
use std::os::fd::{AsRawFd, RawFd};
use std::path::Path;

pub fn with_exclusive_file_lock<T>(
    lock_path: &Path,
    work: impl FnOnce() -> Result<T>,
) -> Result<T> {
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)?;
    let _guard = FileLockGuard::lock(&file)?;
    work()
}

#[cfg(unix)]
struct FileLockGuard {
    fd: RawFd,
}

#[cfg(unix)]
impl FileLockGuard {
    fn lock(file: &fs::File) -> Result<Self> {
        let fd = file.as_raw_fd();
        let result = unsafe { libc::flock(fd, libc::LOCK_EX) };
        if result != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(Self { fd })
    }
}

#[cfg(unix)]
impl Drop for FileLockGuard {
    fn drop(&mut self) {
        let _ = unsafe { libc::flock(self.fd, libc::LOCK_UN) };
    }
}

#[cfg(not(unix))]
struct FileLockGuard;

#[cfg(not(unix))]
impl FileLockGuard {
    fn lock(_file: &fs::File) -> Result<Self> {
        anyhow::bail!("exclusive file lock is not supported on this platform")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn lock_001_serializes_file_mutation() {
        let tmp = tempfile::tempdir().unwrap();
        let lock_path = tmp.path().join("state.lock");
        let state_path = tmp.path().join("state.txt");
        fs::write(&state_path, "0").unwrap();
        let barrier = Arc::new(Barrier::new(8));

        let mut workers = Vec::new();
        for _ in 0..8 {
            let lock_path = lock_path.clone();
            let state_path = state_path.clone();
            let barrier = Arc::clone(&barrier);
            workers.push(thread::spawn(move || {
                barrier.wait();
                for _ in 0..20 {
                    with_exclusive_file_lock(&lock_path, || {
                        let value: u32 = fs::read_to_string(&state_path)?.parse()?;
                        fs::write(&state_path, (value + 1).to_string())?;
                        Ok(())
                    })
                    .unwrap();
                }
            }));
        }

        for worker in workers {
            worker.join().unwrap();
        }

        assert_eq!(fs::read_to_string(state_path).unwrap(), "160");
    }
}
