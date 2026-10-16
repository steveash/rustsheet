from __future__ import annotations

from types import SimpleNamespace
from typing import Any

import pytest
import requests

from rustsheet_tools.oracle.backends import GRAPH, GraphBackend, LibreOfficeBackend, OracleError, text_blocks
from rustsheet_tools.spec2xlsx import build_xlsx

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
    calls: list[str] = []

    def fake_call(method: str, url: str, **_: Any) -> Any:
        calls.append(f"{method} {url}")
        return SimpleNamespace(json=lambda: {"id": "contoso.sharepoint.com,a1,b2"})

    app_only_env.setattr(backend, "_call", fake_call)
    drive = f"{GRAPH}/sites/contoso.sharepoint.com%2Ca1%2Cb2/drive"
    assert backend._drive() == drive
    assert backend._drive() == drive  # resolved once, then cached
    assert calls == [f"GET {GRAPH}/sites/contoso.sharepoint.com:/sites/RustSheet%20Oracle"]


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
    assert backend._redact('{"error": "no drive for me@example.com"}') == '{"error": "no drive for <user>"}'


def test_drive_id_takes_precedence(app_only_env: pytest.MonkeyPatch) -> None:
    app_only_env.setenv("RUSTSHEET_ORACLE_DRIVE_ID", "b!abc")
    app_only_env.setenv("RUSTSHEET_ORACLE_USER", "me@example.com")
    assert GraphBackend()._drive() == f"{GRAPH}/drives/b!abc"


def test_delegated_mode_uses_me_even_when_user_is_set(clean_env: pytest.MonkeyPatch) -> None:
    clean_env.setenv("RUSTSHEET_ORACLE_USER", "me@example.com")
    assert GraphBackend()._drive() == f"{GRAPH}/me/drive"


class FakeGraph:
    """Records Graph calls; the item's eTag moves only once the session is closed (Excel saved)."""

    def __init__(self, *, saves: bool = True) -> None:
        self.calls: list[str] = []
        self.closed = False
        self.saves = saves

    def __call__(self, method: str, url: str, **kw: Any) -> Any:
        tail = url.rsplit("/items/1", 1)[-1] or "<item>"
        self.calls.append(f"{method} {tail}")
        if kw.get("json") and "values" in kw["json"]:
            self.calls.append(f"  values={kw['json']['values']}")
        if tail == "/workbook/closeSession":
            self.closed = True
        etag = "v2" if self.closed and self.saves else "v1"
        return SimpleNamespace(json=lambda: {"id": "1", "eTag": etag}, content=b"saved")


@pytest.fixture
def graph(app_only_env: pytest.MonkeyPatch) -> GraphBackend:
    app_only_env.setenv("RUSTSHEET_ORACLE_DRIVE_ID", "b!x")
    return GraphBackend()


def test_recalc_edits_after_calculating_so_excel_saves(
    graph: GraphBackend, monkeypatch: pytest.MonkeyPatch
) -> None:
    fake = FakeGraph()
    monkeypatch.setattr(graph, "_call", fake)
    content, _ = graph.recalc(build_xlsx({"sheets": [{"name": "S", "cells": {"A1": "=1"}}]}))
    assert content == b"saved"
    session = [c.split(" ", 1)[1].split("('")[0] for c in fake.calls[1:6]]
    assert session == [
        "/workbook/createSession",
        "/workbook/application/calculate",
        "/workbook/worksheets/add",  # the edit that makes closeSession persist
        "/workbook/worksheets",  # ...undone, so the result has no extra sheet
        "/workbook/closeSession",
    ]
    assert fake.calls[-1] == "DELETE <item>"


def test_recalc_fails_when_excel_never_saves(graph: GraphBackend, monkeypatch: pytest.MonkeyPatch) -> None:
    fake = FakeGraph(saves=False)
    monkeypatch.setattr(graph, "_call", fake)
    monkeypatch.setattr("rustsheet_tools.oracle.backends.time.sleep", lambda _: None)
    with pytest.raises(OracleError, match="never saved"):
        graph.recalc(build_xlsx({"sheets": [{"name": "S", "cells": {"A1": "=1"}}]}))
    assert "GET /content" not in fake.calls  # never hands back the upload
    assert fake.calls[-1] == "DELETE <item>"


def test_dropped_connections_are_retried(graph: GraphBackend, monkeypatch: pytest.MonkeyPatch) -> None:
    attempts: list[int] = []

    def flaky(*_: Any, **__: Any) -> Any:
        attempts.append(1)
        if len(attempts) < 3:
            raise requests.ConnectionError("reset by peer")
        return SimpleNamespace(status_code=200)

    monkeypatch.setattr(graph, "_token", lambda: "t")
    monkeypatch.setattr(graph._session, "request", flaky)
    monkeypatch.setattr("rustsheet_tools.oracle.backends.time.sleep", lambda _: None)
    assert graph._call("GET", "https://x").status_code == 200
    assert len(attempts) == 3


