from __future__ import annotations

import hashlib
import os
import re
import shlex
import subprocess
import tempfile
import uuid
from pathlib import Path

from terminal_bench.agents.base_agent import AgentResult, BaseAgent
from terminal_bench.agents.failure_mode import FailureMode
from terminal_bench.terminal.models import TerminalCommand
from terminal_bench.terminal.tmux_session import TmuxSession

from ornnlab_tb_process import AgentCommandTimedOut, run_agent_process


class OrnnLabCommandAgent(BaseAgent):
    @staticmethod
    def name() -> str:
        return "ornnlab-command"

    def perform_task(
        self,
        instruction: str,
        session: TmuxSession,
        logging_dir: Path | None = None,
    ) -> AgentResult:
        command = required_env("ORNNLAB_AGENT_COMMAND")
        input_mode = os.environ.get("ORNNLAB_AGENT_INPUT_MODE", "stdin")
        timeout = int(os.environ.get("ORNNLAB_AGENT_TIMEOUT_SEC", "3600"))
        prompt = self._command_prompt(self._render_instruction(instruction))
        log_dir = prepare_log_dir(logging_dir)
        write_log(log_dir, "prompt.txt", prompt)
        setup_command = os.environ.get("ORNNLAB_AGENT_SETUP_COMMAND", "")
        if setup_command.strip():
            write_log(log_dir, "agent_setup_command.sha256", sha256_hex(setup_command))
            try:
                setup = subprocess.run(
                    setup_command,
                    shell=True,
                    text=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    timeout=max(timeout, 1),
                )
            except Exception as error:
                write_log(log_dir, "agent_setup_error.log", f"ornnlab agent setup failed: {error}")
                return AgentResult(failure_mode=FailureMode.UNKNOWN_AGENT_ERROR)
            write_log(log_dir, "agent_setup_stdout.log", setup.stdout or "")
            write_log(log_dir, "agent_setup_stderr.log", setup.stderr or "")
            if setup.returncode != 0:
                write_log(
                    log_dir,
                    "agent_setup_error.log",
                    f"ornnlab agent setup failed: exit_code={setup.returncode}",
                )
                return AgentResult(failure_mode=FailureMode.UNKNOWN_AGENT_ERROR)

        try:
            output = run_registered_agent(
                command,
                input_mode,
                prompt,
                timeout,
                log_path(log_dir, "agent_cleanup.log"),
            )
        except AgentCommandTimedOut as error:
            write_log(log_dir, "agent_error.log", str(error))
            write_log(log_dir, "agent_stdout_partial.log", error.stdout)
            write_log(log_dir, "agent_stderr_partial.log", error.stderr)
            if error.cleanup_succeeded:
                return AgentResult(failure_mode=FailureMode.AGENT_TIMEOUT)
            return AgentResult(failure_mode=FailureMode.UNKNOWN_AGENT_ERROR)
        except Exception as error:
            write_log(log_dir, "agent_error.log", str(error))
            return AgentResult(failure_mode=FailureMode.UNKNOWN_AGENT_ERROR)

        script = extract_shell_script(output)
        write_log(log_dir, "agent_output.txt", output)
        write_log(log_dir, "container_script.sh", script)
        if not script.strip():
            return AgentResult(failure_mode=FailureMode.PARSE_ERROR)
        execution_shell, shell_resolution = resolve_execution_shell(session)
        write_log(log_dir, "execution_shell.txt", execution_shell)
        if shell_resolution:
            write_log(log_dir, "execution_shell_resolution.log", shell_resolution)
        syntax_error = shell_syntax_error(script, session, execution_shell)
        if syntax_error:
            write_log(log_dir, "script_syntax_error.log", syntax_error)
            return AgentResult(failure_mode=FailureMode.PARSE_ERROR)
        container_command = build_container_execution_command(script, execution_shell)
        write_log(log_dir, "container_command.sh", container_command)

        session.send_command(
            TerminalCommand(
                command=container_command,
                min_timeout_sec=0.0,
                max_timeout_sec=float(timeout),
                block=True,
                append_enter=True,
            )
        )
        return AgentResult()

    def _command_prompt(self, instruction: str) -> str:
        return (
            "You are solving a Terminal-Bench task inside a Linux shell. "
            "Return only a POSIX sh script that should be executed in the task "
            "container. Do not include Markdown fences or explanations.\n\n"
            "Task instruction:\n"
            f"{instruction}\n"
        )


