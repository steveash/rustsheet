"""The fit lab: ulp distance, sweep reading, candidate scoring and ranking, and the ERFC example."""

from __future__ import annotations

import json
import math
import re
from pathlib import Path
from typing import Any

import pytest

from rustsheet_tools import repo_root
from rustsheet_tools.fitlab import cli, fit, sweep
from rustsheet_tools.fitlab.kernels import erfc, registry
from rustsheet_tools.fitlab.score import Candidate, candidate, families, rank, score, ulp_distance
from rustsheet_tools.fitlab.sweep import Cell, Sweep, UnparsedError, Workbook, parse_arg, select, split_args
from rustsheet_tools.snapshot import parse_show

CASES = repo_root() / "eval" / "suites" / "probes-numeric" / "cases"


def test_ulp_distance_matches_the_harness() -> None:
    # crates/rustsheet-eval/src/value.rs `ulp_distance_crosses_zero`.
    tiny = math.ulp(0.0)
    assert ulp_distance(0.0, -0.0) == 0
    assert ulp_distance(tiny, -tiny) == 2
    assert ulp_distance(1.0, math.nextafter(1.0, 2.0)) == 1
    assert ulp_distance(1.0, 1.0 + 4 * 2**-52) == 4
    assert ulp_distance(2.0, math.nextafter(2.0, 0.0)) == 1  # across a binade
    assert ulp_distance(-1.0, 1.0) == 2 * ulp_distance(0.0, 1.0)


def _book() -> Workbook:
    sheets = {
        "Calc": {"A1": '=F(1.5,-2E3,TRUE,"a""b")', "A2": "=F($Z$1,Data!B1:C2)", "A3": "=F(1+1)", "Z1": 4},
        "Data": {"B1": 1, "C1": 2.5, "B2": None, "C2": "=G()"},
    }
    oracle = {
        "Calc": {f"A{i}": {"t": "n", "v": float(i)} for i in (1, 2, 3)},
        "Data": {"C2": {"t": "n", "v": 9.0}},
    }
    return Workbook(sheets, oracle)


def test_split_args_respects_nesting_and_strings() -> None:
    assert split_args('1, "a,b", F(2,3), {1,2}') == ["1", '"a,b"', "F(2,3)", "{1,2}"]
    assert split_args("") == []


def test_parse_arg_literals_and_references() -> None:
    book = _book()
    assert parse_arg("1E-10", "Calc", book) == 1e-10
    assert parse_arg("-.5", "Calc", book) == -0.5
    assert str(parse_arg("-0", "Calc", book)) == "0.0"  # Excel has no negative zero
    assert parse_arg("true", "Calc", book) is True
    assert parse_arg('"x""y"', "Calc", book) == 'x"y'
    assert parse_arg("$Z$1", "Calc", book) == 4.0
    assert parse_arg("Data!B1:C2", "Calc", book) == [1.0, 2.5, None, 9.0]  # row-major, formula → oracle
    with pytest.raises(UnparsedError):
        parse_arg("1+1", "Calc", book)


def test_select_parses_and_reports_unparsed_cells() -> None:
    found = Sweep()
    select("s/c.case.json", _book(), re.compile(r"^=F\((.*)\)$"), found)
    assert [c.address for c in found.cells] == ["A1", "A2"]
    assert found.cells[0].args == (1.5, -2000.0, True, 'a"b')
    assert found.cells[1].args == (4.0, [1.0, 2.5, None, 9.0])
    assert len(found.unparsed) == 1 and "A3" in found.unparsed[0]


def _cells(*pairs: tuple[float, float]) -> list[Cell]:
    return [Cell("s/c", "S", f"A{i}", f"=F({x})", (x,), e) for i, (x, e) in enumerate(pairs, 1)]


def test_score_counts_exact_ulp_close_and_errors() -> None:
    one_up = math.nextafter(1.0, 2.0)
    cells = _cells((1.0, 1.0), (2.0, one_up), (3.0, 1.0 + 1e-12), (4.0, 1.0), (5.0, 1.0))

    def f(x: float) -> float:
        if x == 5.0:
            raise ZeroDivisionError("boom")
        return 1.0 if x != 4.0 else math.inf

    s = score(cells, candidate("f", f), keep=2)
    assert (s.exact, s.ulp, s.close, s.errors) == (1, 2, 3, 1)  # x=2: one ulp; x=3: 1e-12 relative
    assert s.max_ulps is None  # an error and an infinity have no distance
    assert [r.cell.address for r in s.results] == ["A4", "A5"]  # worst kept: no distance first
    assert s.to_json(1)["worst"][0]["cell"] == "S!A4"


