from __future__ import annotations

import os
import shlex


def split_command(command: str, *, windows: bool | None = None) -> list[str]:
    """Parse a configurable executable command without corrupting Windows paths."""
    is_windows = os.name == "nt" if windows is None else windows
    parts = shlex.split(command, posix=not is_windows)
    normalized = [_strip_wrapping_quotes(part) for part in parts]
    if not normalized:
        raise ValueError("command cannot be empty")
    return normalized


def _strip_wrapping_quotes(value: str) -> str:
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {"\"", "'"}:
        return value[1:-1]
    return value
