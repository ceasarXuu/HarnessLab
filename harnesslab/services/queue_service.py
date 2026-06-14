from __future__ import annotations

from harnesslab.services.clock import now_iso
from harnesslab.settings import Settings
from harnesslab.storage import sqlite


class QueueService:
    def __init__(self, settings: Settings):
        self.settings = settings

    def enqueue_experiment(self, experiment_id: str) -> list[dict]:
        now = now_iso()
        with sqlite.connect(self.settings) as conn:
            runs = sqlite.rows(
                conn,
                "SELECT * FROM runs WHERE experiment_id = ? AND status = 'draft' "
                "ORDER BY run_order",
                (experiment_id,),
            )
            next_position = self._next_position(conn)
            for offset, run in enumerate(runs):
                position = next_position + offset
                conn.execute(
                    "INSERT OR REPLACE INTO queue_items("
                    "run_id, queue_position, state, enqueued_at"
                    ") VALUES (?, ?, ?, ?)",
                    (run["id"], position, "queued", now),
                )
                conn.execute(
                    "UPDATE runs SET status = ?, updated_at = ? WHERE id = ?",
                    ("queued", now, run["id"]),
                )
            if runs:
                conn.execute(
                    "UPDATE experiments SET status = ?, updated_at = ? WHERE id = ?",
                    ("queued", now, experiment_id),
                )
        return self.list_queue()

    def dequeue_next(self, experiment_id: str | None = None) -> dict | None:
        now = now_iso()
        with sqlite.connect(self.settings) as conn:
            experiment_filter = ""
            params: tuple[str, ...] = ()
            if experiment_id is not None:
                experiment_filter = "AND r.experiment_id = ?"
                params = (experiment_id,)
            queued = sqlite.rows(
                conn,
                "SELECT r.* FROM queue_items q JOIN runs r ON r.id = q.run_id "
                f"WHERE q.state = 'queued' {experiment_filter} "
                "ORDER BY q.queue_position LIMIT 1",
                params,
            )
            if not queued:
                return None
            run = queued[0]
            conn.execute(
                "UPDATE queue_items SET state = ?, dequeued_at = ? WHERE run_id = ?",
                ("running", now, run["id"]),
            )
            return run

    def finish(self, run_id: str, state: str) -> None:
        now = now_iso()
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE queue_items SET state = ?, finished_at = ? WHERE run_id = ?",
                (state, now, run_id),
            )

    def list_queue(self) -> list[dict]:
        with sqlite.connect(self.settings) as conn:
            return sqlite.rows(conn, "SELECT * FROM queue_items ORDER BY queue_position")

    @staticmethod
    def _next_position(conn) -> int:
        row = conn.execute("SELECT COALESCE(MAX(queue_position), 0) + 1 AS next FROM queue_items")
        return int(row.fetchone()["next"])
