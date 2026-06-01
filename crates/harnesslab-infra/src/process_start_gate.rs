#[cfg(unix)]
#[derive(Clone, Copy)]
pub(super) struct ChildStartGateFds {
    write_fd: libc::c_int,
}

#[cfg(not(unix))]
#[derive(Clone, Copy)]
pub(super) struct ChildStartGateFds;

#[cfg(unix)]
pub(super) struct ChildStartGate {
    read_fd: libc::c_int,
    write_fd: libc::c_int,
}

#[cfg(unix)]
impl ChildStartGate {
    pub(super) fn new() -> std::io::Result<Self> {
        let mut fds = [0; 2];
        if unsafe { libc::pipe(fds.as_mut_ptr()) } == -1 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self {
            read_fd: fds[0],
            write_fd: fds[1],
        })
    }

    pub(super) fn child_fds(&self) -> ChildStartGateFds {
        ChildStartGateFds {
            write_fd: self.write_fd,
        }
    }

    pub(super) fn wrap_command(&self, command: &str) -> String {
        format!(
            "if IFS= read -r _ <&{}; then exec {}<&-; exec sh -c {}; else exit 127; fi",
            self.read_fd,
            self.read_fd,
            shell_quote(command)
        )
    }

    pub(super) fn close_child_end(&mut self) {
        close_fd(&mut self.read_fd);
    }

    pub(super) fn release(&mut self) -> std::io::Result<()> {
        if self.write_fd < 0 {
            return Ok(());
        }
        let byte = [b'\n'];
        let written = unsafe {
            libc::write(
                self.write_fd,
                byte.as_ptr().cast::<libc::c_void>(),
                byte.len(),
            )
        };
        let result = if written == 1 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        };
        close_fd(&mut self.write_fd);
        result
    }
}

#[cfg(unix)]
impl Drop for ChildStartGate {
    fn drop(&mut self) {
        close_fd(&mut self.read_fd);
        close_fd(&mut self.write_fd);
    }
}

#[cfg(not(unix))]
pub(super) struct ChildStartGate;

#[cfg(not(unix))]
impl ChildStartGate {
    pub(super) fn new() -> std::io::Result<Self> {
        Ok(Self)
    }

    pub(super) fn child_fds(&self) -> ChildStartGateFds {
        ChildStartGateFds
    }

    pub(super) fn wrap_command(&self, command: &str) -> String {
        command.to_string()
    }

    pub(super) fn close_child_end(&mut self) {}

    pub(super) fn release(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(unix)]
pub(super) fn prepare_child_after_setsid(start_gate: ChildStartGateFds) {
    unsafe {
        libc::close(start_gate.write_fd);
    }
}

#[cfg(unix)]
fn close_fd(fd: &mut libc::c_int) {
    if *fd >= 0 {
        unsafe {
            libc::close(*fd);
        }
        *fd = -1;
    }
}

#[cfg(unix)]
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn release_is_idempotent_after_success() {
        let mut gate = ChildStartGate::new().unwrap();

        gate.release().unwrap();
        gate.release().unwrap();
    }

    #[test]
    fn prepare_child_closes_write_end_before_exec() {
        let mut gate = ChildStartGate::new().unwrap();
        let child_fds = gate.child_fds();

        prepare_child_after_setsid(child_fds);

        assert!(gate.release().is_err());
    }

    #[test]
    fn wrapped_command_quotes_user_command() {
        let gate = ChildStartGate::new().unwrap();

        let command = gate.wrap_command("printf 'quoted'");

        assert!(command.contains("exec sh -c 'printf '\\''quoted'\\'''"));
    }
}
