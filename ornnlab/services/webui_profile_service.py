from __future__ import annotations

import json
import logging
from typing import Any
from uuid import uuid4

from ornnlab.models.webui import AgentInput, EnvironmentInput
from ornnlab.services.agent_capabilities import custom_agent_capabilities
from ornnlab.services.clock import now_iso
from ornnlab.settings import Settings
from ornnlab.storage import sqlite

logger = logging.getLogger(__name__)


class WebUiProfileService:
    def __init__(self, settings: Settings):
        self.settings = settings

    def list_agents(
        self, query: str | None = None, profile_type: str | None = None, status: str | None = None
    ) -> list[dict]:
        records_by_id = {record["id"]: record for record in self._built_in_agents()}
        records_by_id.update({record["id"]: record for record in self._configured_agents()})
        records = list(records_by_id.values())
        filtered = [record for record in records if _matches(record, query)]
        if profile_type:
            filtered = [record for record in filtered if record["type"] == profile_type]
        if status:
            filtered = [record for record in filtered if record["status"] == status]
        return sorted(
            filtered, key=lambda item: (item["type"] != "built-in", item["agentName"].lower())
        )

    def get_agent(self, agent_id: str) -> dict:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT config_json FROM agents WHERE id = ? AND status = 'active'",
                (agent_id,),
            )
        if rows:
            return _configured_agent(json.loads(rows[0]["config_json"]))
        for agent in self._built_in_agents():
            if agent["id"] == agent_id:
                return agent
        raise KeyError(agent_id)

    def resolve_agent(self, value: str) -> dict:
        for agent in self.list_agents():
            if value in {agent["id"], agent["agentName"], agent["harness"]}:
                return agent
        raise KeyError(value)

    def create_agent(self, payload: AgentInput) -> dict:
        if payload.type != "custom":
            raise PermissionError("built-in agents are provided by Harbor and cannot be created")
        self._assert_custom_agent_id(payload.id)
        agent = _agent_dto(payload)
        self._validate_agent(agent)
        self._write_agent_config(agent, create=True)
        return agent

    def update_agent(self, agent_id: str, payload: AgentInput) -> dict:
        existing = self.get_agent(agent_id)
        if payload.id != agent_id:
            raise ValueError("agent id cannot be changed")
        if payload.type != existing["type"]:
            raise ValueError("agent Harness source cannot be changed")
        if existing["type"] == "built-in" and payload.harness != existing["harness"]:
            raise ValueError("built-in Harness cannot be changed")
        agent = _agent_dto(payload)
        self._validate_agent(agent)
        self._write_agent_config(agent, create=not self._agent_is_persisted(agent_id))
        return agent

    def ensure_agent_persisted(self, agent: dict) -> dict:
        if not self._agent_is_persisted(agent["id"]):
            self._validate_agent(agent)
            self._write_agent_config(agent, create=True)
        return self.get_agent(agent["id"])

    def delete_agent(self, agent_id: str) -> None:
        self._assert_deletable_agent(agent_id)
        with sqlite.connect(self.settings) as conn:
            active_runs = conn.execute(
                "SELECT COUNT(*) FROM runs WHERE agent_id = ? AND status IN ('queued', 'running')",
                (agent_id,),
            ).fetchone()[0]
            if active_runs:
                raise RuntimeError("agent has queued or running runs")
            conn.execute(
                "UPDATE agents SET status = 'deleted', updated_at = ? WHERE id = ?",
                (now_iso(), agent_id),
            )

    def agent_harbor_config(self, agent: dict, model_name: str | None = None) -> dict:
        config: dict = {
            "model_name": model_name or _first(agent["models"]),
            "skills": agent["skillSources"],
            "env": _key_values(agent["env"]),
            "kwargs": _parse_kwargs(agent["kwargs"]),
        }
        if agent.get("setupTimeoutSeconds") is not None:
            config["override_setup_timeout_sec"] = agent["setupTimeoutSeconds"]
        if agent.get("timeoutSeconds") is not None:
            config["override_timeout_sec"] = agent["timeoutSeconds"]
        if agent.get("maxTimeoutSeconds") is not None:
            config["max_timeout_sec"] = agent["maxTimeoutSeconds"]
        if agent.get("importPath"):
            config["import_path"] = agent["importPath"]
        else:
            config["name"] = agent["harness"]
        mcp_servers = []
        for server in agent["mcpServers"]:
            mcp = {
                key: server[key]
                for key in ("name", "transport", "url", "command", "args")
                if server.get(key) not in (None, [])
            }
            if mcp["transport"] == "stdio" and not mcp.get("command"):
                raise ValueError(f"MCP server '{mcp['name']}' requires command for stdio")
            if mcp["transport"] != "stdio" and not mcp.get("url"):
                raise ValueError(f"MCP server '{mcp['name']}' requires URL for remote transport")
            mcp_servers.append(mcp)
        if mcp_servers:
            config["mcp_servers"] = mcp_servers
        return {key: value for key, value in config.items() if value not in (None, "", [], {})}

    def list_environments(
        self, query: str | None = None, profile_type: str | None = None
    ) -> list[dict]:
        records = self._built_in_environments() + self._custom_environments()
        filtered = [record for record in records if _matches(record, query)]
        if profile_type:
            filtered = [record for record in filtered if record["profileType"] == profile_type]
        return sorted(
            filtered, key=lambda item: (item["profileType"] != "built-in", item["name"].lower())
        )

    def get_environment(self, environment_id: str) -> dict:
        for environment in self._built_in_environments():
            if environment["id"] == environment_id:
                return environment
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT config_json FROM webui_environment_profiles "
                "WHERE id = ? AND deleted_at IS NULL",
                (environment_id,),
            )
        if not rows:
            raise KeyError(environment_id)
        return json.loads(rows[0]["config_json"])

    def create_environment(self, payload: EnvironmentInput) -> dict:
        if payload.profile_type != "custom":
            raise PermissionError("built-in environments cannot be created")
        environment = _environment_dto(payload)
        self._validate_environment(environment)
        now = now_iso()
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "INSERT INTO webui_environment_profiles("
                "id, name, profile_type, config_json, created_at, updated_at"
                ") "
                "VALUES (?, ?, ?, ?, ?, ?)",
                (
                    environment["id"],
                    environment["name"],
                    "custom",
                    json.dumps(environment),
                    now,
                    now,
                ),
            )
        return environment

    def update_environment(self, environment_id: str, payload: EnvironmentInput) -> dict:
        self._assert_mutable_environment(environment_id)
        if payload.id != environment_id:
            raise ValueError("environment id cannot be changed")
        environment = _environment_dto(payload)
        self._validate_environment(environment)
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE webui_environment_profiles SET name = ?, config_json = ?, updated_at = ? "
                "WHERE id = ? AND deleted_at IS NULL",
                (environment["name"], json.dumps(environment), now_iso(), environment_id),
            )
        return environment

    def copy_environment(self, environment_id: str) -> dict:
        source = self.get_environment(environment_id)
        copied = {
            **source,
            "id": f"env-{uuid4().hex[:8]}",
            "name": f"{source['name']} copy",
            "profileType": "custom",
        }
        return self.create_environment(EnvironmentInput.model_validate(copied))

    def delete_environment(self, environment_id: str) -> None:
        self._assert_mutable_environment(environment_id)
        with sqlite.connect(self.settings) as conn:
            conn.execute(
                "UPDATE webui_environment_profiles SET deleted_at = ?, updated_at = ? WHERE id = ?",
                (now_iso(), now_iso(), environment_id),
            )

    def environment_harbor_config(self, environment: dict) -> dict:
        config: dict = {
            "force_build": environment["forceBuild"],
            "delete": environment["deleteAfterRun"],
            "cpu_enforcement_policy": environment["cpuPolicy"],
            "memory_enforcement_policy": environment["memoryPolicy"],
            "override_cpus": _int_or_none(environment["overrideCpus"]),
            "override_memory_mb": _int_or_none(environment["overrideMemoryMb"]),
            "override_storage_mb": _int_or_none(environment["overrideStorageMb"]),
            "override_gpus": _int_or_none(environment["overrideGpus"]),
            "override_tpu": _tpu_or_none(environment["overrideTpu"]),
            "mounts": _json_or_none(environment["mounts"]),
            "extra_docker_compose": environment["dockerComposePaths"],
            "env": _key_values(environment["env"]),
            "kwargs": _parse_kwargs(environment["kwargs"]),
            "extra_allowed_hosts": environment["allowedHosts"],
        }
        if environment.get("importPath"):
            config["import_path"] = environment["importPath"]
        else:
            config["type"] = environment["environmentType"]
        return {key: value for key, value in config.items() if value not in (None, "", [], {})}

    def _write_agent_config(self, agent: dict, create: bool) -> None:
        now = now_iso()
        with sqlite.connect(self.settings) as conn:
            if create:
                conn.execute(
                    "INSERT INTO agents(id, name, harness, profile_type, status, config_json, "
                    "created_at, updated_at) VALUES (?, ?, ?, ?, 'active', ?, ?, ?)",
                    (
                        agent["id"],
                        agent["agentName"],
                        agent["harness"],
                        agent["type"],
                        json.dumps(agent),
                        now,
                        now,
                    ),
                )
            else:
                conn.execute(
                    "UPDATE agents SET name = ?, harness = ?, config_json = ?, status = 'active', "
                    "updated_at = ? WHERE id = ?",
                    (
                        agent["agentName"],
                        agent["harness"],
                        json.dumps(agent),
                        now,
                        agent["id"],
                    ),
                )
        logger.info(
            "Agent template persisted",
            extra={
                "agent_id": agent["id"],
                "harness": agent["harness"],
                "operation": "create" if create else "update",
            },
        )

    def _assert_custom_agent_id(self, agent_id: str) -> None:
        if agent_id.startswith("built-in:"):
            raise ValueError("agent id uses reserved built-in prefix")
        try:
            self.get_agent(agent_id)
        except KeyError:
            return
        raise ValueError("agent id already exists")

    def _assert_deletable_agent(self, agent_id: str) -> None:
        if agent_id.startswith("built-in:"):
            raise PermissionError("built-in Agent presets cannot be deleted")
        self.get_agent(agent_id)

    def _agent_is_persisted(self, agent_id: str) -> bool:
        with sqlite.connect(self.settings) as conn:
            return conn.execute(
                "SELECT 1 FROM agents WHERE id = ? AND status = 'active'", (agent_id,)
            ).fetchone() is not None

    def _assert_mutable_environment(self, environment_id: str) -> None:
        if environment_id.startswith("built-in:"):
            raise PermissionError("built-in Harbor environments are immutable")
        self.get_environment(environment_id)

    def _built_in_agents(self) -> list[dict]:
        from harbor.models.agent.name import AgentName

        return [_built_in_agent(name) for name in AgentName.values()]

    def _configured_agents(self) -> list[dict]:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT config_json FROM agents WHERE status = 'active'",
            )
        return [_configured_agent(json.loads(row["config_json"])) for row in rows]

    def _built_in_environments(self) -> list[dict]:
        return [_built_in_environment(item.value) for item in _environment_type()]

    def _custom_environments(self) -> list[dict]:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT config_json FROM webui_environment_profiles WHERE deleted_at IS NULL",
            )
        return [json.loads(row["config_json"]) for row in rows]

    def _validate_agent(self, agent: dict) -> None:
        from harbor.models.agent.name import AgentName
        from harbor.models.trial.config import AgentConfig

        if not agent.get("importPath") and agent["harness"] not in AgentName.values():
            raise ValueError("harness must be a built-in Harbor AgentName or use importPath")
        config = self.agent_harbor_config(agent)
        if "env" in config:
            config["env"] = {
                key: value for key, value in config["env"].items() if value is not None
            }
        AgentConfig.model_validate(config)

    def _validate_environment(self, environment: dict) -> None:
        from harbor.models.trial.config import EnvironmentConfig

        if not environment.get("importPath") and environment["environmentType"] not in {
            item.value for item in _environment_type()
        }:
            raise ValueError(
                "environmentType must be a Harbor EnvironmentType or importPath must be set"
            )
        valid_policies = {item.value for item in _resource_mode()}
        for field in ("cpuPolicy", "memoryPolicy"):
            if environment[field] not in valid_policies:
                raise ValueError(f"{field} must be one of {sorted(valid_policies)}")
        EnvironmentConfig.model_validate(self.environment_harbor_config(environment))


