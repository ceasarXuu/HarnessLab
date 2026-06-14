from __future__ import annotations

import argparse
import json

import uvicorn

from harnesslab import __version__
from harnesslab.app import create_app
from harnesslab.services.doctor_service import DoctorService
from harnesslab.settings import Settings


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="harnesslab")
    parser.add_argument("--version", action="store_true", help="Print HarnessLab version")
    sub = parser.add_subparsers(dest="command")

    web = sub.add_parser("web", help="Start the local WebUI backend")
    web.add_argument("--host", default="127.0.0.1")
    web.add_argument("--port", type=int, default=8765)

    sub.add_parser("doctor", help="Print local system status")
    sub.add_parser("version", help="Print HarnessLab version")

    args = parser.parse_args(argv)
    if args.version or args.command == "version":
        print(__version__)
        return 0
    if args.command == "doctor":
        print(json.dumps(DoctorService(Settings.from_env()).status(), indent=2, sort_keys=True))
        return 0
    if args.command == "web":
        settings = Settings.from_env()
        app = create_app(settings)
        uvicorn.run(app, host=args.host, port=args.port, log_level="info")
        return 0
    parser.print_help()
    return 0
