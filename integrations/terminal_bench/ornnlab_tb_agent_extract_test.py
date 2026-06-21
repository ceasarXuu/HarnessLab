import unittest

import pytest

pytest.importorskip("terminal_bench.agents.base_agent")

from ornnlab_tb_agent import extract_shell_script


class OrnnLabShellExtractionTests(unittest.TestCase):
    def test_extract_shell_script_removes_common_agent_preamble(self):
        output = """I'll output the POSIX sh script directly:

apt-get update -y
mkdir -p ramfs
cat > ramfs/init << 'EOF'
#!/bin/sh
echo boot
EOF
"""

        script = extract_shell_script(output)

        self.assertTrue(script.startswith("apt-get update -y"))
        self.assertIn("cat > ramfs/init << 'EOF'", script)
        self.assertNotIn("POSIX sh script directly", script)

    def test_extract_shell_script_removes_preamble_before_control_flow(self):
        cases = [
            ("for item in a b; do\n  echo \"$item\"\ndone", "for item in a b; do"),
            ("if [ -f /etc/os-release ]; then\n  cat /etc/os-release\nfi", "if ["),
            ("while [ ! -f done ]; do\n  sleep 1\ndone", "while ["),
            ("case \"$ARCH\" in\n  x86_64) echo ok ;;\nesac", "case \"$ARCH\""),
        ]
        for body, expected_start in cases:
            with self.subTest(expected_start=expected_start):
                output = f"Here is the script:\n\n{body}\n"

                script = extract_shell_script(output)

                self.assertTrue(script.startswith(expected_start), script)
                self.assertNotIn("Here is the script", script)

    def test_extract_shell_script_removes_preamble_before_shell_launcher(self):
        for launcher in ("bash /tmp/run.sh", "sh ./setup.sh", "/bin/sh ./setup.sh"):
            with self.subTest(launcher=launcher):
                script = extract_shell_script(f"I will run this:\n{launcher}\n")

                self.assertEqual(script, launcher)

    def test_extract_shell_script_removes_preamble_before_assignment(self):
        script = extract_shell_script("Script follows:\nARCH=x86_64\nmake all\n")

        self.assertEqual(script, "ARCH=x86_64\nmake all")

    def test_extract_shell_script_preserves_comment_leading_script(self):
        output = """The script is below:
# prepare initramfs
set -e
echo ready
"""

        script = extract_shell_script(output)

        self.assertTrue(script.startswith("# prepare initramfs"))
        self.assertNotIn("script is below", script)

    def test_extract_shell_script_preserves_heredoc_body(self):
        output = """Here is the shell:
cat > /tmp/init << 'EOF'
#!/bin/sh
echo boot
EOF
chmod +x /tmp/init
"""

        script = extract_shell_script(output)

        self.assertIn("cat > /tmp/init << 'EOF'", script)
        self.assertIn("#!/bin/sh", script)
        self.assertTrue(script.endswith("chmod +x /tmp/init"))


if __name__ == "__main__":
    unittest.main()
