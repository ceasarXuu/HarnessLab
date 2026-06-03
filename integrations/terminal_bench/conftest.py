import enum
import importlib.util
import sys
import types
from dataclasses import dataclass


if importlib.util.find_spec("terminal_bench") is None:
    terminal_bench = types.ModuleType("terminal_bench")
    agents = types.ModuleType("terminal_bench.agents")
    base_agent = types.ModuleType("terminal_bench.agents.base_agent")
    failure_mode = types.ModuleType("terminal_bench.agents.failure_mode")
    terminal = types.ModuleType("terminal_bench.terminal")
    models = types.ModuleType("terminal_bench.terminal.models")
    tmux_session = types.ModuleType("terminal_bench.terminal.tmux_session")

    class FailureMode(enum.Enum):
        NONE = "none"
        PARSE_ERROR = "parse_error"
        AGENT_TIMEOUT = "agent_timeout"
        UNKNOWN_AGENT_ERROR = "unknown_agent_error"

    class BaseAgent:
        def _render_instruction(self, instruction):
            return instruction

    class AgentResult:
        def __init__(self, failure_mode=FailureMode.NONE):
            self.failure_mode = failure_mode

    @dataclass
    class TerminalCommand:
        command: str
        min_timeout_sec: float
        max_timeout_sec: float
        block: bool
        append_enter: bool

    class TmuxSession:
        pass

    base_agent.AgentResult = AgentResult
    base_agent.BaseAgent = BaseAgent
    failure_mode.FailureMode = FailureMode
    models.TerminalCommand = TerminalCommand
    tmux_session.TmuxSession = TmuxSession

    sys.modules["terminal_bench"] = terminal_bench
    sys.modules["terminal_bench.agents"] = agents
    sys.modules["terminal_bench.agents.base_agent"] = base_agent
    sys.modules["terminal_bench.agents.failure_mode"] = failure_mode
    sys.modules["terminal_bench.terminal"] = terminal
    sys.modules["terminal_bench.terminal.models"] = models
    sys.modules["terminal_bench.terminal.tmux_session"] = tmux_session