def _agent_dto(payload: AgentInput) -> dict:
    agent = payload.model_dump(by_alias=True, exclude_none=True)
    agent["env"] = [entry.model_dump() for entry in payload.env]
    return {
        **agent,
        "capabilities": custom_agent_capabilities(agent["harness"]),
        "status": "configured",
    }


def _configured_agent(agent: dict) -> dict:
    return {
        **agent,
        "capabilities": custom_agent_capabilities(agent["harness"]),
        "status": "configured",
    }


def _environment_dto(payload: EnvironmentInput) -> dict:
    environment = payload.model_dump(by_alias=True, exclude_none=True)
    environment["env"] = [entry.model_dump() for entry in payload.env]
    return environment


def _built_in_agent(harness: str) -> dict:
    return {
        # This is an OrnnLab Agent preset backed by a Harbor built-in Harness.
        # Saving it materializes an editable profile without modifying Harbor.
        "capabilities": custom_agent_capabilities(harness),
        "id": f"built-in:{harness}",
        "agentName": harness,
        "env": [],
        "harness": harness,
        "kwargs": "",
        "mcpServers": [],
        "models": [],
        "skillSources": [],
        "status": "available",
        "type": "built-in",
    }


def _built_in_environment(environment_type: str) -> dict:
    return {
        "id": f"built-in:{environment_type}",
        "name": environment_type,
        "profileType": "built-in",
        "environmentType": environment_type,
        "allowedHosts": [],
        "mounts": "",
        "env": [],
        "kwargs": "",
        "forceBuild": False,
        "deleteAfterRun": True,
        "cpuPolicy": "auto",
        "memoryPolicy": "auto",
        "overrideCpus": "",
        "overrideMemoryMb": "",
        "overrideStorageMb": "",
        "overrideGpus": "",
        "overrideTpu": "",
        "dockerComposePaths": [],
    }


