import hashlib
import os
import shlex
import tempfile
from pathlib import Path
from unittest.mock import patch

import pytest

terminal_bench = pytest.importorskip("terminal_bench.agents.failure_mode")
FailureMode = terminal_bench.FailureMode

from harnesslab_tb_agent import HarnessLabCommandAgent


class FakeSession:
    def __init__(self):
        self.commands = []
        self.container = None
        self._session_name = "task-session"
        self._user = ""

    def send_command(self, command):
        self.commands.append(command)


def agent_print_command(output):
    return f"python -c {shlex.quote(f'print({output!r})')}"


def test_agt_reg_005_runs_setup_command_before_agent():
    agent = HarnessLabCommandAgent()
    with tempfile.TemporaryDirectory() as tmp:
        log_dir = Path(tmp)
        marker = log_dir / "setup-marker"
        setup_code = f"import pathlib; pathlib.Path({str(marker)!r}).write_text('ok')"
        env = {
            "HARNESSLAB_AGENT_SETUP_COMMAND": f"python -c {shlex.quote(setup_code)}",
            "HARNESSLAB_AGENT_COMMAND": agent_print_command("echo ok"),
            "HARNESSLAB_AGENT_INPUT_MODE": "stdin",
            "HARNESSLAB_AGENT_TIMEOUT_SEC": "5",
        }
        with patch.dict(os.environ, env, clear=False):
            result = agent.perform_task("do it", FakeSession(), log_dir)

        assert result.failure_mode == FailureMode.NONE
        assert marker.read_text() == "ok"
        assert (log_dir / "agent_setup_stdout.log").exists()
        assert (log_dir / "agent_setup_stderr.log").exists()
        expected_hash = hashlib.sha256(env["HARNESSLAB_AGENT_SETUP_COMMAND"].encode()).hexdigest()
        assert (log_dir / "agent_setup_command.sha256").read_text() == expected_hash
