from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class LocalTaskPage:
    cache_hit: bool
    items: list[dict]
    next_cursor: str | None
    total: int


@dataclass(frozen=True)
class _TaskDirectoryIndex:
    directory_mtime_ns: int
    task_directories: tuple[Path, ...]


class LocalDatasetTaskCatalog:
    """Indexes Task directories cheaply and parses summaries one page at a time."""

    def __init__(self) -> None:
        self._indexes: dict[Path, _TaskDirectoryIndex] = {}

    def list_page(
        self,
        path: Path,
        dataset_ref: str,
        query: str | None,
        offset: int,
        limit: int,
    ) -> LocalTaskPage:
        task_directories, cache_hit = self._task_directories(path)
        if query:
            needle = query.casefold()
            task_directories = tuple(
                task_dir for task_dir in task_directories if needle in task_dir.name.casefold()
            )

        total = len(task_directories)
        selected = task_directories[offset : offset + limit]
        items = [
            {
                "datasetRef": dataset_ref,
                "description": "",
                "environment": None,
                "name": task_dir.name,
            }
            for task_dir in selected
        ]
        next_cursor = str(offset + limit) if offset + limit < total else None
        return LocalTaskPage(
            cache_hit=cache_hit,
            items=items,
            next_cursor=next_cursor,
            total=total,
        )

    def invalidate(self, path: Path | None = None) -> None:
        if path is None:
            self._indexes.clear()
            return
        self._indexes.pop(path.expanduser().resolve(), None)

    def _task_directories(self, path: Path) -> tuple[tuple[Path, ...], bool]:
        resolved = path.expanduser().resolve()
        if not resolved.is_dir():
            return (), False
        directory_mtime_ns = resolved.stat().st_mtime_ns
        cached = self._indexes.get(resolved)
        if cached and cached.directory_mtime_ns == directory_mtime_ns:
            return cached.task_directories, True

        task_directories = tuple(
            child
            for child in sorted(resolved.iterdir(), key=lambda item: item.name.casefold())
            if _is_task_directory_candidate(child)
        )
        self._indexes[resolved] = _TaskDirectoryIndex(
            directory_mtime_ns=directory_mtime_ns,
            task_directories=task_directories,
        )
        return task_directories, False


def page_offset(cursor: str | None, limit: int) -> int:
    if limit < 1 or limit > 100:
        raise ValueError("limit must be between 1 and 100")
    try:
        offset = int(cursor or "0")
    except ValueError as exc:
        raise ValueError("cursor must be a non-negative integer") from exc
    if offset < 0:
        raise ValueError("cursor must be a non-negative integer")
    return offset


def _is_task_directory_candidate(path: Path) -> bool:
    # Registry downloads and local imports are fully validated when registered.
    # The index only needs stable filesystem markers so the first page stays cheap.
    return path.is_dir() and (path / "task.toml").is_file() and (path / "environment").is_dir()
