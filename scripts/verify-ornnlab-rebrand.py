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
    "docs/architecture/docs-index.md": "rename-now",
    "docs/architecture/benchmark-compatibility-strategy.md": "historical-stub",
    "docs/architecture/harnesslab-vs-harbor.md": "historical-stub",
    "docs/playbooks/development-operations.md": "rename-now",
    "docs/playbooks/harbor-upgrade-procedure.md": "rename-now",
    "docs/playbooks/install-quickstart.md": "rename-now",
    "docs/playbooks/npm-package-reservation.md": "rename-now",
    "docs/playbooks/terminal-bench-claude-ds.md": "historical",
    "docs/releases/v0.1.3/checklist.md": "rename-now",
    "docs/releases/v0.1.3/2026-06-16-ornnlab-0.1.3.md": "rename-now",
    "docs/releases/v0.1.4/shim-retirement/harbor-rebrand-residue-fix-plan.md": "rename-now",
    "docs/releases/v0.1.4/shim-retirement/harnesslab-shim-retirement-prd.md": "rename-now",
    "docs/releases/v0.1.4/shim-retirement/harnesslab-shim-retirement-plan.md": "rename-now",
    "docs/releases/v0.1.4/product-goal.md": "rename-now",
    "docs/releases/v0.1.4/bugfix/README.md": "rename-now",
    "docs/releases/v0.1.4/bugfix/01-toctou-cancel-overwrites-completed-run.md": "rename-now",
    "docs/releases/v0.1.4/bugfix/02-crash-recovery-blind-spot-dequeue-to-running.md": "rename-now",
    "docs/releases/v0.1.4/bugfix/03-wait-true-blocks-unrelated-experiments.md": "rename-now",
    "docs/releases/v0.1.4/bugfix/04-sse-stream-not-realtime.md": "rename-now",
    "docs/releases/v0.1.4/bugfix/05-subprocess-synthesizes-completed-without-result.md": "rename-now",
    "docs/releases/v0.1.4/bugfix/06-duplicated-inconsistent-status-derivation.md": "rename-now",
    "docs/releases/v0.1.4/bugfix/07-event-mirror-quadratic-read-write.md": "rename-now",
    "docs/releases/v0.1.4/bugfix/08-db-connect-repeated-ensure-dirs-pragma.md": "rename-now",
    "docs/releases/v0.1.4/bugfix/09-worker-recreates-experiment-service-per-run.md": "rename-now",
    "docs/releases/v0.1.4/bugfix/10-worker-serial-run-execution.md": "rename-now",
    "docs/releases/v0.1.4/bugfix/11-profile-compiler-redundant-hashing.md": "rename-now",
    "docs/releases/v0.1.4/web-connectivity/README.md": "rename-now",
    "docs/releases/v0.1.4/web-connectivity/01-vite-dev-proxy-missing.md": "rename-now",
    "docs/releases/v0.1.4/web-connectivity/02-views-not-consuming-api.md": "rename-now",
    "docs/releases/v0.1.4/web-connectivity/03-contract-gap-vs-backend.md": "rename-now",
    "docs/releases/v0.1.4/web-connectivity/04-loading-error-empty-states.md": "rename-now",
    "docs/releases/v0.1.4/web-connectivity/05-integration-test-gap.md": "rename-now",
    "docs/releases/v0.1.4/web-connectivity/06-web-design-best-practices.md": "rename-now",
    "docs/releases/v1.0.5/README.md": "rename-now",
    "docs/releases/v1.0.5/harbor-cli-to-ui-architecture.md": "rename-now",
    "docs/releases/v1.0.5/frontend-rebuild-architecture.md": "rename-now",
    "docs/releases/v1.0.5/harbor-webui-feature-coverage-checklist.md": "rename-now",
    "docs/spikes/2026-06-15-harbor-lifecycle-spike.md": "rename-now",
    "docs/architecture/technology-decisions.md": "rename-now",
    "docs/architecture/test-engineering.md": "rename-now",
    "docs/releases/v0.1.3/ornnlab-0.1.3-docs.md": "rename-now",
    "docs/releases/v0.1.3/engineering-plan.md": "rename-now",
    "docs/releases/v0.1.3/ornnlab-0.1.3.md": "rename-now",
    "docs/releases/v0.1.3/technical-design.md": "rename-now",
    "docs/releases/v0.1.3/prd.md": "rename-now",
    "docs/releases/v0.1.3/version-governance.md": "rename-now",
}

