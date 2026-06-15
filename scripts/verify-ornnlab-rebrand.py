from __future__ import annotations

import json
import re
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
CHECKLIST = ROOT / "docs/plans/2026-06-15-ornnlab-rebrand-checklist.md"
ARTIFACT = ROOT / "artifacts/rebrand/ornnlab-rebrand-verification.json"

DOC_INVENTORY = {
    "docs/README.md": "rename-now",
    "docs/adapter-protocol.md": "historical-stub",
    "docs/agent-profile-reference.md": "historical-stub",
    "docs/agent-registration-guide.md": "historical-stub",
    "docs/architecture.md": "superseded-stub",
    "docs/architecture/benchmark-compatibility-strategy.md": "historical-stub",
    "docs/architecture/harnesslab-vs-harbor.md": "historical-stub",
    "docs/development-operations.md": "rename-now",
    "docs/harbor-upgrade-procedure.md": "rename-now",
    "docs/install-quickstart.md": "rename-now",
    "docs/mvp-development-spec.md": "superseded-stub",
    "docs/plans/2026-06-03-agent-registration-gap-completion.md": "historical",
    "docs/plans/2026-06-03-agent-registration-registry.md": "historical",
    "docs/plans/2026-06-04-benchmark-adapter-architecture-design.md": "historical",
    "docs/plans/2026-06-04-benchmark-adapter-phase-0-inventory.md": "historical",
    "docs/plans/2026-06-04-benchmark-adapter-phase-1-boundary.md": "historical",
    "docs/plans/2026-06-04-benchmark-adapter-phase-1-coverage.md": "historical",
    "docs/plans/2026-06-04-benchmark-adapter-phase-1-inventory.md": "historical",
    "docs/plans/2026-06-05-benchmark-adapter-phase-2-inventory.md": "historical",
    (
        "docs/plans/"
        "2026-06-06-benchmark-adapter-phase-4-terminal-bench-runtime-extraction.md"
    ): "historical",
    (
        "docs/plans/"
        "2026-06-06-benchmark-adapter-phase-5-swe-bench-pro-runtime-extraction.md"
    ): "historical",
    "docs/plans/2026-06-06-benchmark-adapter-phase-6-runtime-snapshot-cleanup.md": "historical",
    "docs/plans/2026-06-06-benchmark-adapter-phase-7-docs-diagnostics.md": "historical",
    "docs/plans/2026-06-06-benchmark-adapter-phase-8-full-gate-closure.md": "historical",
    (
        "docs/plans/"
        "2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md"
    ): "historical",
    (
        "docs/plans/"
        "2026-06-08-universal-benchmark-adapter-protocol-phase-0-branch-inventory.md"
    ): "historical",
    (
        "docs/plans/"
        "2026-06-08-universal-benchmark-adapter-protocol-phase-0-frozen-selector-manifest.md"
    ): "historical",
    "docs/plans/2026-06-12-remove-external-runner-kind-plan.md": "historical",
    "docs/plans/2026-06-15-harbor-integration-engineering-plan.md": "historical-stub",
    "docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md": "rename-now",
    "docs/plans/2026-06-15-harnesslab-webui-engineering-plan.md": "historical-stub",
    "docs/plans/2026-06-15-ornnlab-rebrand-checklist.md": "rename-now",
    "docs/playbooks/npm-package-reservation.md": "rename-now",
    "docs/playbooks/terminal-bench-claude-ds.md": "historical",
    "docs/prd.md": "superseded-stub",
    "docs/release-checklist.md": "rename-now",
    "docs/releases/2026-06-16-ornnlab-0.1.3.md": "rename-now",
    "docs/reviews/2026-05-27-docker-runner-review-3.md": "historical",
    "docs/rust-legacy-fate.md": "historical",
    "docs/spikes/2026-06-15-harbor-lifecycle-spike.md": "rename-now",
    "docs/technology-decisions.md": "rename-now",
    "docs/test-engineering.md": "rename-now",
    "docs/v0.1.3/README.md": "rename-now",
    "docs/v0.1.3/engineering-plan.md": "rename-now",
    "docs/v0.1.3/release-ledger.md": "rename-now",
    "docs/v0.1.3/technical-design.md": "rename-now",
    "docs/v0.1.3/version-prd.md": "rename-now",
    "docs/version-governance.md": "rename-now",
    "prd/2026-06-07-universal-benchmark-adapter-protocol.md": "historical-stub",
    "prd/2026-06-15-ornnlab-npm-distribution.md": "rename-now",
    "prd/2026-06-15-ornnlab-webui-prd.md": "rename-now",
    "prd/2026-06-16-ornnlab-zero-friction-bootstrap.md": "rename-now",
}