def test_rank_prefers_exact_then_ulp_then_worst() -> None:
    cells = _cells((1.0, 1.0), (2.0, 2.0))
    near = candidate("near", lambda x: math.nextafter(x, 9.0))
    right = candidate("right", lambda x: x)
    half = candidate("half", lambda x: x if x == 1.0 else x * 3)
    scores = [score(cells, c) for c in (near, half, right)]
    assert [s.name for s in rank(scores)] == ["right", "half", "near"]


def test_families_report_best_and_cumulative_union() -> None:
    cells = _cells((1.0, 1.0), (2.0, 2.0), (3.0, 3.0))
    cands = [
        Candidate("a1", lambda _: 1.0, "a"),
        Candidate("a2", lambda _: 2.0, "a"),
        Candidate("b1", lambda c: 3.0 if c.args[0] == 3.0 else 0.5, "b"),
    ]
    rows = families([score(cells, c) for c in cands])
    assert [(r["family"], r["family_union_exact"], r["cumulative_union_exact"]) for r in rows] == [
        ("a", 2, 2),
        ("b", 1, 3),
    ]


def test_parse_show_reads_the_harness_output() -> None:
    text = (
        "s/c.case.json  status=Fail  checked=2 passed=1 deviations=0  source=excel-graph\n"
        "Calc!A1              Pass\n    formula : =ERFC(1)\n    expected: 0.15729920705028513\n"
        "    actual  : 0.15729920705028513\n"
        "Calc!A2              NumericUlp\n    formula : =ERFC(2)\n    expected: 0.004677734981047266\n"
        "    actual  : 0.004677734981047265\n"
    )
    shown = parse_show(text)
    assert shown["Calc!A2"] == {
        "verdict": "NumericUlp",
        "formula": "=ERFC(2)",
        "expected": "0.004677734981047266",
        "actual": "0.004677734981047265",
    }


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


@pytest.mark.usefixtures("real_cases")
def test_erfc_fit_scores_libm_on_the_real_sweep() -> None:
    result = fit.run(registry()["erfc"], engine=False)
    assert len(result.sweep.cells) == 530 and not result.sweep.unparsed
    by_name = {s.name: s for s in result.scores}
    assert by_name["math.erfc"].ulp > 500  # libm is near Excel almost everywhere
    assert by_name["math.erfc"].exact > by_name["1 - math.erf"].exact


@pytest.mark.usefixtures("real_cases")
def test_erfc_tail_reproduces_the_t1187_grid() -> None:
    tail = registry()["erfc-tail"]
    small = fit.Fit(tail.name, tail.description, tail.cases, tail.pattern, erfc.libm_candidates, tail.where)
    result = fit.run(small, engine=False)
    assert len(result.sweep.cells) == 205  # T-1187: x >= 1 where Excel is not 0
    assert all(isinstance(c.args[0], float) and c.args[0] >= 1 for c in result.sweep.cells)


@pytest.mark.usefixtures("real_cases")
def test_cli_writes_json_and_ranks_the_engine(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    engine_value = {"probes-numeric/t1040-erfc.case.json": "0.5"}

    def fake_engine(case_ids: list[str]) -> Candidate:
        assert case_ids == ["probes-numeric/t1040-erfc.case.json"]
        return Candidate(fit.ENGINE, lambda c: float(engine_value[c.case]), "engine")

    monkeypatch.setattr(fit, "engine_candidate", fake_engine)
    out = tmp_path / "erfc.json"
    argv = ["--case", "probes-numeric/t1040-erfc", "--pattern", r"^=ERFC\((.*)\)$"]
    assert cli.main([*argv, "--candidate", "math:erfc", "--json", str(out)]) == 0
    data = json.loads(out.read_text())
    assert [c["name"] for c in data["candidates"]] == ["math:erfc", fit.ENGINE]
    assert data["cells"] == 265 and data["candidates"][0]["within_4_ulp"] > 250
