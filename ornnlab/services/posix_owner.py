from __future__ import annotations

import os


def current_posix_uid_gid() -> tuple[int, int]:
    """Return the current POSIX uid/gid, or (-1, -1) on hosts without them."""
    getuid = getattr(os, "getuid", None)
    getgid = getattr(os, "getgid", None)
    if getuid is None or getgid is None:
        return (-1, -1)
    return (getuid(), getgid())
