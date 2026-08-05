"""Dataset ref and directory safety primitives.

这些原语是 Dataset 文件系统边界的唯一操作入口：目录名净化、归属标记
（marker）写入与校验、受管目录删除，以及 ref 与路径的映射。删除或取消
清理只允许触碰带匹配 marker 的目录，路径穿越与符号链接跟随在此处被拒绝。
"""

from __future__ import annotations

import json
import os
import re
import shutil
from pathlib import Path

_MARKER_FILE = ".ornnlab-dataset.json"


def require_parent_directory(value: str) -> Path:
    path = Path(value).expanduser().resolve()
    if not path.is_dir():
        raise ValueError("dataset parent directory must exist")
    if not os.access(path, os.W_OK | os.X_OK):
        raise ValueError("dataset parent directory is not writable")
    return path


def require_existing_directory(value: str | None) -> Path:
    if not value:
        raise ValueError("dataset path is not available")
    path = Path(value).expanduser().resolve()
    if not path.is_dir():
        raise ValueError("dataset path must be an existing directory")
    return path


def managed_directory_name(ref: str) -> str:
    name = re.sub(r"[^A-Za-z0-9._@-]+", "-", ref.replace("/", "--")).strip(".-")
    if not name:
        raise ValueError("dataset reference cannot produce a directory name")
    return name


def write_marker(path: Path, ref: str) -> None:
    (path / _MARKER_FILE).write_text(json.dumps({"ref": ref}, sort_keys=True), encoding="utf-8")


def assert_managed_directory(
    path: Path, ref: str, legacy_root: Path, *, allow_legacy: bool = True
) -> None:
    marker = path / _MARKER_FILE
    if marker.is_file():
        try:
            if json.loads(marker.read_text(encoding="utf-8")).get("ref") == ref:
                return
        except json.JSONDecodeError:
            pass
    if allow_legacy and path.parent.resolve() == legacy_root.resolve():
        return
    raise ValueError("dataset directory is not managed by OrnnLab")


def remove_marked_directory(
    path: Path, ref: str, *, allow_legacy: bool, legacy_root: Path | None = None
) -> None:
    if not path.exists():
        return
    root = legacy_root or path.parent
    assert_managed_directory(path, ref, root, allow_legacy=allow_legacy)
    shutil.rmtree(path)


def split_ref(ref: str) -> tuple[str, str | None]:
    name, separator, version = ref.rpartition("@")
    return (name, version) if separator else (ref, None)


def join_ref(name: str, version: str | None) -> str:
    return f"{name}@{version}" if version else name
