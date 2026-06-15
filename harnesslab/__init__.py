from __future__ import annotations

import warnings

from ornnlab import __version__

warnings.warn(
    "harnesslab is deprecated; use ornnlab instead.",
    DeprecationWarning,
    stacklevel=2,
)

__all__ = ["__version__"]