def run_registered_agent(
    command: str,
    input_mode: str,
    prompt: str,
    timeout: int,
    cleanup_log_path: Path | None = None,
) -> str:
    stdin = None
    if input_mode == "stdin":
        rendered = wrap_run_as(command)
        stdin = prompt
    elif input_mode == "argument":
        rendered = wrap_run_as(command.replace("{{instruction}}", shlex.quote(prompt)))
    elif input_mode == "file":
        with tempfile.NamedTemporaryFile("w", delete=False) as handle:
            handle.write(prompt)
            prompt_path = handle.name
        try:
            rendered = command.replace("{{instruction_file}}", shlex.quote(prompt_path))
            rendered = rendered.replace("{{instruction}}", shlex.quote(prompt_path))
            rendered = wrap_run_as(rendered)
            completed = run_agent_process(rendered, stdin, timeout, cleanup_log_path)
        finally:
            Path(prompt_path).unlink(missing_ok=True)
        return completed_stdout_or_error(completed)
    else:
        raise ValueError(f"unsupported input mode: {input_mode}")

    completed = run_agent_process(rendered, stdin, timeout, cleanup_log_path)
    return completed_stdout_or_error(completed)


def wrap_run_as(command: str) -> str:
    run_as = os.environ.get("ORNNLAB_AGENT_RUN_AS", "current")
    if run_as != "ornnlab":
        return command
    quoted = shlex.quote(command)
    return (
        'if [ "$(id -u)" = "0" ] && id -u ornnlab >/dev/null 2>&1; '
        f"then exec runuser -u ornnlab --preserve-environment -- bash -lc {quoted}; "
        f"else exec bash -lc {quoted}; fi"
    )


def completed_stdout_or_error(completed: subprocess.CompletedProcess) -> str:
    if completed.returncode != 0:
        raise RuntimeError(
            "agent exited "
            f"{completed.returncode}: stdout={trim_for_log(completed.stdout)} "
            f"stderr={trim_for_log(completed.stderr)}"
        )
    return completed.stdout


def trim_for_log(text: str | None, limit: int = 4000) -> str:
    if not text:
        return ""
    stripped = text.strip()
    if len(stripped) <= limit:
        return stripped
    return stripped[:limit] + "...<truncated>"


def extract_shell_script(output: str) -> str:
    fenced = re.search(r"```(?:sh|bash|shell)?\s*(.*?)```", output, re.DOTALL)
    if fenced:
        return fenced.group(1).strip()
    stripped = output.strip()
    lines = stripped.splitlines()
    for index, line in enumerate(lines):
        if looks_like_shell_start(line):
            if index == 0:
                return stripped
            if not all(looks_like_preamble_line(prefix) for prefix in lines[:index]):
                return stripped
            return "\n".join(lines[index:]).strip()
    return stripped


def looks_like_preamble_line(line: str) -> bool:
    stripped = line.strip()
    if not stripped:
        return True
    return not re.search(r"[=(){}<>|;&`$\\\\]", stripped)


def looks_like_shell_start(line: str) -> bool:
    stripped = line.strip()
    if not stripped:
        return False
    first = stripped.split(maxsplit=1)[0]
    if re.match(r"^[A-Za-z_][A-Za-z0-9_]*=", stripped):
        return True
    return stripped.startswith("#") or first in {
        "{",
        "(",
        "if",
        "then",
        "elif",
        "else",
        "fi",
        "for",
        "while",
        "until",
        "do",
        "done",
        "case",
        "esac",
        "bash",
        "sh",
        "/bin/bash",
        "/bin/sh",
        "set",
        "export",
        "apt-get",
        "apt",
        "apk",
        "yum",
        "dnf",
        "mkdir",
        "cd",
        "cat",
        "chmod",
        "chown",
        "wget",
        "curl",
        "git",
        "tar",
        "gzip",
        "cp",
        "mv",
        "rm",
        "make",
        "./configure",
        "python",
        "python3",
        "pip",
        "uv",
        "cargo",
        "npm",
        "echo",
        "printf",
        "qemu-system-x86_64",
    }


def resolve_execution_shell(session: TmuxSession | None) -> tuple[str, str | None]:
    container = getattr(session, "container", None)
    user = execution_user(session)
    if container is None:
        return (
            "/bin/sh",
            f"no terminal container available; session_user={user}; using /bin/sh",
        )
    session_name = getattr(session, "_session_name", "")
    if not session_name:
        return (
            "/bin/sh",
            f"terminal session name unavailable; session_user={user}; using /bin/sh",
        )
    try:
        result = exec_in_session(
            session,
            ["tmux", "show-options", "-t", session_name, "-v", "default-shell"],
        )
    except Exception as error:
        return (
            "/bin/sh",
            f"tmux default-shell lookup failed: session={session_name} user={user} error={error}; using /bin/sh",
        )
    if getattr(result, "exit_code", 1) != 0:
        output = decode_exec_output(getattr(result, "output", b"")).strip()
        return (
            "/bin/sh",
            f"tmux default-shell lookup exited {result.exit_code}: session={session_name} user={user} output={output}; using /bin/sh",
        )
    shell = decode_exec_output(getattr(result, "output", b"")).strip()
    if not shell:
        return (
            "/bin/sh",
            f"tmux default-shell lookup returned empty output: session={session_name} user={user}; using /bin/sh",
        )
    return shell, None


