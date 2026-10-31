"""Math references for eval cells (T-3380, SPEC-0029 "Math references for eval cells")."""

from __future__ import annotations

import json
from collections.abc import Sequence
from pathlib import Path
from typing import Any

import mpmath
import pytest

from rustsheet_tools.accuracy import eval_math
from rustsheet_tools.accuracy.defs.student_t import T_DIST, cdf
from rustsheet_tools.accuracy.eval_math import case_math, fit_args, math_path, single_call, write_suite
from rustsheet_tools.accuracy.model import Definition, Interval, MathArg, NoValueError, Param
from rustsheet_tools.accuracy.tables import DroppedError, compute


def _case(tmp_path: Path, cells: dict[str, Any], oracle: dict[str, Any] | None) -> tuple[Path, Path]:
    case = tmp_path / "c.case.json"
    case.write_text(
        json.dumps({"format": "rustsheet-case/v1", "sheets": [{"name": "S", "cells": cells}], "expected": {}})
    )
    sidecar = tmp_path / "c.oracle.json"
    if oracle is not None:
        sidecar.write_text(json.dumps({"format": "rustsheet-oracle/v1", "sheets": {"S": oracle}}))
    return case, sidecar


def test_single_call_takes_one_whole_call_only() -> None:
    assert single_call("=T.DIST(1,2,TRUE)") == ("T.DIST", "1,2,TRUE")
    assert single_call("=t.dist(1,2,TRUE)") == ("T.DIST", "1,2,TRUE")
    assert single_call("=T.DIST(1,2,TRUE)+T.DIST(1,3,TRUE)") is None
    assert single_call("=T.DIST((1+1),2,TRUE)") == ("T.DIST", "(1+1),2,TRUE")
    assert single_call("=1+T.DIST(1,2,TRUE)") is None
    assert single_call("T.DIST(1,2,TRUE)") is None


def test_fit_args_follows_the_parameters() -> None:
    assert fit_args(T_DIST, [1.5, 3.0, True]) == [1.5, 3, True]
    assert fit_args(T_DIST, [1.5, 3.0, 1.0]) == "number for a logical"
    assert fit_args(T_DIST, [1.5, 2.5, True]) == "non-integer for an integer parameter"
    assert fit_args(T_DIST, [1.5, 3.0]) == "argument count"
    assert fit_args(T_DIST, ["1", 3.0, True]) == "non-numeric argument"
    assert fit_args(T_DIST, [True, 3.0, True]) == "logical for a number"


def test_case_math_writes_values_and_skips_what_it_cannot_read(tmp_path: Path) -> None:
    cells = {
        "A1": "=T.DIST(1.5,3,TRUE)",  # a value
        "A2": "=T.DIST(B1,3,TRUE)",  # a constant cell: read
        "A3": "=T.DIST(A1,3,TRUE)",  # a formula cell: never read
        "A4": "=T.DIST(1.5,2.5,TRUE)",  # non-integer df: skipped
        "A5": "=T.INV(0,3)",  # outside the domain: a math domain error
        "A6": "=NORM.DIST(1,0,1,TRUE)",  # no definition: not a candidate
        "A7": "=T.DIST(1.5,3,TRUE)",  # not scored (no oracle value): not a candidate
        "A8": "=T.DIST(B2,3,TRUE)",  # an arithmetic-over-literals formula cell: read
        "B1": 1.5,
        "B2": "=(3*2^-1)",
    }
    oracle = {a: {"t": "n", "v": 0.5} for a in ("A1", "A2", "A3", "A4", "A5", "A6", "A8")}
    case, sidecar = _case(tmp_path, cells, oracle)
    math = case_math("t/c.case.json", case, sidecar)
    got = math.cells["S"]
    assert set(got) == {"A1", "A2", "A5", "A8"}
    with mpmath.workdps(60):
        want = cdf(mpmath.mpf(1.5), mpmath.mpf(3))
        assert abs(mpmath.mpf(got["A1"]["v"]) - want) < mpmath.mpf("1e-45") * want
    assert got["A1"]["f"] == "=T.DIST(1.5,3,TRUE)"
    assert got["A2"]["v"] == got["A1"]["v"] == got["A8"]["v"]
    assert got["A5"]["t"] == "e" and got["A5"]["f"] == "=T.INV(0,3)"
    assert math.skipped == {"unparsed (UnparsedError)": 1, "non-integer for an integer parameter": 1}


def test_write_suite_is_deterministic_and_removes_only_its_own_stale_sidecars(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    with_math = tmp_path / "a"
    without = tmp_path / "b"
    foreign = tmp_path / "c"
    for d in (with_math, without, foreign):
        d.mkdir()
    a_case, a_sidecar = _case(with_math, {"A1": "=T.DIST(1.5,3,TRUE)"}, {"A1": {"t": "n", "v": 0.9}})
    b_case, b_sidecar = _case(without, {"A1": "=1+1"}, {"A1": {"t": "n", "v": 2}})
    c_case, c_sidecar = _case(foreign, {"A1": "=1+1"}, {"A1": {"t": "n", "v": 2}})
    math_path(b_sidecar).write_text(
        json.dumps(
            {"format": "rustsheet-math/v1", "source": {"generator": "rustsheet-accuracy"}, "sheets": {}}
        )
    )
    math_path(c_sidecar).write_text('{"format": "something else"}')
    entries = [
        {"id": f"t/{p.parent.name}", "path": str(p), "sidecar": str(s)}
        for p, s in ((a_case, a_sidecar), (b_case, b_sidecar), (c_case, c_sidecar))
    ]
    monkeypatch.setattr(eval_math, "suite_cases", lambda _suite: entries)
    write_suite("t", workers=1)
    first = math_path(a_sidecar).read_bytes()
    write_suite("t", workers=1)
    assert math_path(a_sidecar).read_bytes() == first
    data = json.loads(first)
    assert data["format"] == "rustsheet-math/v1"
    assert data["source"]["generator"] == "rustsheet-accuracy" and "seed" not in data["source"]
    assert set(data["sheets"]["S"]) == {"A1"} and "s" not in data["sheets"]["S"]["A1"]
    assert not math_path(b_sidecar).exists()  # ours and now empty: removed
    assert math_path(c_sidecar).exists()  # not ours: kept


def test_an_mpmath_value_error_drops_the_point_but_a_domain_error_does_not() -> None:
    def gives_up(_args: Sequence[MathArg]) -> mpmath.mpf:
        msg = "hypercomb() failed to converge to the requested 223 bits of accuracy"
        raise ValueError(msg)

    def several_roots(_args: Sequence[MathArg]) -> mpmath.mpf:
        raise NoValueError("several roots")

    x = Param("x", "real", Interval(0, 1))
    with pytest.raises(DroppedError, match="did not converge"):
        compute(Definition("GIVES.UP", (x,), gives_up, ("test",)), [0.5])
    with pytest.raises(NoValueError):
        compute(Definition("ROOTS", (x,), several_roots, ("test",)), [0.5])
    assert compute(Definition("GIVES.UP", (x,), gives_up, ("test",)), [2.0]).why == "x outside (0, 1)"
