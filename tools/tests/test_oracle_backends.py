from __future__ import annotations

import pytest

from rustsheet_tools.oracle.backends import GRAPH, GraphBackend

VARS = ("TENANT_ID", "CLIENT_ID", "CLIENT_SECRET", "DRIVE_ID", "USER", "SITE", "MSAL_CACHE_B64")


@pytest.fixture
def clean_env(monkeypatch: pytest.MonkeyPatch) -> pytest.MonkeyPatch:
    for var in VARS:
        monkeypatch.delenv(f"RUSTSHEET_ORACLE_{var}", raising=False)
    return monkeypatch


@pytest.fixture
def app_only_env(clean_env: pytest.MonkeyPatch) -> pytest.MonkeyPatch:
    clean_env.setenv("RUSTSHEET_ORACLE_TENANT_ID", "t")
    clean_env.setenv("RUSTSHEET_ORACLE_CLIENT_ID", "c")
    clean_env.setenv("RUSTSHEET_ORACLE_CLIENT_SECRET", "s")
    return clean_env


def test_app_only_accepts_user_email(app_only_env: pytest.MonkeyPatch) -> None:
    app_only_env.setenv("RUSTSHEET_ORACLE_USER", "me@example.com")
    backend = GraphBackend()
    assert backend.available() == (True, "app-only")
    assert backend._drive() == f"{GRAPH}/users/me%40example.com/drive"


@pytest.mark.usefixtures("app_only_env")
def test_app_only_requires_a_drive_target() -> None:
    assert GraphBackend().available() == (
        False,
        "app-only mode needs RUSTSHEET_ORACLE_SITE, RUSTSHEET_ORACLE_DRIVE_ID or RUSTSHEET_ORACLE_USER",
    )


def test_site_targets_the_sites_document_library(app_only_env: pytest.MonkeyPatch) -> None:
    app_only_env.setenv("RUSTSHEET_ORACLE_SITE", "contoso.sharepoint.com/sites/RustSheet Oracle/")
    backend = GraphBackend()
    assert backend.available() == (True, "app-only")
    assert backend._drive() == f"{GRAPH}/sites/contoso.sharepoint.com:/sites/RustSheet%20Oracle:/drive"


def test_site_root_without_path(app_only_env: pytest.MonkeyPatch) -> None:
    app_only_env.setenv("RUSTSHEET_ORACLE_SITE", "contoso.sharepoint.com")
    assert GraphBackend()._drive() == f"{GRAPH}/sites/contoso.sharepoint.com/drive"


def test_user_with_slash_cannot_escape_the_path(app_only_env: pytest.MonkeyPatch) -> None:
    app_only_env.setenv("RUSTSHEET_ORACLE_USER", "x/../../sites/abc")
    assert GraphBackend()._drive() == f"{GRAPH}/users/x%2F..%2F..%2Fsites%2Fabc/drive"


def test_errors_redact_the_owner_email(app_only_env: pytest.MonkeyPatch) -> None:
    app_only_env.setenv("RUSTSHEET_ORACLE_USER", "me@example.com")
    backend = GraphBackend()
    assert backend._redact(backend._drive() + "/items/1") == f"{GRAPH}/users/<user>/drive/items/1"


def test_drive_id_takes_precedence(app_only_env: pytest.MonkeyPatch) -> None:
    app_only_env.setenv("RUSTSHEET_ORACLE_DRIVE_ID", "b!abc")
    app_only_env.setenv("RUSTSHEET_ORACLE_USER", "me@example.com")
    assert GraphBackend()._drive() == f"{GRAPH}/drives/b!abc"


def test_delegated_mode_uses_me_even_when_user_is_set(clean_env: pytest.MonkeyPatch) -> None:
    clean_env.setenv("RUSTSHEET_ORACLE_USER", "me@example.com")
    assert GraphBackend()._drive() == f"{GRAPH}/me/drive"
