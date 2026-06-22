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

    async def wait_experiment_terminal(
        self,
        experiment_id: str,
        poll_interval_sec: float = 0.2,
    ) -> None:
        TERMINAL_RUN_STATUSES = {"completed", "failed", "cancelled", "interrupted"}
        while True:
            state = ExperimentService(self.settings).get(experiment_id)
            statuses = {run["status"] for run in state["runs"]}
            if statuses and statuses.issubset(TERMINAL_RUN_STATUSES):
                return
            await asyncio.sleep(poll_interval_sec)

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

    async def _run_until_no_queued_runs(
        self,
        experiment_id: str | None = None,
        max_concurrent: int | None = None,
    ) -> int:
        processed = 0
        limit = max_concurrent or self.settings.worker_max_concurrent
        pending: set[asyncio.Task[None]] = set()
        dequeue_service = ExperimentService(self.settings)

        while True:
            done = {task for task in pending if task.done()}
            for task in done:
                pending.discard(task)
                run_id = task.get_name()
                self._active_runs.pop(run_id, None)
                self._consume_task_result(task)

            if len(pending) >= limit:
                done, _ = await asyncio.wait(pending, return_when=asyncio.FIRST_COMPLETED)
                for task in done:
                    pending.discard(task)
                    run_id = task.get_name()
                    self._active_runs.pop(run_id, None)
                    self._consume_task_result(task)
                continue

            run = dequeue_service.dequeue_next_run(experiment_id)
            if run is None:
                if pending:
                    done, _ = await asyncio.wait(pending, return_when=asyncio.ALL_COMPLETED)
                    for task in done:
                        pending.discard(task)
                        run_id = task.get_name()
                        self._active_runs.pop(run_id, None)
                        self._consume_task_result(task)
                return processed

            processed += 1
            task = asyncio.create_task(self._execute_run(run), name=run["id"])
            self._active_runs[run["id"]] = task
            pending.add(task)

    async def _execute_run(self, run: dict) -> None:
        service = ExperimentService(self.settings)
        await service.execute_dequeued_run(run)

    def _consume_task_result(self, task: asyncio.Task[None]) -> None:
        try:
            task.result()
        except asyncio.CancelledError:
            return
        except Exception as exc:
            self.last_error = f"{type(exc).__name__}: {exc}"

    def _loop_lock(self) -> asyncio.Lock:
        loop = asyncio.get_running_loop()
        if self._lock is None or self._lock_loop is not loop:
            self._lock = asyncio.Lock()
            self._lock_loop = loop
        return self._lock