def _matches(record: dict, query: str | None) -> bool:
    if not query:
        return True
    haystack = " ".join(str(value) for value in record.values()).lower()
    return query.lower() in haystack


def _key_values(values: list[dict[str, Any]]) -> dict[str, str]:
    resolved: dict[str, str] = {}
    for entry in values:
        key = str(entry["key"])
        value = entry.get("value")
        resolved[key] = str(value) if value is not None else f"${{{key}}}"
    return resolved


def _first(values: list[str]) -> str | None:
    return values[0] if values else None


def _parse_kwargs(value: str) -> dict:
    if not value.strip():
        return {}
    try:
        parsed = json.loads(value)
    except json.JSONDecodeError:
        return dict(entry.split("=", 1) for entry in value.splitlines() if "=" in entry)
    if not isinstance(parsed, dict):
        raise ValueError("kwargs must be a JSON object or key=value lines")
    return parsed


def _int_or_none(value: str) -> int | None:
    return int(value) if value.strip() and value.strip().lower() != "none" else None


def _tpu_or_none(value: str) -> dict | None:
    if not value.strip() or value.strip().lower() == "none":
        return None
    accelerator, separator, topology = value.partition("=")
    if not separator or not accelerator or not topology:
        raise ValueError("overrideTpu must use type=topology")
    return {"type": accelerator, "topology": topology}


def _json_or_none(value: str) -> object | None:
    if not value.strip() or value.strip().lower() == "none":
        return None
    return json.loads(value)


def _environment_type() -> Any:
    from harbor.models.environment_type import EnvironmentType

    return EnvironmentType


def _resource_mode() -> Any:
    from harbor.models.trial.config import ResourceMode

    return ResourceMode