DOC_CONTROL_REQUIRED = {
    "docs/playbooks/development-operations.md",
    "docs/playbooks/harbor-upgrade-procedure.md",
    "docs/playbooks/install-quickstart.md",
    "docs/playbooks/npm-package-reservation.md",
    "docs/releases/v0.1.3/checklist.md",
    "docs/releases/v0.1.3/2026-06-16-ornnlab-0.1.3.md",
    "docs/releases/v0.1.4/shim-retirement/harbor-rebrand-residue-fix-plan.md",
    "docs/releases/v0.1.4/shim-retirement/harnesslab-shim-retirement-prd.md",
    "docs/releases/v0.1.4/shim-retirement/harnesslab-shim-retirement-plan.md",
    "docs/spikes/2026-06-15-harbor-lifecycle-spike.md",
    "docs/architecture/technology-decisions.md",
    "docs/architecture/test-engineering.md",
    "docs/releases/v0.1.3/ornnlab-0.1.3-docs.md",
    "docs/releases/v0.1.3/engineering-plan.md",
    "docs/releases/v0.1.3/ornnlab-0.1.3.md",
    "docs/releases/v0.1.3/technical-design.md",
    "docs/releases/v0.1.3/prd.md",
    "docs/releases/v0.1.3/version-governance.md",
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
        _check_docs_root_converged(),
        _check_version_folder_contract(),
        _check_product_metadata(),
        _check_current_docs(),
        _check_python_package(),
        _check_npm_package_surface(),
        _check_scoped_transition_package(),
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
        for root in ["docs"]
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


def _check_docs_root_converged() -> dict[str, Any]:
    root_markdown = sorted(path.name for path in (ROOT / "docs").glob("*.md"))
    return _result(
        "docs root has no direct markdown files",
        "inspect docs/*.md",
        not root_markdown,
        {"root_markdown": root_markdown},
    )


def _check_version_folder_contract() -> dict[str, Any]:
    version_dir = ROOT / "docs/releases/v0.1.3"
    required = [
        "prd.md",
        "technical-design.md",
        "engineering-plan.md",
    ]
    discovered = sorted(path.name for path in version_dir.glob("*.md"))
    missing = [name for name in required if not (version_dir / name).exists()]
    expected_links = {
        "technical-design.md": ["prd.md", "engineering-plan.md"],
        "engineering-plan.md": ["prd.md", "technical-design.md"],
    }
    missing_links: list[str] = []
    for name, needles in expected_links.items():
        path = version_dir / name
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        for needle in needles:
            if needle not in text:
                missing_links.append(f"docs/releases/v0.1.3/{name}:{needle}")
    prd_text = (version_dir / "prd.md").read_text(encoding="utf-8")
    prd_metadata_present = (
        "PRD Document Version:" in prd_text
        and "## PRD Document Version History" in prd_text
    )
    return _result(
        "current version folder has required PRD/design/plan bundle",
        "inspect docs/releases/v0.1.3 document contract",
        not missing
        and not missing_links
        and prd_metadata_present,
        {
            "version_dir": "docs/releases/v0.1.3",
            "required": required,
            "discovered": discovered,
            "missing": missing,
            "missing_links": missing_links,
            "prd_metadata_present": prd_metadata_present,
        },
    )


def _check_product_metadata() -> dict[str, Any]:
    root_package = _json(ROOT / "package.json")
    pyproject = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
    frontend_package_path = ROOT / "frontend/package.json"
    frontend_name = None
    if frontend_package_path.exists():
        frontend_package = _json(frontend_package_path)
        frontend_name = frontend_package.get("name")
    passed = (
        root_package["name"] == "ornnlab"
        and root_package["bin"] == {"ornnlab": "bin/ornnlab.js"}
        and frontend_name in {None, "@ceasarxuu/ornnlab-frontend"}
        and 'name = "ornnlab"' in pyproject
        and 'ornnlab = "ornnlab.cli:main"' in pyproject
    )
    return _result(
        "package metadata uses OrnnLab names",
        "inspect package.json, frontend/package.json, pyproject.toml",
        passed,
        {
            "npm_name": root_package.get("name"),
            "frontend_name": frontend_name,
            "frontend_status": "present" if frontend_package_path.exists() else "pending-v1.0.5-rebuild",
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
        and not (ROOT / "harnesslab").exists()
        and (ROOT / "ornnlab/api").exists()
    )
    return _result(
        "Python package is OrnnLab only (HarnessLab shim retired)",
        "inspect ornnlab package directory and confirm harnesslab is gone",
        passed,
        {
            "ornnlab_main": str(ROOT / "ornnlab/__main__.py"),
            "harnesslab_dir_present": (ROOT / "harnesslab").exists(),
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


def _write_artifact(checks: list[dict[str, Any]], ok: bool) -> None:
    ARTIFACT.parent.mkdir(parents=True, exist_ok=True)
    root_package = _json(ROOT / "package.json")
    payload = {
        "schema_version": 1,
        "generated_at": datetime.now(UTC).isoformat(),
        "status": "passed" if ok else "failed",
        "version": root_package.get("version"),
        "package_artifact_path": "ornnlab-{}.tgz".format(root_package.get("version")),
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
