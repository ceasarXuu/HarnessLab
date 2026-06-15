from __future__ import annotations

from pathlib import Path


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def relative_to_home(path: Path, home: Path) -> str:
    return path.resolve().relative_to(home.resolve()).as_posix()


def atomic_write_text(path: Path, body: str) -> None:
    ensure_parent(path)
    tmp = path.with_name(f".{path.name}.tmp")
    tmp.write_text(body, encoding="utf-8")
    tmp.replace(path)
