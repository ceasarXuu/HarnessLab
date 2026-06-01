import os
import re
import shlex
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from terminal_bench.agents.failure_mode import FailureMode

from harnesslab_tb_process import AgentCommandTimedOut, CleanupResult

from harnesslab_tb_agent import (
    HarnessLabCommandAgent,
    extract_shell_script,
    run_registered_agent,
    shell_syntax_error,
)


class FakeSession:
    def __init__(self, container=None, session_name="task-session", user=""):
        self.commands = []
        self.container = container
        self._session_name = session_name
        self._user = user

    def send_command(self, command):
        self.commands.append(command)


class FakeExecResult:
    def __init__(self, exit_code, output=""):
        self.exit_code = exit_code
        self.output = output.encode()


class FakeContainer:
    def __init__(self, shell="/bin/sh"):
        self.shell = shell
        self.commands = []
        self.users = []

    def exec_run(self, command, user=""):
        self.commands.append(command)
        self.users.append(user)
        if command == [
            "tmux",
            "show-options",
            "-t",
            "task-session",
            "-v",
            "default-shell",
        ]:
            return FakeExecResult(0, self.shell + "\n")
        completed = subprocess.run(command, capture_output=True, text=True, check=False)
        return FakeExecResult(
            completed.returncode,
            completed.stdout + completed.stderr,
        )


def agent_print_command(output):
    return f"python -c {shlex.quote(f'print({output!r})')}"


