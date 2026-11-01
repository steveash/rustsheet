"""The fit lab on IM* text results (T-3525): complex parsing, part-by-part scoring, engine builds."""

from __future__ import annotations

import json
import math
import re
from pathlib import Path

import pytest

from rustsheet_tools.fitlab import cli, fit
from rustsheet_tools.fitlab.kernels import im_hyperbolic, registry
from rustsheet_tools.fitlab.score import Candidate, score
from rustsheet_tools.fitlab.sweep import Cell, Sweep, Workbook, parse_complex_text, select


@pytest.mark.parametrize(
    ("text", "parts"),
    [
        ("1.5-2i", (1.5, -2.0)),
        ("-0.5j", (0.0, -0.5)),
        ("1.5i", (0.0, 1.5)),
        ("i", (0.0, 1.0)),
        ("-i", (0.0, -1.0)),
        ("3", (3.0, 0.0)),
        ("2+i", (2.0, 1.0)),
        ("1.17873946358278E+307+1.6621900184285E+308i", (1.17873946358278e307, 1.6621900184285e308)),
        ("-1.2E-5-3.4E-7i", (-1.2e-5, -3.4e-7)),
    ],
)
def test_parse_complex_text_reads_im_results(text: str, parts: tuple[float, float]) -> None:
    assert parse_complex_text(text) == parts


@pytest.mark.parametrize("text", ["", "x", "1+2", "1+2k", "1.5ii", "#NUM!"])
def test_parse_complex_text_rejects_other_text(text: str) -> None:
    with pytest.raises(ValueError, match="not a complex number"):
        parse_complex_text(text)


def _book() -> Workbook:
    sheets = {"S": {"A1": '=IMSINH("1+i")', "A2": '=IMSINH("2")', "A3": '=IMSINH("3")', "A4": '=IMSINH("4")'}}
    oracle = {
        "S": {
            "A1": {"t": "s", "v": "0.634963914784736+1.29845758141598i"},
            "A2": {"t": "s", "v": "3.62686040784702"},
            "A3": {"t": "e", "v": "#NUM!"},
            "A4": {"t": "s", "v": "not complex"},
        }
    }
    return Workbook(sheets, oracle)


def test_select_keeps_complex_text_only_when_asked() -> None:
    plain, parts = Sweep(), Sweep()
    select("s/c", _book(), re.compile(r"^=IMSINH\((.*)\)$"), plain)
    select("s/c", _book(), re.compile(r"^=IMSINH\((.*)\)$"), parts, complex_text=True)
    assert plain.cells == [] and plain.non_numeric == 4
    assert [(c.address, c.expected, c.expected_im) for c in parts.cells] == [
        ("A1", 0.634963914784736, 1.29845758141598),
        ("A2", 3.62686040784702, 0.0),
    ]
    assert parts.non_numeric == 2  # the error (errors=False) and the text that is no number
    assert parts.cells[0].args == ("1+i",)


def _complex_cells() -> list[Cell]:
    return [
        Cell("s/c", "S", "A1", "=F()", (), 1.0, expected_im=2.0),
        Cell("s/c", "S", "A2", "=F()", (), 1.0, expected_im=2.0),
        Cell("s/c", "S", "A3", "=F()", (), 1.0, expected_im=2.0),
        Cell("s/c", "S", "A4", "=F()", (), 1.0, expected_im=2.0),
    ]


def test_complex_cells_score_part_by_part() -> None:
    answers = {
        "A1": "1+2i",  # exact
        "A2": complex(1.0, math.nextafter(2.0, 3.0)),  # imaginary part one ulp off
        "A3": "1+2.000000001i",  # within 1e-9 only
        "A4": 1.0,  # a bare number is no complex result
    }
    s = score(_complex_cells(), Candidate("c", lambda cell: answers[cell.address]))
    assert (s.exact, s.ulp, s.close, s.digits15, s.errors) == (1, 2, 3, 2, 1)
    worst = s.to_json(4)["worst"]
    assert worst[0]["cell"] == "S!A4" and "returned float" in worst[0]["note"]
    assert next(w for w in worst if w["cell"] == "S!A2")["actual_im"] == math.nextafter(2.0, 3.0)


def test_correctly_rounded_reference_writes_fifteen_digits() -> None:
    cell = Cell("s/c", "S", "A1", '=IMSINH("1+i")', ("1+i",), 0.0, expected_im=0.0)
    assert im_hyperbolic.correctly_rounded(cell) == complex(0.634963914784736, 1.29845758141598)
    far = Cell("s/c", "S", "A1", '=IMCOSH("800+0.5i")', ("800+0.5i",), 0.0, expected_im=0.0)
    with pytest.raises(ArithmeticError):
        im_hyperbolic.correctly_rounded(far)  # past the double range: #NUM!


def test_im_hyperbolic_fits_are_registered() -> None:
    fits = registry()
    assert fits["im-hyperbolic"].complex_text and fits["im-hyperbolic-fresh"].complex_text
    assert fits["im-hyperbolic-fresh"].cases[0] == "probes-numeric/t3525-im-hyperbolic-fresh-sweep"


def test_cli_scores_each_build_as_a_candidate(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    seen: list[tuple[Path | None, str]] = []

    def fake_engine(_case_ids: list[str], binary: Path | None = None, name: str = fit.ENGINE) -> Candidate:
        seen.append((binary, name))
        return Candidate(name, lambda _cell: 0.5, "engine" if name == fit.ENGINE else name)

    monkeypatch.setattr(fit, "engine_candidate", fake_engine)
    out = tmp_path / "erfc.json"
    argv = [
        "--case",
        "probes-numeric/t1040-erfc",
        "--pattern",
        r"^=ERFC\((.*)\)$",
        "--candidate",
        "math:erfc",
    ]
    assert cli.main([*argv, "--build", "alt=/bin/alt", "--json", str(out)]) == 0
    assert seen == [(None, fit.ENGINE), (Path("/bin/alt"), "engine build: alt")]
    names = {c["name"] for c in json.loads(out.read_text())["candidates"]}
    assert "engine build: alt" in names
    with pytest.raises(SystemExit, match="NAME=PATH"):
        cli.main([*argv, "--build", "no-path"])


def test_engine_text_on_an_error_cell_is_a_miss_not_a_crash(monkeypatch: pytest.MonkeyPatch) -> None:
    shown = {"S!A1": {"actual": '"1+2i"'}, "S!A2": {"actual": '"3-4i"'}}
    monkeypatch.setattr(fit, "show_cells", lambda _case, _binary=None: shown)
    engine = fit.engine_candidate(["s/c"])
    complex_cell = Cell("s/c", "S", "A1", "=F()", (), 1.0, expected_im=2.0)
    error_cell = Cell("s/c", "S", "A2", "=F()", (), math.nan, "#NUM!")
    s = score([complex_cell, error_cell], engine)
    assert s.exact == 1
    assert next(r for r in s.results if r.cell.address == "A2").note == "returned str"