def test_dropped_connections_raise_oracle_error_when_exhausted(
    app_only_env: pytest.MonkeyPatch,
) -> None:
    app_only_env.setenv("RUSTSHEET_ORACLE_USER", "me@example.com")
    backend = GraphBackend()

    def dead(*_: Any, **__: Any) -> Any:
        raise requests.ConnectionError("reset by peer")

    app_only_env.setattr(backend, "_token", lambda: "t")
    app_only_env.setattr(backend._session, "request", dead)
    app_only_env.setattr("rustsheet_tools.oracle.backends.time.sleep", lambda _: None)
    with pytest.raises(OracleError, match="retries exhausted") as e:
        backend._call("GET", backend._drive())
    assert "me@example.com" not in str(e.value)
    assert "me%40example.com" not in str(e.value)


def test_recalc_enters_typed_cells_before_calculating(
    graph: GraphBackend, monkeypatch: pytest.MonkeyPatch
) -> None:
    fake = FakeGraph()
    monkeypatch.setattr(graph, "_call", fake)
    spec = {"sheets": [{"name": "It's", "cells": {"A1": {"entered": "(1,000)"}, "B1": "=A1"}}]}
    graph.recalc(build_xlsx(spec))
    i = fake.calls.index("POST /workbook/createSession")
    assert fake.calls[i + 1 : i + 4] == [
        "PATCH /workbook/worksheets('It%27%27s')/range(address='A1')",
        "  values=[['(1,000)']]",
        "POST /workbook/application/calculate",
    ]


def test_libreoffice_refuses_typed_cells() -> None:
    xlsx = build_xlsx({"sheets": [{"name": "S", "cells": {"A1": {"entered": "(1)"}}}]})
    with pytest.raises(OracleError, match="typed cell input"):
        LibreOfficeBackend().recalc(xlsx)


def test_recalc_reads_the_text_of_listed_cells_after_calculating(
    graph: GraphBackend, monkeypatch: pytest.MonkeyPatch
) -> None:
    fake = FakeGraph()
    blocks = {"A1:B2": [["1.50", ""], ["", "####"]]}

    def call(method: str, url: str, **kw: Any) -> Any:
        response = fake(method, url, **kw)
        if url.endswith("?$select=text"):
            block = url.split("address='")[1].split("'", maxsplit=1)[0].replace("%3A", ":")
            return SimpleNamespace(json=lambda: {"text": blocks[block]})
        return response

    monkeypatch.setattr(graph, "_call", call)
    spec = {
        "sheets": [{"name": "It's", "cells": {"A1": 1.5, "B2": 12345678}, "column_widths": {"B": 2}}],
        "text": ["'It''s'!A1", "It's!$B$2"],
    }
    _, info = graph.recalc(build_xlsx(spec))
    assert info.texts == {"It's": {"A1": "1.50", "B2": "####"}}
    i = fake.calls.index("POST /workbook/application/calculate")
    assert fake.calls[i + 1] == "GET /workbook/worksheets('It%27%27s')/range(address='A1%3AB2')?$select=text"
    assert fake.calls[i + 2] == "POST /workbook/worksheets/add"


def test_text_blocks_read_a_dense_sheet_at_once_and_sparse_cells_one_by_one() -> None:
    dense = [("S", f"{c}{r}") for c in "ABC" for r in range(1, 4)]
    assert text_blocks(dense) == [
        ("S", "A1:C3", {f"{c}{r}": (r - 1, "ABC".index(c)) for c in "ABC" for r in range(1, 4)})
    ]
    sparse = [("T", "A1"), ("T", "Z100")]
    assert text_blocks(sparse) == [("T", "A1", {"A1": (0, 0)}), ("T", "Z100", {"Z100": (0, 0)})]
    assert text_blocks([("U", "C5")]) == [("U", "C5", {"C5": (0, 0)})]


def test_a_missing_text_cell_is_refused(graph: GraphBackend, monkeypatch: pytest.MonkeyPatch) -> None:
    def call(method: str, url: str, **kw: Any) -> Any:
        return SimpleNamespace(json=lambda: {"text": [["1"]]})

    monkeypatch.setattr(graph, "_call", call)
    with pytest.raises(OracleError, match="no text for S!B1"):
        graph._read_texts("item", {}, [("S", "A1"), ("S", "B1")])


def test_a_malformed_text_cell_is_refused() -> None:
    with pytest.raises(ValueError, match="must be Sheet!A1"):
        build_xlsx({"sheets": [{"name": "S", "cells": {"A1": 1}}], "text": ["A1"]})
