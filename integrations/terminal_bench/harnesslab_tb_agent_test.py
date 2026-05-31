import os
import shlex
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from terminal_bench.agents.failure_mode import FailureMode

from harnesslab_tb_agent import (
    HarnessLabCommandAgent,
    extract_shell_script,
    run_registered_agent,
)


class FakeSession:
    def __init__(self):
        self.commands = []

    def send_command(self, command):
        self.commands.append(command)


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
        self.assertEqual(session.commands[0].command, "echo ok")
        self.assertEqual(session.commands[0].max_timeout_sec, 5.0)


if __name__ == "__main__":
    unittest.main()
