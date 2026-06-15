from __future__ import annotations

import argparse
import json
from pathlib import Path

import uvicorn

from ornnlab import __version__
from ornnlab.app import create_app
from ornnlab.services.backup_service import BackupService
from ornnlab.services.cleanup_service import CleanupService
from ornnlab.services.doctor_service import DoctorService
from ornnlab.settings import Settings


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="ornnlab")
    parser.add_argument("--version", action="store_true", help="Print OrnnLab version")
    sub = parser.add_subparsers(dest="command")

    web = sub.add_parser("web", help="Start the local WebUI backend")
    web.add_argument("--host", default="127.0.0.1")
    web.add_argument("--port", type=int, default=8765)

    doctor = sub.add_parser("doctor", help="Print local system status")
    doctor.add_argument("--logs", action="store_true", help="Include failed run log paths")
    backup = sub.add_parser("backup", help="Export or import the local OrnnLab home")
    backup_sub = backup.add_subparsers(dest="backup_command", required=True)
    backup_export = backup_sub.add_parser("export", help="Create a local backup archive")
    backup_export.add_argument("--output", help="Backup archive path")
    backup_import = backup_sub.add_parser("import", help="Import into an empty OrnnLab home")
    backup_import.add_argument("archive", help="Backup archive path")
    cleanup = sub.add_parser("cleanup", help="Plan or archive stale local artifacts")
    cleanup_sub = cleanup.add_subparsers(dest="cleanup_command", required=True)
    cleanup_sub.add_parser("plan", help="Print stale artifact cleanup candidates")
    cleanup_sub.add_parser("archive", help="Move cleanup candidates into local archive")
    sub.add_parser("version", help="Print OrnnLab version")

    args = parser.parse_args(argv)
    if args.version or args.command == "version":
        print(__version__)
        return 0
    if args.command == "doctor":
        print(
            json.dumps(
                DoctorService(Settings.from_env()).status(include_logs=args.logs),
                indent=2,
                sort_keys=True,
            )
        )
        return 0
    if args.command == "backup":
        service = BackupService(Settings.from_env())
        if args.backup_command == "export":
            result = service.export_home(Path(args.output) if args.output else None)
        else:
            result = service.import_home(Path(args.archive))
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0
    if args.command == "cleanup":
        service = CleanupService(Settings.from_env())
        result = service.plan() if args.cleanup_command == "plan" else service.archive()
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0
    if args.command == "web":
        settings = Settings.from_env()
        app = create_app(settings)
        uvicorn.run(app, host=args.host, port=args.port, log_level="info")
        return 0
    parser.print_help()
    return 0
