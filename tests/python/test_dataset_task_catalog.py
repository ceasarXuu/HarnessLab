from pathlib import Path

from ornnlab.services.dataset_task_catalog import LocalDatasetTaskCatalog


def test_local_task_catalog_returns_only_the_requested_page(tmp_path: Path):
    dataset = tmp_path / "dataset"
    for index in range(45):
        task = dataset / f"task-{index + 1:03d}"
        (task / "environment").mkdir(parents=True)
        (task / "task.toml").write_text('schema_version = "1.3"\n', encoding="utf-8")

    catalog = LocalDatasetTaskCatalog()

    first = catalog.list_page(dataset, "demo@1.0", None, offset=0, limit=20)
    second = catalog.list_page(dataset, "demo@1.0", None, offset=20, limit=20)

    assert first.total == 45
    assert first.next_cursor == "20"
    assert [item["name"] for item in first.items] == [f"task-{index:03d}" for index in range(1, 21)]
    assert first.cache_hit is False
    assert second.cache_hit is True
    assert all(item["environment"] is None for item in first.items + second.items)


def test_local_task_catalog_filters_names_before_pagination(tmp_path: Path):
    dataset = tmp_path / "dataset"
    for name in ("apt-setup", "git-rebase", "apt-cleanup"):
        task = dataset / name
        (task / "environment").mkdir(parents=True)
        (task / "task.toml").write_text('schema_version = "1.3"\n', encoding="utf-8")

    page = LocalDatasetTaskCatalog().list_page(
        dataset, "demo@1.0", "apt", offset=0, limit=1
    )

    assert page.total == 2
    assert page.next_cursor == "1"
    assert [item["name"] for item in page.items] == ["apt-cleanup"]
