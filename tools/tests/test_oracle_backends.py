from __future__ import annotations

import pytest

from rustsheet_tools.oracle.backends import GRAPH, GraphBackend


def _clear(monkeypatch: pytest.MonkeyPatch) -> None:
    for var in ("TENANT_ID", "CLIENT_ID", "CLIENT_SECRET", "DRIVE_ID", "USER", "MSAL_CACHE_B64"):
        monkeypatch.delenv(f"RUSTSHEET_ORACLE_{var}", raising=False)


def test_app_only_accepts_user_email(monkeypatch: pytest.MonkeyPatch) -> None:
    _clear(monkeypatch)
    monkeypatch.setenv("RUSTSHEET_ORACLE_TENANT_ID", "t")
    monkeypatch.setenv("RUSTSHEET_ORACLE_CLIENT_ID", "c")
    monkeypatch.setenv("RUSTSHEET_ORACLE_CLIENT_SECRET", "s")
    monkeypatch.setenv("RUSTSHEET_ORACLE_USER", "me@example.com")
    backend = GraphBackend()
    assert backend.available() == (True, "app-only")
    assert backend._drive() == f"{GRAPH}/users/me%40example.com/drive"


def test_app_only_requires_a_drive_target(monkeypatch: pytest.MonkeyPatch) -> None:
    _clear(monkeypatch)
    monkeypatch.setenv("RUSTSHEET_ORACLE_TENANT_ID", "t")
    monkeypatch.setenv("RUSTSHEET_ORACLE_CLIENT_ID", "c")
    monkeypatch.setenv("RUSTSHEET_ORACLE_CLIENT_SECRET", "s")
    ok, why = GraphBackend().available()
    assert not ok
    assert "RUSTSHEET_ORACLE_USER" in why


def test_drive_id_takes_precedence(monkeypatch: pytest.MonkeyPatch) -> None:
    _clear(monkeypatch)
    monkeypatch.setenv("RUSTSHEET_ORACLE_DRIVE_ID", "b!abc")
    monkeypatch.setenv("RUSTSHEET_ORACLE_USER", "me@example.com")
    assert GraphBackend()._drive() == f"{GRAPH}/drives/b!abc"
