"""Sheet edits replayed after load (T-1608)."""

from types import SimpleNamespace
from typing import Any
from urllib.parse import unquote

import pytest

from rustsheet_tools.oracle.backends import GraphBackend, OracleError, recorded_sheet_ops
from rustsheet_tools.oracle.canary import SHEET as CANARY
from rustsheet_tools.oracle.package import patch_parts
from rustsheet_tools.spec2xlsx import build_xlsx, sheet_ops


def _spec(ops: list[Any]) -> dict[str, Any]:
    return {
        "sheets": [{"name": "Data", "cells": {"A1": 1}}, {"name": "Calc", "cells": {"A1": "=Data!A1"}}],
        "sheet_ops": ops,
    }


def test_each_op_is_recorded_in_order() -> None:
    ops = [
        {"op": "rename", "sheet": "Data", "to": "Inputs"},
        {"op": "add", "sheet": "New"},
        {"op": "move", "sheet": "Calc", "to": 0},
        {"op": "insert_row", "sheet": "Calc", "row": 1},
        {"op": "delete_row", "sheet": "Calc", "row": 2},
        {"op": "delete_row", "sheet": "Calc", "row": 4, "count": 4},
        {"op": "delete_row", "sheet": "Calc", "row": 3, "count": 1},
        {"op": "insert_column", "sheet": "Calc", "column": "b"},
        {"op": "delete_column", "sheet": "Calc", "column": "XFD"},
        {"op": "convert_table", "sheet": "Calc", "table": "Sales"},
        {"op": "delete_table", "sheet": "Calc", "table": "Sales"},
        {"op": "delete_sheet", "sheet": "New"},
    ]
    xlsx = build_xlsx(_spec(ops))
    expected = [
        ("rename", "Data", "Inputs"),
        ("add", "New", None),
        ("move", "Calc", 0),
        ("insert_row", "Calc", 1),
        ("delete_row", "Calc", 2),
        ("delete_row", "Calc", "4:7"),
        ("delete_row", "Calc", 3),
        ("insert_column", "Calc", "b"),
        ("delete_column", "Calc", "XFD"),
        ("convert_table", "Calc", "Sales"),
        ("delete_table", "Calc", "Sales"),
        ("delete_sheet", "New", None),
    ]
    assert sheet_ops(xlsx) == expected
    assert recorded_sheet_ops(xlsx) == expected


def test_workbook_without_ops_lists_none() -> None:
    assert sheet_ops(build_xlsx({"sheets": [{"name": "S", "cells": {"A1": 1}}]})) == []


@pytest.mark.parametrize(
    "op",
    [
        {"op": "delete", "sheet": "Data"},
        {"op": "rename", "sheet": "Data"},
        {"op": "move", "sheet": "Data", "to": -1},
        {"op": "move", "sheet": "Data", "to": True},
        {"op": "add", "sheet": "New", "to": 1},
        {"op": "add", "sheet": "New", "extra": 1},
        {"op": "add"},
        {"op": "insert_row", "sheet": "Data"},
        {"op": "insert_row", "sheet": "Data", "row": 0},
        {"op": "insert_row", "sheet": "Data", "to": 1},
        {"op": "delete_row", "sheet": "Data"},
        {"op": "delete_row", "sheet": "Data", "row": 0},
        {"op": "delete_row", "sheet": "Data", "row": True},
        {"op": "delete_row", "sheet": "Data", "row": 2, "count": 0},
        {"op": "delete_row", "sheet": "Data", "row": 2, "count": True},
        {"op": "delete_row", "sheet": "Data", "row": "2:3", "count": 2},
        {"op": "delete_row", "sheet": "Data", "row": "3:2"},
        {"op": "delete_row", "sheet": "Data", "row": "2:3"},
        {"op": "insert_row", "sheet": "Data", "row": 2, "count": 2},
        {"op": "insert_column", "sheet": "Data"},
        {"op": "insert_column", "sheet": "Data", "column": "XFE"},
        {"op": "insert_column", "sheet": "Data", "column": "A1"},
        {"op": "delete_column", "sheet": "Data", "column": 2},
        {"op": "delete_column", "sheet": "Data", "row": 2},
        {"op": "delete_sheet", "sheet": "Data", "to": 1},
        {"op": "convert_table", "sheet": "Data"},
        {"op": "delete_table", "sheet": "Data", "table": ""},
        {"op": "delete_table", "sheet": "Data", "name": "T"},
        "add",
    ],
)
def test_unknown_or_malformed_op_is_refused(op: Any) -> None:
    with pytest.raises(ValueError, match="sheet op"):
        build_xlsx(_spec([op]))


