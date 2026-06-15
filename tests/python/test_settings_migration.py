from __future__ import annotations

from ornnlab.settings import Settings


def test_ornnlab_home_wins_over_legacy_home(monkeypatch, tmp_path):
    new_home = tmp_path / "new"
    old_home = tmp_path / "old"
    monkeypatch.setenv("ORNNLAB_HOME", str(new_home))
    monkeypatch.setenv("HARNESSLAB_HOME", str(old_home))

    settings = Settings.from_env()

    assert settings.home == new_home
    assert "legacy_home_ignored" in settings.warnings


def test_legacy_home_env_is_supported_for_transition(monkeypatch, tmp_path):
    old_home = tmp_path / "legacy"
    monkeypatch.delenv("ORNNLAB_HOME", raising=False)
    monkeypatch.setenv("HARNESSLAB_HOME", str(old_home))

    settings = Settings.from_env()

    assert settings.home == old_home
    assert "legacy_env_in_use" in settings.warnings
    assert "using_legacy_home" in settings.warnings


def test_launcher_root_does_not_block_legacy_home_migration(monkeypatch, tmp_path):
    home = tmp_path / "home"
    monkeypatch.setenv("HOME", str(home))
    monkeypatch.delenv("ORNNLAB_HOME", raising=False)
    monkeypatch.delenv("HARNESSLAB_HOME", raising=False)
    launcher_root = home / ".ornnlab" / "launcher" / "source"
    legacy_home = home / ".harnesslab"
    launcher_root.mkdir(parents=True)
    (legacy_home / "agents").mkdir(parents=True)
    (legacy_home / "agents" / "oracle.toml").write_text("name = 'Oracle'\n", encoding="utf-8")
    (legacy_home / "harnesslab.sqlite").write_bytes(b"legacy-db")

    settings = Settings.from_env()
    settings.ensure_dirs()

    assert settings.home == home / ".ornnlab" / "data"
    assert settings.migration is not None
    assert settings.migration["ok"] is True
    assert (settings.home / ".ornnlab-home.json").exists()
    assert (settings.home / "migration" / "ornnlab-home-migration.json").exists()
    assert (settings.home / "agents" / "oracle.toml").exists()
    assert (settings.home / "ornnlab.sqlite").read_bytes() == b"legacy-db"
    assert (legacy_home / "agents" / "oracle.toml").exists()


def test_default_home_initializes_marker(monkeypatch, tmp_path):
    home = tmp_path / "home"
    monkeypatch.setenv("HOME", str(home))
    monkeypatch.delenv("ORNNLAB_HOME", raising=False)
    monkeypatch.delenv("HARNESSLAB_HOME", raising=False)

    settings = Settings.from_env()
    settings.ensure_dirs()

    assert settings.home == home / ".ornnlab" / "data"
    assert (settings.home / ".ornnlab-home.json").exists()
