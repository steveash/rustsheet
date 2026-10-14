"""Shared fixtures."""

from __future__ import annotations

from typing import Any

import pytest

from rustsheet_tools import repo_root
from rustsheet_tools.fitlab import sweep


@pytest.fixture
def real_cases(monkeypatch: pytest.MonkeyPatch) -> None:
    """Case discovery without the release binary: the listing points at the committed files."""

    def listing(suite: str) -> list[dict[str, Any]]:
        return [
            {
                "id": f"{suite}/{p.name}",
                "path": str(p),
                "sidecar": str(p).replace(".case.json", ".oracle.json"),
            }
            for p in sorted((repo_root() / "eval" / "suites" / suite / "cases").glob("*.case.json"))
        ]

    monkeypatch.setattr(sweep, "suite_cases", listing)