def test_backend_refuses_a_malformed_recorded_op() -> None:
    xlsx = build_xlsx(_spec([{"op": "add", "sheet": "New"}]))
    tampered = patch_parts(xlsx, [("docProps/custom.xml", lambda text: text.replace('"add"', '"drop"'))])
    with pytest.raises(OracleError, match="unknown sheet op"):
        recorded_sheet_ops(tampered)


class _FakeGraph(GraphBackend):
    """Records the replay's calls against an in-memory sheet order; `stuck` sheets ignore a move."""

    def __init__(self, order: list[str], stuck: frozenset[str] = frozenset()) -> None:
        self.order, self.stuck = order, stuck
        self.calls: list[tuple[str, str, Any]] = []

    def _call(self, method: str, url: str, **kwargs: Any) -> Any:
        self.calls.append((method, url, kwargs.get("json")))
        name = unquote(url.rsplit("('", 1)[-1][:-2]).replace("''", "'") if url.endswith("')") else ""
        body = kwargs.get("json") or {}
        if method == "POST" and "name" in body:
            self.order.append(body["name"])
        elif method == "PATCH" and "name" in body:
            self.order[self.order.index(name)] = body["name"]
        elif method == "PATCH" and name not in self.stuck:
            self.order.remove(name)
            self.order.insert(body["position"], name)
        return SimpleNamespace(
            json=lambda: {"position": self.order.index(name)} if name in self.order else {}
        )


def test_graph_replay_keeps_the_canary_last() -> None:
    graph = _FakeGraph(["Calc", "Data", CANARY])
    ops = [("rename", "Data", "My Data"), ("add", "New", None), ("move", "New", 0)]
    graph._replay_sheet_ops("item", {}, ops)
    assert graph.order == ["New", "Calc", "My Data", CANARY]


def test_graph_replay_refuses_a_move_onto_the_canary_or_one_excel_ignored() -> None:
    with pytest.raises(OracleError, match="past the last sheet"):
        _FakeGraph(["Calc", "Data", CANARY])._replay_sheet_ops("item", {}, [("move", "Data", 2)])
    with pytest.raises(OracleError, match="left it at 1"):
        _FakeGraph(["Calc", "Data", CANARY], frozenset({"Data"}))._replay_sheet_ops(
            "item", {}, [("move", "Data", 0)]
        )


def test_graph_replay_inserts_a_row_above_the_given_one() -> None:
    graph = _FakeGraph(["S", CANARY])
    graph._replay_sheet_ops("item", {}, [("insert_row", "S", 3)])
    assert graph.calls == [
        ("POST", "item/workbook/worksheets('S')/range(address='3:3')/insert", {"shift": "Down"})
    ]
    assert graph.order == ["S", CANARY]


@pytest.mark.parametrize(
    ("op", "to", "url", "body"),
    [
        ("delete_row", 3, "item/workbook/worksheets('S')/range(address='3:3')/delete", {"shift": "Up"}),
        ("delete_row", "4:7", "item/workbook/worksheets('S')/range(address='4:7')/delete", {"shift": "Up"}),
        (
            "insert_column",
            "c",
            "item/workbook/worksheets('S')/range(address='C:C')/insert",
            {"shift": "Right"},
        ),
        (
            "delete_column",
            "C",
            "item/workbook/worksheets('S')/range(address='C:C')/delete",
            {"shift": "Left"},
        ),
    ],
)
def test_graph_replay_shifts_rows_and_columns(op: str, to: Any, url: str, body: dict[str, str]) -> None:
    graph = _FakeGraph(["S", CANARY])
    graph._replay_sheet_ops("item", {}, [(op, "S", to)])
    assert graph.calls == [("POST", url, body)]


def test_graph_replay_deletes_a_sheet_and_converts_or_deletes_a_table() -> None:
    graph = _FakeGraph(["S", "Other", CANARY])
    graph._replay_sheet_ops(
        "item",
        {},
        [("delete_sheet", "Other", None), ("convert_table", "S", "Sa'les"), ("delete_table", "S", "T1")],
    )
    assert graph.calls == [
        ("DELETE", "item/workbook/worksheets('Other')", None),
        ("POST", "item/workbook/tables('Sa%27%27les')/convertToRange", None),
        ("DELETE", "item/workbook/tables('T1')", None),
    ]


def test_graph_replay_never_deletes_the_canary_sheet() -> None:
    with pytest.raises(OracleError, match="canary"):
        _FakeGraph(["S", CANARY])._replay_sheet_ops("item", {}, [("delete_sheet", CANARY, None)])
