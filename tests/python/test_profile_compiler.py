import json

from ornnlab.models.agent import AgentProfile
from ornnlab.services.profile_compiler import ProfileCompiler, normalize_command_preview


def test_builtin_profile_compiles_to_harbor_agent_config(settings):
    profile = AgentProfile.model_validate(
        {
            "schema_version": 2,
            "id": "oracle",
            "name": "Oracle",
            "kind": "oracle",
            "harbor": {"agent": "oracle"},
            "auth": {"inherit_env": ["OPENAI_API_KEY"]},
        }
    )

    result = ProfileCompiler(settings).compile(profile)

    assert result["mode"] == "built-in"
    assert result["agent_config"]["name"] == "oracle"
    assert result["agent_config"]["env"] == {"OPENAI_API_KEY": None}


def test_custom_command_generates_manifest(settings):
    profile = AgentProfile.model_validate(
        {
            "schema_version": 2,
            "id": "custom-agent",
            "name": "Custom Agent",
            "kind": "custom-command",
            "command_agent": {"run": "agent run {{instruction}}"},
        }
    )

    result = ProfileCompiler(settings).compile(profile)
    manifest_path = settings.generated_agents_dir / "custom-agent" / "manifest.json"

    assert result["mode"] == "generated"
    assert manifest_path.exists()
    assert json.loads(manifest_path.read_text())["profile_id"] == "custom-agent"


def test_command_preview_quotes_instruction():
    preview = normalize_command_preview("agent run {{instruction}}", "solve && leak")

    assert preview == "agent run 'solve && leak'"


def test_include_paths_denied_for_non_docker_backend(settings):
    profile = AgentProfile.model_validate(
        {
            "schema_version": 2,
            "id": "unsafe",
            "name": "Unsafe",
            "kind": "custom-command",
            "command_agent": {"run": "agent {{instruction}}"},
            "auth": {"include_paths": ["~/.ssh"]},
            "runtime": {"backend": "local"},
        }
    )

    try:
        ProfileCompiler(settings).compile(profile)
    except ValueError as exc:
        assert "include_paths" in str(exc)
    else:
        raise AssertionError("expected include_paths denial")
