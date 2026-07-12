from importlib.util import module_from_spec, spec_from_file_location
from pathlib import Path, PureWindowsPath

ROOT = Path(__file__).resolve().parents[2]


def _verification_module():
    module_path = ROOT / "scripts/verify-ornnlab-rebrand.py"
    spec = spec_from_file_location("verify_ornnlab_rebrand", module_path)
    assert spec and spec.loader
    module = module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_v1_0_5_document_inventory_tracks_current_release_documents():
    module = _verification_module()
    expected = {
        "docs/releases/v1.0.5/README.md",
        "docs/releases/v1.0.5/dev-daemon/README.md",
        "docs/releases/v1.0.5/dev-daemon/engineering-design.md",
        "docs/releases/v1.0.5/engineering-plan.md",
        "docs/releases/v1.0.5/prd.md",
        "docs/releases/v1.0.5/technical-design.md",
    }
    inventory = {path for path in module.DOC_INVENTORY if path.startswith("docs/releases/v1.0.5/")}

    assert inventory == expected
    assert all((ROOT / path).is_file() for path in inventory)


def test_rebrand_inventory_normalizes_windows_paths():
    module = _verification_module()

    assert module._normalize_path(PureWindowsPath("docs\\releases\\v1.0.5\\prd.md")) == (
        "docs/releases/v1.0.5/prd.md"
    )