def shell_syntax_error(
    script: str,
    session: TmuxSession | None = None,
    shell: str = "/bin/sh",
) -> str | None:
    container = getattr(session, "container", None)
    if container is not None:
        return container_shell_syntax_error(session, script, shell)
    try:
        completed = subprocess.run(
            [shell, "-n"],
            input=script,
            text=True,
            capture_output=True,
            check=False,
        )
    except OSError as error:
        return f"shell syntax check failed: {error}"
    if completed.returncode == 0:
        return None
    return syntax_error_message(shell, completed.returncode, completed.stderr)


def container_shell_syntax_error(
    session: TmuxSession,
    script: str,
    shell: str,
) -> str | None:
    path = f"/tmp/ornnlab-agent-syntax-{uuid.uuid4().hex}.sh"
    command = build_container_syntax_command(script, shell, path)
    try:
        result = exec_in_session(session, ["/bin/sh", "-lc", command])
    except Exception as error:
        return (
            "shell syntax check failed: "
            f"shell={shell} user={execution_user(session)} error={error}"
        )
    exit_code = getattr(result, "exit_code", 1)
    output = decode_exec_output(getattr(result, "output", b""))
    if exit_code == 0:
        return None
    return syntax_error_message(shell, exit_code, output)


def exec_in_session(session: TmuxSession, command: list[str]):
    return session.container.exec_run(command, user=execution_user(session))


def execution_user(session: TmuxSession | None) -> str:
    return getattr(session, "_user", "") or ""


def build_container_syntax_command(script: str, shell: str, path: str) -> str:
    delimiter = heredoc_delimiter(script)
    quoted_path = shlex.quote(path)
    quoted_shell = shlex.quote(shell)
    return (
        f"cat > {quoted_path} <<'{delimiter}'\n"
        f"{script}\n"
        f"{delimiter}\n"
        f"{quoted_shell} -n {quoted_path}\n"
        "status=$?\n"
        f"rm -f {quoted_path}\n"
        'exit "$status"'
    )


def build_container_execution_command(script: str, shell: str) -> str:
    delimiter = heredoc_delimiter(script)
    path = f"/tmp/ornnlab-agent-run-{uuid.uuid4().hex}.sh"
    quoted_path = shlex.quote(path)
    quoted_shell = shlex.quote(shell)
    body = (
        f"cat > {quoted_path} <<'{delimiter}'\n"
        f"{script}\n"
        f"{delimiter}\n"
        f"{quoted_shell} {quoted_path}\n"
        "status=$?\n"
        f"rm -f {quoted_path}\n"
        'test "$status" -eq 0'
    )
    return f"/bin/sh -lc {shlex.quote(body)}"


def heredoc_delimiter(script: str) -> str:
    delimiter = f"ORNNLAB_SCRIPT_{uuid.uuid4().hex}"
    while delimiter in script:
        delimiter = f"ORNNLAB_SCRIPT_{uuid.uuid4().hex}"
    return delimiter


def syntax_error_message(shell: str, exit_code: int, output: str) -> str:
    details = output.strip() or "shell syntax check failed"
    return f"shell={shell}\nexit_code={exit_code}\n{details}"


def decode_exec_output(output) -> str:
    if isinstance(output, bytes):
        return output.decode(errors="replace")
    return str(output)


def required_env(name: str) -> str:
    value = os.environ.get(name)
    if not value:
        raise RuntimeError(f"missing required environment variable {name}")
    return value


def prepare_log_dir(logging_dir: Path | None) -> Path | None:
    if logging_dir is None:
        return None
    logging_dir.mkdir(parents=True, exist_ok=True)
    return logging_dir


def write_log(log_dir: Path | None, name: str, text: str) -> None:
    if log_dir is None:
        return
    (log_dir / name).write_text(text)


def sha256_hex(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def log_path(log_dir: Path | None, name: str) -> Path | None:
    if log_dir is None:
        return None
    return log_dir / name
