import os
import shlex
import signal
import subprocess
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

import pytest

pytest.importorskip("terminal_bench.agents.base_agent")

from harnesslab_tb_agent import run_registered_agent
from harnesslab_tb_process import AgentCommandTimedOut


def wait_for_process_exit(pid, timeout=5.0):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if not process_is_running(pid):
            return True
        time.sleep(0.05)
    return not process_is_running(pid)


def process_is_running(pid):
    completed = subprocess.run(
        ["ps", "-p", str(pid), "-o", "stat="],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    if completed.returncode != 0:
        return False
    state = completed.stdout.strip()
    return bool(state) and not state.startswith("Z")


def terminate_test_process(pid):
    for sig in (signal.SIGTERM, signal.SIGKILL):
        try:
            subprocess.run(
                ["kill", f"-{sig.value}", str(pid)],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
            )
        except OSError:
            pass
        if wait_for_process_exit(pid, 0.5):
            return


def wait_for_file(path, timeout=5.0):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.exists():
            return True
        time.sleep(0.05)
    return path.exists()


class HarnessLabCommandProcessTests(unittest.TestCase):
    def test_agent_process_uses_full_configured_timeout_budget(self):
        code = "import time; time.sleep(7); print('finished')"

        output = run_registered_agent(f"python -c {shlex.quote(code)}", "stdin", "", 11)

        self.assertEqual(output.strip(), "finished")

    def test_timeout_kills_detached_descendant_process_group(self):
        with tempfile.TemporaryDirectory() as tmp:
            child_pid_file = Path(tmp) / "child.pid"
            child_code = (
                "import os, pathlib, sys, time; "
                "pathlib.Path(sys.argv[1]).write_text(str(os.getpid())); "
                "time.sleep(60)"
            )
            parent_code = (
                "import pathlib, subprocess, sys, time\n"
                f"pid_file = pathlib.Path({str(child_pid_file)!r})\n"
                "subprocess.Popen(\n"
                f"    [sys.executable, '-c', {child_code!r}, str(pid_file)],\n"
                "    start_new_session=True,\n"
                ")\n"
                "deadline = time.time() + 5\n"
                "while not pid_file.exists() and time.time() < deadline:\n"
                "    time.sleep(0.01)\n"
                "time.sleep(60)"
            )
            command = f"python -c {shlex.quote(parent_code)}"

            with self.assertRaises(AgentCommandTimedOut) as raised:
                run_registered_agent(command, "stdin", "ignored", 1)
            self.assertTrue(child_pid_file.exists())
            child_pid = int(child_pid_file.read_text())

        self.assertIn("timed out", str(raised.exception))
        self.assertIn("succeeded=True", str(raised.exception))
        self.assertTrue(
            wait_for_process_exit(child_pid),
            f"detached child process {child_pid} was not cleaned up",
        )

    def test_timeout_kills_reparented_env_tokened_descendant_process_group(self):
        with tempfile.TemporaryDirectory() as tmp:
            child_pid_file = Path(tmp) / "daemon.pid"
            grandchild_code = (
                "import os, pathlib, sys, time; "
                "pathlib.Path(sys.argv[1]).write_text(str(os.getpid())); "
                "time.sleep(60)"
            )
            parent_code = (
                "import os, pathlib, subprocess, sys, time\n"
                f"pid_file = pathlib.Path({str(child_pid_file)!r})\n"
                "subprocess.Popen(\n"
                "    [sys.executable, '-c', "
                f"{grandchild_code!r}, str(pid_file)],\n"
                "    start_new_session=True,\n"
                "    stdin=subprocess.DEVNULL,\n"
                "    stdout=subprocess.DEVNULL,\n"
                "    stderr=subprocess.DEVNULL,\n"
                ")\n"
                "deadline = time.time() + 5\n"
                "while not pid_file.exists() and time.time() < deadline:\n"
                "    time.sleep(0.01)\n"
                "sys.exit(0)\n"
            )
            command = f"python -c {shlex.quote(parent_code)} && sleep 60"

            with self.assertRaises(AgentCommandTimedOut) as raised:
                run_registered_agent(command, "stdin", "ignored", 1)
            self.assertTrue(wait_for_file(child_pid_file))
            child_pid = int(child_pid_file.read_text())

        self.assertIn("token_survivors=[]", str(raised.exception))
        self.assertTrue(
            wait_for_process_exit(child_pid),
            f"reparented child process {child_pid} was not cleaned up",
        )

    def test_success_kills_reparented_env_tokened_descendant_process_group(self):
        with tempfile.TemporaryDirectory() as tmp:
            child_pid_file = Path(tmp) / "daemon.pid"
            grandchild_code = (
                "import os, pathlib, sys, time; "
                "pathlib.Path(sys.argv[1]).write_text(str(os.getpid())); "
                "time.sleep(60)"
            )
            parent_code = (
                "import pathlib, subprocess, sys, time\n"
                f"pid_file = pathlib.Path({str(child_pid_file)!r})\n"
                "subprocess.Popen(\n"
                "    [sys.executable, '-c', "
                f"{grandchild_code!r}, str(pid_file)],\n"
                "    start_new_session=True,\n"
                "    stdin=subprocess.DEVNULL,\n"
                "    stdout=subprocess.DEVNULL,\n"
                "    stderr=subprocess.DEVNULL,\n"
                ")\n"
                "deadline = time.time() + 5\n"
                "while not pid_file.exists() and time.time() < deadline:\n"
                "    time.sleep(0.01)\n"
                "print('ok')\n"
            )
            command = f"python -c {shlex.quote(parent_code)}"

            output = run_registered_agent(command, "stdin", "ignored", 5)
            self.assertTrue(wait_for_file(child_pid_file))
            child_pid = int(child_pid_file.read_text())

        self.assertEqual(output.strip(), "ok")
        self.assertTrue(
            wait_for_process_exit(child_pid),
            f"successful command left child process {child_pid} running",
        )

    def test_success_kills_reparented_descendant_that_clears_run_token(self):
        with tempfile.TemporaryDirectory() as tmp:
            child_pid_file = Path(tmp) / "daemon.pid"
            grandchild_code = (
                "import os, pathlib, sys, time; "
                "pathlib.Path(sys.argv[1]).write_text(str(os.getpid())); "
                "time.sleep(60)"
            )
            parent_code = (
                "import os, pathlib, subprocess, sys, time\n"
                f"pid_file = pathlib.Path({str(child_pid_file)!r})\n"
                "env = os.environ.copy()\n"
                "env.pop('HARNESSLAB_AGENT_RUN_TOKEN', None)\n"
                "subprocess.Popen(\n"
                "    [sys.executable, '-c', "
                f"{grandchild_code!r}, str(pid_file)],\n"
                "    start_new_session=True,\n"
                "    stdin=subprocess.DEVNULL,\n"
                "    stdout=subprocess.DEVNULL,\n"
                "    stderr=subprocess.DEVNULL,\n"
                "    env=env,\n"
                ")\n"
                "deadline = time.time() + 5\n"
                "while not pid_file.exists() and time.time() < deadline:\n"
                "    time.sleep(0.01)\n"
                "print('ok')\n"
            )
            command = f"python -c {shlex.quote(parent_code)}"

            with patch.dict(
                os.environ,
                {"HARNESSLAB_AGENT_STRICT_GLOBAL_PROCESS_SCAN": "1"},
            ):
                with self.assertRaises(RuntimeError) as raised:
                    run_registered_agent(command, "stdin", "ignored", 5)
            self.assertTrue(wait_for_file(child_pid_file))
            child_pid = int(child_pid_file.read_text())

        self.assertIn("left live child processes", str(raised.exception))
        terminate_test_process(child_pid)
        self.assertTrue(
            wait_for_process_exit(child_pid),
            f"token-cleared child process {child_pid} was not cleaned up by test",
        )

    def test_argument_mode_timeout_uses_same_cleanup_path(self):
        with tempfile.TemporaryDirectory() as tmp:
            record = Path(tmp) / "argument.txt"
            code = (
                "import pathlib, sys, time; "
                "pathlib.Path(sys.argv[1]).write_text(sys.argv[2]); "
                "time.sleep(60)"
            )
            command = f"python -c {shlex.quote(code)} {record} {{{{instruction}}}}"

            with self.assertRaises(AgentCommandTimedOut):
                run_registered_agent(command, "argument", "hello arg", 1)

            self.assertEqual(record.read_text(), "hello arg")

    def test_file_mode_timeout_removes_instruction_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            record = Path(tmp) / "prompt-path.txt"
            code = (
                "import pathlib, sys, time; "
                "pathlib.Path(sys.argv[2]).write_text(sys.argv[1]); "
                "time.sleep(60)"
            )
            command = f"python -c {shlex.quote(code)} {{{{instruction_file}}}} {record}"

            with self.assertRaises(AgentCommandTimedOut):
                run_registered_agent(command, "file", "hello file", 1)
            prompt_path = Path(record.read_text())

        self.assertFalse(prompt_path.exists())


if __name__ == "__main__":
    unittest.main()
