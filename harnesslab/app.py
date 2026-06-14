from __future__ import annotations

from fastapi import FastAPI

from harnesslab.api import agents, benchmarks, experiments, leaderboard, runs, system, templates
from harnesslab.settings import Settings
from harnesslab.storage import sqlite


def create_app(settings: Settings | None = None) -> FastAPI:
    active_settings = settings or Settings.from_env()
    active_settings.ensure_dirs()
    sqlite.initialize(active_settings)

    app = FastAPI(title="HarnessLab", version="0.2.0")
    app.state.settings = active_settings
    app.include_router(system.router)
    app.include_router(agents.router)
    app.include_router(benchmarks.router)
    app.include_router(experiments.router)
    app.include_router(runs.router)
    app.include_router(templates.router)
    app.include_router(leaderboard.router)
    return app
