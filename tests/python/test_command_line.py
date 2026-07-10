from ornnlab.services.command_line import split_command


def test_split_command_keeps_windows_path_separators():
    assert split_command(
        '"C:\\Program Files\\Python\\python.exe" C:\\work\\simulator.py run',
        windows=True,
    ) == ["C:\\Program Files\\Python\\python.exe", "C:\\work\\simulator.py", "run"]


def test_split_command_rejects_blank_commands():
    try:
        split_command("")
    except ValueError as error:
        assert str(error) == "command cannot be empty"
    else:
        raise AssertionError("blank command was accepted")


def test_split_command_rejects_whitespace_only_commands():
    try:
        split_command("   ")
    except ValueError as error:
        assert str(error) == "command cannot be empty"
    else:
        raise AssertionError("whitespace-only command was accepted")


def test_split_command_posix_mode_strips_quotes():
    assert split_command("'/usr/local/bin/my tool' --flag value", windows=False) == [
        "/usr/local/bin/my tool",
        "--flag",
        "value",
    ]
