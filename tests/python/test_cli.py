from __future__ import annotations

import json

from harnesslab.cli import main


def test_doctor_logs_flag_prints_logs_section(settings, capsys):
    assert main(["doctor", "--logs"]) == 0

    payload = json.loads(capsys.readouterr().out)

    assert "logs" in payload
    assert payload["logs"]["latest_failed_run"] is None


def test_backup_export_command_prints_archive_path(settings, capsys):
    assert main(["backup", "export"]) == 0

    payload = json.loads(capsys.readouterr().out)

    assert payload["archive_path"].endswith(".tar.gz")
    assert payload["file_count"] >= 0
