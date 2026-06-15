from __future__ import annotations

import json

from harnesslab.cli import main


def test_doctor_logs_flag_prints_logs_section(settings, capsys):
    assert main(["doctor", "--logs"]) == 0

    payload = json.loads(capsys.readouterr().out)

    assert "logs" in payload
    assert payload["logs"]["latest_failed_run"] is None
