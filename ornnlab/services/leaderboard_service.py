from __future__ import annotations

from ornnlab.settings import Settings
from ornnlab.storage import sqlite


class LeaderboardService:
    def __init__(self, settings: Settings):
        self.settings = settings

    def list(self, benchmark: str | None = None) -> list[dict]:
        query = (
            "SELECT id, agent_id, benchmark_name, benchmark_version, split, finished_at, score, "
            "comparability_key, report_path FROM runs "
            "WHERE status = 'completed' AND leaderboard_eligible = 1"
        )
        params: tuple[str, ...] = ()
        if benchmark:
            query += " AND benchmark_name = ?"
            params = (benchmark,)
        query += " ORDER BY score DESC, finished_at DESC"
        with sqlite.connect(self.settings) as conn:
            return sqlite.rows(conn, query, params)