DOC_CONTROL_REQUIRED = {
    "docs/development-operations.md",
    "docs/harbor-upgrade-procedure.md",
    "docs/install-quickstart.md",
    "docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md",
    "docs/playbooks/npm-package-reservation.md",
    "docs/release-checklist.md",
    "docs/releases/2026-06-16-ornnlab-0.1.3.md",
    "docs/spikes/2026-06-15-harbor-lifecycle-spike.md",
    "docs/technology-decisions.md",
    "docs/test-engineering.md",
    "docs/v0.1.3/README.md",
    "docs/v0.1.3/engineering-plan.md",
    "docs/v0.1.3/release-ledger.md",
    "docs/v0.1.3/technical-design.md",
    "docs/v0.1.3/version-prd.md",
    "docs/version-governance.md",
    "prd/2026-06-15-ornnlab-npm-distribution.md",
    "prd/2026-06-15-ornnlab-webui-prd.md",
    "prd/2026-06-16-ornnlab-zero-friction-bootstrap.md",
}

FORBIDDEN_CURRENT_PATTERNS = [
    re.compile(r"2026-06-15-harnesslab-webui-prd\.md"),
    re.compile(r"~/.ornnlab/HarnessLab"),
    re.compile(r"\.\./harnesslab/"),
]


def main() -> int:
    checks: list[dict[str, Any]] = [
        _check_doc_inventory(),
        _check_doc_control_tables(),
        _check_version_folder_contract(),
        _check_product_metadata(),
        _check_current_docs(),
        _check_python_package(),
        _check_npm_package_surface(),
        _check_scoped_transition_package(),
        _check_migration_tests(),
    ]
    ok = all(check["status"] == "passed" for check in checks)
    _write_artifact(checks, ok)
    if not ok:
        failed = [check for check in checks if check["status"] != "passed"]
        raise SystemExit(json.dumps(failed, indent=2, sort_keys=True))
    return 0


def _check_doc_inventory() -> dict[str, Any]:
    discovered = {
        str(path.relative_to(ROOT))
        for root in ["docs", "prd"]
        for path in (ROOT / root).rglob("*.md")
        if "/archive/" not in str(path.relative_to(ROOT))
    }
    expected = set(DOC_INVENTORY)
    missing = sorted(discovered - expected)
    stale = sorted(expected - discovered)
    return _result(
        "docs/prd inventory is explicit",
        "verify non-archive docs/prd files are represented",
        not missing and not stale,
        {"missing": missing, "stale": stale, "count": len(discovered)},
    )


def _check_doc_control_tables() -> dict[str, Any]:
    missing: list[str] = []
    incomplete: list[str] = []
    required_header = "| Document Version | Engineering Version(s) | Updated | Change |"
    for relative in sorted(DOC_CONTROL_REQUIRED):
        text = (ROOT / relative).read_text(encoding="utf-8")
        top = "\n".join(text.splitlines()[:20])
        if "## Document Control" not in top:
            missing.append(relative)
            continue
        if required_header not in top:
            incomplete.append(relative)
    return _result(
        "active PRD and technical docs have document version tables",
        "scan Document Control sections near top of governed docs",
        not missing and not incomplete,
        {
            "required_count": len(DOC_CONTROL_REQUIRED),
            "missing": missing,
            "incomplete": incomplete,
        },
    )


def _check_version_folder_contract() -> dict[str, Any]:
    version_dir = ROOT / "docs/v0.1.3"
    required = [
        "README.md",
        "version-prd.md",
        "technical-design.md",
        "engineering-plan.md",
        "release-ledger.md",
    ]
    missing = [name for name in required if not (version_dir / name).exists()]
    expected_links = {
        "README.md": [
            "version-prd.md",
            "technical-design.md",
            "engineering-plan.md",
            "release-ledger.md",
        ],
        "technical-design.md": ["version-prd.md", "engineering-plan.md", "release-ledger.md"],
        "engineering-plan.md": ["version-prd.md", "technical-design.md", "release-ledger.md"],
    }
    missing_links: list[str] = []
    for name, needles in expected_links.items():
        path = version_dir / name
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        for needle in needles:
            if needle not in text:
                missing_links.append(f"docs/v0.1.3/{name}:{needle}")
    return _result(
        "current version folder has required PRD/design/plan bundle",
        "inspect docs/v0.1.3 document contract",
        not missing and not missing_links,
        {
            "version_dir": "docs/v0.1.3",
            "required": required,
            "missing": missing,
            "missing_links": missing_links,
        },
    )


def _check_product_metadata() -> dict[str, Any]:
    root_package = _json(ROOT / "package.json")
    frontend_package = _json(ROOT / "frontend/package.json")
    pyproject = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
    passed = (
        root_package["name"] == "ornnlab"
        and root_package["bin"] == {"ornnlab": "bin/ornnlab.js"}
        and frontend_package["name"] == "@ceasarxuu/ornnlab-frontend"
        and 'name = "ornnlab"' in pyproject
        and 'ornnlab = "ornnlab.cli:main"' in pyproject
    )
    return _result(
        "package metadata uses OrnnLab names",
        "inspect package.json, frontend/package.json, pyproject.toml",
        passed,
        {
            "npm_name": root_package.get("name"),
            "frontend_name": frontend_package.get("name"),
            "python_project": "ornnlab" if 'name = "ornnlab"' in pyproject else None,
        },
    )


