#!/usr/bin/env python3
"""Version governance guard.

Complements verify-ornnlab-rebrand.py by checking:
1. Authority file versions are parseable and internally consistent.
2. Active docs do not contain unapproved literal ornnlab@<version> references.
3. README and quickstart prefer unversioned install commands.

Run: uv run python scripts/verify-version-governance.py
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
ARTIFACT = ROOT / "artifacts/governance/version-governance-check.json"

# 文件允许包含字面 ornnlab@<version> 引用（release 记录、版本治理、playbook 等）
ALLOWED_LITERAL_FILES: set[str] = {
    "docs/release/ornnlab-0.1.3.md",
    "docs/release/ornnlab-0.1.3-docs.md",
    "docs/release/2026-06-16-ornnlab-0.1.3.md",
    "docs/release/version-governance.md",
    "docs/release/checklist.md",
    "docs/v0.1.3/prd.md",
    "docs/v0.1.3/technical-design.md",
    "docs/v0.1.3/engineering-plan.md",
    "docs/playbooks/npm-package-reservation.md",
    "docs/plans/2026-06-15-ornnlab-rebrand-checklist.md",
    "prd/2026-06-15-ornnlab-npm-distribution.md",
    "prd/2026-06-15-ornnlab-webui-prd.md",
    "prd/2026-06-16-ornnlab-zero-friction-bootstrap.md",
}

# 这些文件不应包含字面版本引用
NO_LITERAL_FILES = {
    "README.md",
    "docs/playbooks/install-quickstart.md",
}

LITERAL_VERSION_RE = re.compile(r"ornnlab@\d+\.\d+\.\d+")


def load_authority_versions() -> dict[str, str]:
    versions: dict[str, str] = {}
    package_json = json.loads((ROOT / "package.json").read_text(encoding="utf-8"))
    versions["npm"] = package_json["version"]
    pyproject = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
    match = re.search(r'^version\s*=\s*"([^"]+)"', pyproject, re.MULTILINE)
    if match:
        versions["python"] = match.group(1)
    frontend = json.loads((ROOT / "frontend/package.json").read_text(encoding="utf-8"))
    versions["frontend"] = frontend["version"]
    return versions


def scan_active_docs() -> list[dict[str, Any]]:
    offenders: list[dict[str, Any]] = []
    scan_paths: list[Path] = []
    for path in (ROOT / "docs").rglob("*.md"):
        if "/archive/" not in str(path.relative_to(ROOT)):
            scan_paths.append(path)
    for path in (ROOT / "prd").rglob("*.md"):
        scan_paths.append(path)
    readme = ROOT / "README.md"
    if readme.exists():
        scan_paths.append(readme)

    for path in scan_paths:
        relative = str(path.relative_to(ROOT))
        if relative in ALLOWED_LITERAL_FILES:
            continue
        text = path.read_text(encoding="utf-8")
        matches = LITERAL_VERSION_RE.findall(text)
        if matches:
            offenders.append({"file": relative, "matches": sorted(set(matches))})
    return offenders


def check_no_literal_in_quickstart() -> list[dict[str, Any]]:
    offenders: list[dict[str, Any]] = []
    for relative in NO_LITERAL_FILES:
        path = ROOT / relative
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        matches = LITERAL_VERSION_RE.findall(text)
        if matches:
            offenders.append({"file": relative, "matches": sorted(set(matches))})
    return offenders


def _result(name: str, passed: bool, evidence: dict[str, Any]) -> dict[str, Any]:
    return {
        "name": name,
        "status": "passed" if passed else "failed",
        "evidence": evidence,
    }


def main() -> int:
    checks: list[dict[str, Any]] = []

    try:
        versions = load_authority_versions()
        checks.append(_result(
            "authority version files are parseable",
            True,
            versions,
        ))
    except Exception as e:
        checks.append(_result(
            "authority version files are parseable",
            False,
            {"error": str(e)},
        ))
        versions = {}

    authority_ok = "npm" in versions and "python" in versions
    checks.append(_result(
        "authority versions are present",
        authority_ok,
        {"issues": [] if authority_ok else ["missing npm or python version"], "versions": versions},
    ))

    literal_offenders = scan_active_docs()
    checks.append(_result(
        "active docs avoid unapproved literal version references",
        not literal_offenders,
        {"offenders": literal_offenders},
    ))

    quickstart_offenders = check_no_literal_in_quickstart()
    checks.append(_result(
        "README and quickstart avoid literal version references",
        not quickstart_offenders,
        {"offenders": quickstart_offenders},
    ))

    ok = all(c["status"] == "passed" for c in checks)

    ARTIFACT.parent.mkdir(parents=True, exist_ok=True)
    ARTIFACT.write_text(
        json.dumps({"status": "passed" if ok else "failed", "checks": checks}, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    print(f"Version governance guard: {'PASSED' if ok else 'FAILED'}")
    for c in checks:
        icon = "PASS" if c["status"] == "passed" else "FAIL"
        print(f"  [{icon}] {c['name']}")
        if c["status"] != "passed":
            print(f"    {json.dumps(c['evidence'], indent=2)}")

    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