class HarnessLabCommandAgentTests(unittest.TestCase):
    def test_stdin_mode_passes_prompt_to_subprocess_stdin(self):
        output = run_registered_agent(
            "python -c 'import sys; print(sys.stdin.read())'",
            "stdin",
            "hello",
            5,
        )

        self.assertEqual(output.strip(), "hello")

    def test_argument_mode_substitutes_instruction_argument(self):
        command = "python -c 'import sys; print(sys.argv[1])' {{instruction}}"

        output = run_registered_agent(command, "argument", "hello there", 5)

        self.assertEqual(output.strip(), "hello there")

    def test_file_mode_substitutes_instruction_file_and_removes_it(self):
        with tempfile.TemporaryDirectory() as tmp:
            record = Path(tmp) / "path.txt"
            code = (
                "import pathlib, sys; "
                "pathlib.Path(sys.argv[2]).write_text(sys.argv[1]); "
                "print(pathlib.Path(sys.argv[1]).read_text())"
            )
            command = f"python -c {shlex.quote(code)} {{{{instruction_file}}}} {record}"

            output = run_registered_agent(command, "file", "hello file", 5)
            prompt_path = Path(record.read_text())

        self.assertEqual(output.strip(), "hello file")
        self.assertFalse(prompt_path.exists())

    def test_extract_shell_script_accepts_fenced_and_plain_output(self):
        self.assertEqual(extract_shell_script("```bash\necho ok\n```"), "echo ok")
        self.assertEqual(extract_shell_script("echo ok\n"), "echo ok")

    def test_shell_syntax_error_rejects_natural_language_with_parentheses(self):
        self.assertIsNone(shell_syntax_error("echo ok\n"))
        self.assertIn(
            "syntax error",
            shell_syntax_error(
                "All 11 tests passed (file existence check + 10 maze content checks)."
            ).lower(),
        )

    def test_shell_syntax_error_reports_missing_shell(self):
        with patch("harnesslab_tb_agent.subprocess.run", side_effect=OSError("missing")):
            self.assertIn("missing", shell_syntax_error("echo ok"))

    def test_perform_task_maps_nonzero_agent_to_unknown_agent_error(self):
        agent = HarnessLabCommandAgent()
        with patch.dict(
            os.environ,
            {
                "HARNESSLAB_AGENT_COMMAND": "python -c 'import sys; sys.exit(7)'",
                "HARNESSLAB_AGENT_INPUT_MODE": "stdin",
                "HARNESSLAB_AGENT_TIMEOUT_SEC": "5",
            },
            clear=False,
        ):
            result = agent.perform_task("do it", FakeSession())

        self.assertEqual(result.failure_mode, FailureMode.UNKNOWN_AGENT_ERROR)

    def test_perform_task_maps_empty_agent_output_to_parse_error(self):
        agent = HarnessLabCommandAgent()
        with patch.dict(
            os.environ,
            {
                "HARNESSLAB_AGENT_COMMAND": "python -c 'print()'",
                "HARNESSLAB_AGENT_INPUT_MODE": "stdin",
                "HARNESSLAB_AGENT_TIMEOUT_SEC": "5",
            },
            clear=False,
        ):
            result = agent.perform_task("do it", FakeSession())

        self.assertEqual(result.failure_mode, FailureMode.PARSE_ERROR)

    def test_perform_task_maps_agent_command_timeout_to_agent_timeout(self):
        agent = HarnessLabCommandAgent()
        command = (
            "python -c "
            + shlex.quote(
                "import time; print('partial-output', flush=True); time.sleep(60)"
            )
        )
        with tempfile.TemporaryDirectory() as tmp:
            with patch.dict(
                os.environ,
                {
                    "HARNESSLAB_AGENT_COMMAND": command,
                    "HARNESSLAB_AGENT_INPUT_MODE": "stdin",
                    "HARNESSLAB_AGENT_TIMEOUT_SEC": "1",
                },
                clear=False,
            ):
                result = agent.perform_task("do it", FakeSession(), Path(tmp))

            log_dir = Path(tmp)
            error_text = (log_dir / "agent_error.log").read_text()
            stdout_text = (log_dir / "agent_stdout_partial.log").read_text()
            prompt_text = (log_dir / "prompt.txt").read_text()

        self.assertEqual(result.failure_mode, FailureMode.AGENT_TIMEOUT)
        self.assertIn("timed out", error_text)
        self.assertIn("configured_timeout", error_text)
        self.assertIn("succeeded=True", error_text)
        self.assertIn("partial-output", stdout_text)
        self.assertIn("Task instruction:", prompt_text)

    def test_perform_task_maps_failed_cleanup_to_unknown_agent_error(self):
        agent = HarnessLabCommandAgent()
        cleanup = CleanupResult(
            root_pid=123,
            pids={123},
            pgids={123},
            token="token",
            token_survivors={456},
            alive_pids=set(),
        )
        timeout_error = AgentCommandTimedOut(1, cleanup, "out", "err")
        with tempfile.TemporaryDirectory() as tmp:
            with patch.dict(
                os.environ,
                {
                    "HARNESSLAB_AGENT_COMMAND": "agent",
                    "HARNESSLAB_AGENT_INPUT_MODE": "stdin",
                    "HARNESSLAB_AGENT_TIMEOUT_SEC": "1",
                },
                clear=False,
            ), patch(
                "harnesslab_tb_agent.run_registered_agent",
                side_effect=timeout_error,
            ):
                result = agent.perform_task("do it", FakeSession(), Path(tmp))
            error_text = (Path(tmp) / "agent_error.log").read_text()

        self.assertEqual(result.failure_mode, FailureMode.UNKNOWN_AGENT_ERROR)
        self.assertIn("token_survivors=[456]", error_text)

    def test_perform_task_maps_invalid_shell_output_to_parse_error(self):
        agent = HarnessLabCommandAgent()
        session = FakeSession()
        output = "All 11 tests passed (file existence check + 10 maze content checks)."
        with tempfile.TemporaryDirectory() as tmp:
            with patch.dict(
                os.environ,
                {
                    "HARNESSLAB_AGENT_COMMAND": agent_print_command(output),
                    "HARNESSLAB_AGENT_INPUT_MODE": "stdin",
                    "HARNESSLAB_AGENT_TIMEOUT_SEC": "5",
                },
                clear=False,
            ):
                result = agent.perform_task("do it", session, Path(tmp))

            log_dir = Path(tmp)
            syntax_text = (log_dir / "script_syntax_error.log").read_text()
            output_text = (log_dir / "agent_output.txt").read_text()
            script_text = (log_dir / "container_script.sh").read_text()
            shell_text = (log_dir / "execution_shell.txt").read_text()

        self.assertEqual(result.failure_mode, FailureMode.PARSE_ERROR)
        self.assertEqual(session.commands, [])
        self.assertIn("syntax error", syntax_text.lower())
        self.assertEqual(output_text.strip(), output)
        self.assertEqual(script_text.strip(), output)
        self.assertEqual(shell_text.strip(), "/bin/sh")

    def test_perform_task_uses_container_tmux_shell_for_syntax_check(self):
        agent = HarnessLabCommandAgent()
        script = "arr=(a b)\necho ${arr[0]}"
        with tempfile.TemporaryDirectory() as tmp:
            fake_shell = Path(tmp) / "fake-task-shell"
            fake_shell.write_text(
                "#!/usr/bin/env python3\n"
                "import pathlib, sys\n"
                "if sys.argv[1] == '-n':\n"
                "    script = pathlib.Path(sys.argv[2]).read_text()\n"
                "    if 'arr=(' in script:\n"
                "        print('fake task shell syntax error', file=sys.stderr)\n"
                "        sys.exit(2)\n"
                "sys.exit(0)\n"
            )
            fake_shell.chmod(0o755)
            session = FakeSession(FakeContainer(str(fake_shell)), user="agent-user")
            with patch.dict(
                os.environ,
                {
                    "HARNESSLAB_AGENT_COMMAND": agent_print_command(script),
                    "HARNESSLAB_AGENT_INPUT_MODE": "stdin",
                    "HARNESSLAB_AGENT_TIMEOUT_SEC": "5",
                },
                clear=False,
            ):
                result = agent.perform_task("do it", session, Path(tmp))

            syntax_text = (Path(tmp) / "script_syntax_error.log").read_text()
            shell_text = (Path(tmp) / "execution_shell.txt").read_text()

        self.assertEqual(result.failure_mode, FailureMode.PARSE_ERROR)
        self.assertEqual(session.commands, [])
        self.assertIn("fake task shell syntax error", syntax_text)
        self.assertEqual(shell_text.strip(), str(fake_shell))
        self.assertTrue(
            any(
                command
                == [
                    "tmux",
                    "show-options",
                    "-t",
                    "task-session",
                    "-v",
                    "default-shell",
                ]
                for command in session.container.commands
            )
        )
        self.assertTrue(session.container.users)
        self.assertTrue(all(user == "agent-user" for user in session.container.users))

    def test_perform_task_logs_shell_lookup_fallback_reason(self):
        agent = HarnessLabCommandAgent()
        session = FakeSession(container=None)
        with tempfile.TemporaryDirectory() as tmp:
            with patch.dict(
                os.environ,
                {
                    "HARNESSLAB_AGENT_COMMAND": agent_print_command("echo ok"),
                    "HARNESSLAB_AGENT_INPUT_MODE": "stdin",
                    "HARNESSLAB_AGENT_TIMEOUT_SEC": "5",
                },
                clear=False,
            ):
                result = agent.perform_task("do it", session, Path(tmp))

            resolution_text = (Path(tmp) / "execution_shell_resolution.log").read_text()

        self.assertEqual(result.failure_mode, FailureMode.NONE)
        self.assertIn("no terminal container", resolution_text)

    def test_perform_task_rejects_fenced_invalid_shell_before_tmux(self):
        agent = HarnessLabCommandAgent()
        session = FakeSession()
        script = "All 11 tests passed (file existence check + 10 maze content checks)."
        output = f"```sh\n{script}\n```"
        with tempfile.TemporaryDirectory() as tmp:
            with patch.dict(
                os.environ,
                {
                    "HARNESSLAB_AGENT_COMMAND": agent_print_command(output),
                    "HARNESSLAB_AGENT_INPUT_MODE": "stdin",
                    "HARNESSLAB_AGENT_TIMEOUT_SEC": "5",
                },
                clear=False,
            ):
                result = agent.perform_task("do it", session, Path(tmp))

            log_dir = Path(tmp)
            syntax_text = (log_dir / "script_syntax_error.log").read_text()
            output_text = (log_dir / "agent_output.txt").read_text()
            script_text = (log_dir / "container_script.sh").read_text()
            shell_text = (log_dir / "execution_shell.txt").read_text()

        self.assertEqual(result.failure_mode, FailureMode.PARSE_ERROR)
        self.assertEqual(session.commands, [])
        self.assertIn("syntax error", syntax_text.lower())
        self.assertEqual(output_text.strip(), output)
        self.assertEqual(script_text.strip(), script)
        self.assertEqual(shell_text.strip(), "/bin/sh")

    def test_perform_task_sends_valid_multiline_script_to_terminal_session(self):
        agent = HarnessLabCommandAgent()
        session = FakeSession()
        script = "set -eu\necho ok\nprintf '%s\\n' done"
        with patch.dict(
            os.environ,
            {
                "HARNESSLAB_AGENT_COMMAND": agent_print_command(f"```sh\n{script}\n```"),
                "HARNESSLAB_AGENT_INPUT_MODE": "stdin",
                "HARNESSLAB_AGENT_TIMEOUT_SEC": "5",
            },
            clear=False,
        ):
            result = agent.perform_task("do it", session)

        self.assertEqual(result.failure_mode, FailureMode.NONE)
        self.assertIn("set -eu\necho ok", session.commands[0].command)
        self.assertIn("printf", session.commands[0].command)
        self.assertTrue(session.commands[0].command.startswith("/bin/sh -lc "))

    def test_perform_task_uses_resolved_shell_in_execution_wrapper(self):
        agent = HarnessLabCommandAgent()
        with tempfile.TemporaryDirectory() as tmp:
            shell_dir = Path(tmp) / "task shell dir"
            shell_dir.mkdir()
            fake_shell = shell_dir / "fake task shell"
            fake_shell.write_text(
                "#!/usr/bin/env python3\n"
                "import sys\n"
                "if sys.argv[1] == '-n':\n"
                "    sys.exit(0)\n"
                "raise SystemExit('unexpected execution during test')\n"
            )
            fake_shell.chmod(0o755)
            session = FakeSession(FakeContainer(str(fake_shell)))
            with patch.dict(
                os.environ,
                {
                    "HARNESSLAB_AGENT_COMMAND": agent_print_command("echo ok"),
                    "HARNESSLAB_AGENT_INPUT_MODE": "stdin",
                    "HARNESSLAB_AGENT_TIMEOUT_SEC": "5",
                },
                clear=False,
            ):
                result = agent.perform_task("do it", session, Path(tmp))

            command = session.commands[0].command
            wrapper = shlex.split(command)
            shell_text = (Path(tmp) / "execution_shell.txt").read_text()

        self.assertEqual(result.failure_mode, FailureMode.NONE)
        self.assertEqual(shell_text.strip(), str(fake_shell))
        self.assertEqual(wrapper[:2], ["/bin/sh", "-lc"])
        quoted_shell = shlex.quote(str(fake_shell))
        execution_lines = [
            line for line in wrapper[2].splitlines() if line.startswith(quoted_shell)
        ]
        self.assertEqual(len(execution_lines), 1)
        self.assertRegex(
            execution_lines[0],
            rf"^{re.escape(quoted_shell)} /tmp/harnesslab-agent-run-[0-9a-f]+\.sh$",
        )
        self.assertTrue(command.startswith("/bin/sh -lc "))

    def test_perform_task_sends_agent_script_to_terminal_session(self):
        agent = HarnessLabCommandAgent()
        session = FakeSession()
        with patch.dict(
            os.environ,
            {
                "HARNESSLAB_AGENT_COMMAND": "python -c 'print(\"echo ok\")'",
                "HARNESSLAB_AGENT_INPUT_MODE": "stdin",
                "HARNESSLAB_AGENT_TIMEOUT_SEC": "5",
            },
            clear=False,
        ):
            result = agent.perform_task("do it", session)

        self.assertEqual(result.failure_mode, FailureMode.NONE)
        self.assertIn("echo ok", session.commands[0].command)
        self.assertTrue(session.commands[0].command.startswith("/bin/sh -lc "))
        self.assertEqual(session.commands[0].max_timeout_sec, 5.0)


if __name__ == "__main__":
    unittest.main()
