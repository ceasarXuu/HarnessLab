from __future__ import annotations

import asyncio

from ornnlab.services.event_service import EventService
from ornnlab.services.experiment_service import ExperimentService
from ornnlab.settings import Settings


class QueueWorkerService:
    def __init__(self, settings: Settings):
        self.settings = settings
        self._task: asyncio.Task[None] | None = None
        self._active_runs: dict[str, asyncio.Task[None]] = {}
        self._lock: asyncio.Lock | None = None
        self._lock_loop: asyncio.AbstractEventLoop | None = None
        self.last_error: str | None = None

    def start(self) -> None:
        if self.is_running:
            return
        loop = asyncio.get_running_loop()
        self._task = loop.create_task(self._run_background())

    @property
    def is_running(self) -> bool:
        return self._task is not None and not self._task.done()

    def cancel_run(self, run_id: str) -> bool:
        task = self._active_runs.get(run_id)
        if task is None or task.done():
            return False
        task.cancel()
        return True

    async def wait_until_idle(self) -> None:
        task = self._task
        if task is not None and not task.done() and task is not asyncio.current_task():
            await task
            return
        await self._drain()

    async def _run_background(self) -> None:
        try:
            await self._drain()
        except Exception as exc:
            self.last_error = f"{type(exc).__name__}: {exc}"
            raise

    async def _drain(self) -> None:
        async with self._loop_lock():
            event_service = EventService(self.settings)
            event_service.append("queue", "queue", "queue.worker_started", {})
            processed = await self._run_until_no_queued_runs()
            event_service.append("queue", "queue", "queue.worker_idle", {"processed": processed})

    async def _run_until_no_queued_runs(self) -> int:
        processed = 0
        while True:
            service = ExperimentService(self.settings)
            run = service.dequeue_next_run()
            if run is None:
                return processed
            processed += 1
            task = asyncio.create_task(service.execute_dequeued_run(run))
            self._active_runs[run["id"]] = task
            try:
                await task
            finally:
                self._active_runs.pop(run["id"], None)

    def _loop_lock(self) -> asyncio.Lock:
        loop = asyncio.get_running_loop()
        if self._lock is None or self._lock_loop is not loop:
            self._lock = asyncio.Lock()
            self._lock_loop = loop
        return self._lock
