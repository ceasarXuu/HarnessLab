from __future__ import annotations

import hashlib
import json
import re
import shlex
from pathlib import Path

from harnesslab.models.agent import AgentProfile
from harnesslab.settings import Settings
from harnesslab.storage.paths import atomic_write_text

BUILT_IN_AGENTS = {
    "oracle",
    "claude-code",
    "codex",
    "aider",
    "opencode",
    "openhands",
    "pi",
    "gemini-cli",
    "qwen-coder",
}


class ProfileCompiler:
    def __init__(self, settings: Settings):
        self.settings = settings

    def compile(self, profile: AgentProfile) -> dict:
        if profile.kind == "custom-command":
            return self._compile_custom(profile)
        harbor_agent = profile.harbor.agent or profile.kind
        if harbor_agent not in BUILT_IN_AGENTS:
            raise ValueError(f"unsupported built-in Harbor agent: {harbor_agent}")
        return {
            "mode": "built-in",
            "agent_config": {
                "name": harbor_agent,
                "model_name": profile.harbor.model,
                "env": {name: None for name in profile.auth.inherit_env},
                "skills": profile.skills.paths,
                "mcp_servers": profile.mcp.config_paths,
                "kwargs": profile.harbor.kwargs,
                "agent_timeout_sec": profile.runtime.agent_timeout_sec,
            },
            "manifest": self._manifest(profile, generated_file=None),
        }

    def _compile_custom(self, profile: AgentProfile) -> dict:
        command = profile.command_agent
        if command is None:
            raise ValueError("custom-command profile requires [command_agent]")
        if profile.auth.include_paths and profile.runtime.backend != "docker":
            raise ValueError("include_paths are denied for non-Docker backends")
        agent_dir = self.settings.generated_agents_dir / profile.id
        class_name = _class_name(profile.id)
        file_path = agent_dir / "agent.py"
        run_template = command.run.replace("{{instruction}}", "{instruction}")
        body = _generated_agent_source(class_name, profile.id, run_template, command.shell)
        atomic_write_text(file_path, body)
        manifest = self._manifest(profile, generated_file=file_path)
        atomic_write_text(
            agent_dir / "manifest.json",
            json.dumps(manifest, indent=2, sort_keys=True),
        )
        return {
            "mode": "generated",
            "agent_config": {
                "import_path": f"{file_path}:{class_name}",
                "env": {name: None for name in profile.auth.inherit_env},
                "kwargs": {},
                "agent_timeout_sec": profile.runtime.agent_timeout_sec,
            },
            "manifest": manifest,
        }

    def _manifest(self, profile: AgentProfile, generated_file: Path | None) -> dict:
        profile_json = profile.model_dump_json(exclude_none=True)
        generated_hash = None
        if generated_file is not None and generated_file.exists():
            generated_hash = hashlib.sha256(generated_file.read_bytes()).hexdigest()
        return {
            "profile_id": profile.id,
            "profile_hash": hashlib.sha256(profile_json.encode("utf-8")).hexdigest(),
            "generated_file": str(generated_file) if generated_file else None,
            "generated_file_hash": generated_hash,
            "compiler_version": "agent-profile-v2.0",
        }


def _class_name(agent_id: str) -> str:
    parts = re.split(r"[^a-zA-Z0-9]+", agent_id)
    return "".join(part.capitalize() for part in parts if part) + "Agent"


def _generated_agent_source(class_name: str, agent_id: str, run_template: str, shell: str) -> str:
    quoted_template = repr(run_template)
    return f'''"""Generated HarnessLab custom command agent."""

from harbor.agents.installed.base import BaseInstalledAgent, with_prompt_template


class {class_name}(BaseInstalledAgent):
    @staticmethod
    def name() -> str:
        return {agent_id!r}

    @with_prompt_template
    async def run(self, instruction, environment, context):
        import shlex

        command = {quoted_template}.format(instruction=shlex.quote(instruction))
        await self.exec_as_agent(environment, command, shell={shell!r})
'''


def normalize_command_preview(template: str, sample_instruction: str = "Solve the task") -> str:
    return template.replace("{{instruction}}", shlex.quote(sample_instruction))
