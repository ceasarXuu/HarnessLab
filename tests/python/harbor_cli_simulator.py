from __future__ import annotations

import argparse
import json
import signal
import time
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("run")
    parser.add_argument("--config", required=True)
    args = parser.parse_args()
    if args.run != "run":
        raise SystemExit(f"unsupported simulator command: {args.run}")

    config_path = Path(args.config)
    config = json.loads(config_path.read_text(encoding="utf-8"))
    job_dir = Path(config["jobs_dir"])
    job_dir.mkdir(parents=True, exist_ok=True)
    dataset = config["datasets"][0]["name"]

    if dataset == "simulated-docker-failure":
        print("docker compose returned code -9", flush=True)
        return 9
    if dataset == "simulated-slow-cancel":
        _run_until_terminated(job_dir)
        return 0

    result = {
        "status": "completed",
        "score": 1.0,
        "job_dir": str(job_dir),
        "result_path": str(job_dir / "result.json"),
    }
    (job_dir / "result.json").write_text(
        json.dumps(result, indent=2, sort_keys=True),
        encoding="utf-8",
    )
    print("simulated harbor completed", flush=True)
    return 0


def _run_until_terminated(job_dir: Path) -> None:
    marker = job_dir / "simulator.terminated"

    def handle_term(signum, frame) -> None:
        marker.write_text(str(signum), encoding="utf-8")
        raise SystemExit(0)

    signal.signal(signal.SIGTERM, handle_term)
    while True:
        time.sleep(0.1)


if __name__ == "__main__":
    raise SystemExit(main())
