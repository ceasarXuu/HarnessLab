from __future__ import annotations

import argparse
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    parser.add_argument("--maximum", type=int, default=500)
    args = parser.parse_args()

    violations = [
        f"{path}:{len(path.read_text(encoding='utf-8').splitlines())}"
        for path in args.root.rglob("*.py")
        if len(path.read_text(encoding="utf-8").splitlines()) > args.maximum
    ]
    if violations:
        raise SystemExit("Files exceed the line-count limit:\n" + "\n".join(violations))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