def _check_current_docs() -> dict[str, Any]:
    offenders: list[str] = []
    for relative, classification in DOC_INVENTORY.items():
        if classification not in {"rename-now", "superseded-stub"}:
            continue
        text = (ROOT / relative).read_text(encoding="utf-8")
        for pattern in FORBIDDEN_CURRENT_PATTERNS:
            if pattern.search(text):
                offenders.append(f"{relative}:{pattern.pattern}")
    return _result(
        "current docs avoid stale product links",
        "scan rename-now and superseded-stub docs for stale paths",
        not offenders,
        {"offenders": offenders},
    )


def _check_python_package() -> dict[str, Any]:
    passed = (
        (ROOT / "ornnlab/__main__.py").exists()
        and (ROOT / "harnesslab/__main__.py").exists()
        and not (ROOT / "harnesslab/api").exists()
        and (ROOT / "ornnlab/api").exists()
    )
    return _result(
        "Python package is renamed with compatibility shim",
        "inspect ornnlab and harnesslab package directories",
        passed,
        {
            "ornnlab_main": str(ROOT / "ornnlab/__main__.py"),
            "compat_main": str(ROOT / "harnesslab/__main__.py"),
        },
    )


def _check_npm_package_surface() -> dict[str, Any]:
    root_package = _json(ROOT / "package.json")
    files = set(root_package.get("files", []))
    passed = "bin/ornnlab.js" in files and "bin/harnesslab.js" not in files
    return _result(
        "root npm package excludes old harnesslab shim",
        "inspect package.json files array",
        passed,
        {"files": sorted(files)},
    )


def _check_scoped_transition_package() -> dict[str, Any]:
    package_dir = ROOT / "npm/harnesslab-transition"
    package_json = _json(package_dir / "package.json")
    bin_path = package_dir / "bin/harnesslab.js"
    help_text = bin_path.read_text(encoding="utf-8")
    passed = (
        package_json["name"] == "@ceasarxuu/harnesslab"
        and package_json["bin"] == {"harnesslab": "bin/harnesslab.js"}
        and "ornnlab --help" in help_text
        and "bin/ornnlab.js" not in package_json.get("files", [])
    )
    return _result(
        "old scoped npm transition package is staged separately",
        "inspect npm/harnesslab-transition",
        passed,
        {
            "package": package_json.get("name"),
            "bin": package_json.get("bin"),
            "staging_manifest": "npm/harnesslab-transition/package.json",
        },
    )


def _check_migration_tests() -> dict[str, Any]:
    test_text = (ROOT / "tests/python/test_settings_migration.py").read_text(encoding="utf-8")
    docker_text = (ROOT / "tests/python/test_docker_orphan_service.py").read_text(encoding="utf-8")
    backup_text = (ROOT / "tests/python/test_backup_service.py").read_text(encoding="utf-8")
    harbor_text = (ROOT / "tests/python/test_harbor_config.py").read_text(encoding="utf-8")
    passed = all(
        needle in haystack
        for needle, haystack in [
            ("test_ornnlab_home_wins_over_legacy_home", test_text),
            ("test_launcher_root_does_not_block_legacy_home_migration", test_text),
            ("harnesslab-backup-manifest.json", backup_text),
            ("harnesslab.run_id", docker_text),
            ("ORNNLAB_HARBOR_ENGINE", harbor_text),
        ]
    )
    return _result(
        "compatibility tests cover migration and legacy surfaces",
        "inspect targeted pytest modules",
        passed,
        {
            "migration_fixture_path": "tests/python/test_settings_migration.py",
            "docker_fixture_path": "tests/python/test_docker_orphan_service.py",
        },
    )


def _write_artifact(checks: list[dict[str, Any]], ok: bool) -> None:
    ARTIFACT.parent.mkdir(parents=True, exist_ok=True)
    root_package = _json(ROOT / "package.json")
    payload = {
        "schema_version": 1,
        "generated_at": datetime.now(UTC).isoformat(),
        "status": "passed" if ok else "failed",
        "version": root_package.get("version"),
        "package_artifact_path": "ornnlab-{}.tgz".format(root_package.get("version")),
        "migration_fixture_path": "tests/python/test_settings_migration.py",
        "command": "uv run python scripts/verify-ornnlab-rebrand.py",
        "checks": checks,
    }
    ARTIFACT.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _result(name: str, command: str, passed: bool, evidence: dict[str, Any]) -> dict[str, Any]:
    return {
        "name": name,
        "command": command,
        "status": "passed" if passed else "failed",
        "evidence": evidence,
    }


if __name__ == "__main__":
    raise SystemExit(main())
