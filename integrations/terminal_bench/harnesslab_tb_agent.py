from __future__ import annotations

import os
import re
import shlex
import subprocess
import tempfile
from pathlib import Path

from terminal_bench.agents.base_agent import AgentResult, BaseAgent
from terminal_bench.agents.failure_mode import FailureMode
from terminal_bench.terminal.models import TerminalCommand
from terminal_bench.terminal.tmux_session import TmuxSession


class HarnessLabCommandAgent(BaseAgent):
    @staticmethod
    def name() -> str:
        return "harnesslab-command"

    def perform_task(
        self,
        instruction: str,
        session: TmuxSession,
        logging_dir: Path | None = None,
    ) -> AgentResult:
        command = required_env("HARNESSLAB_AGENT_COMMAND")
        input_mode = os.environ.get("HARNESSLAB_AGENT_INPUT_MODE", "stdin")
        timeout = int(os.environ.get("HARNESSLAB_AGENT_TIMEOUT_SEC", "3600"))
        prompt = self._command_prompt(self._render_instruction(instruction))
        log_dir = prepare_log_dir(logging_dir)

        try:
            output = run_registered_agent(command, input_mode, prompt, timeout)
        except Exception as error:
            write_log(log_dir, "agent_error.log", str(error))
            return AgentResult(failure_mode=FailureMode.UNKNOWN_AGENT_ERROR)

        script = extract_shell_script(output)
        write_log(log_dir, "prompt.txt", prompt)
        write_log(log_dir, "agent_output.txt", output)
        write_log(log_dir, "container_script.sh", script)
        if not script.strip():
            return AgentResult(failure_mode=FailureMode.PARSE_ERROR)

        session.send_command(
            TerminalCommand(
                command=script,
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
) -> str:
    if input_mode == "stdin":
        completed = subprocess.run(
            command,
            input=prompt,
            text=True,
            shell=True,
            capture_output=True,
            timeout=timeout,
            check=False,
        )
    elif input_mode == "argument":
        completed = subprocess.run(
            command.replace("{{instruction}}", shlex.quote(prompt)),
            text=True,
            shell=True,
            capture_output=True,
            timeout=timeout,
            check=False,
        )
    elif input_mode == "file":
        with tempfile.NamedTemporaryFile("w", delete=False) as handle:
            handle.write(prompt)
            prompt_path = handle.name
        try:
            rendered = command.replace("{{instruction_file}}", shlex.quote(prompt_path))
            rendered = rendered.replace("{{instruction}}", shlex.quote(prompt_path))
            completed = subprocess.run(
                rendered,
                text=True,
                shell=True,
                capture_output=True,
                timeout=timeout,
                check=False,
            )
        finally:
            Path(prompt_path).unlink(missing_ok=True)
    else:
        raise ValueError(f"unsupported input mode: {input_mode}")

    if completed.returncode != 0:
        raise RuntimeError(
            f"agent exited {completed.returncode}: {completed.stderr.strip()}"
        )
    return completed.stdout


def extract_shell_script(output: str) -> str:
    fenced = re.search(r"```(?:sh|bash|shell)?\s*(.*?)```", output, re.DOTALL)
    if fenced:
        return fenced.group(1).strip()
    return output.strip()


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
