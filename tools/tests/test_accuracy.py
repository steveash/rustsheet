"""Gate A's generator (T-3357, SPEC-0029): literals, sampler, definitions, batches, fill, refusal."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import random
import subprocess
from collections.abc import Sequence
from decimal import Decimal
from pathlib import Path
from typing import Any

import mpmath
import pytest
import yaml

from rustsheet_tools import repo_root, snapshot
from rustsheet_tools.accuracy import cli as accuracy_cli
from rustsheet_tools.accuracy import literal
from rustsheet_tools.accuracy.defs.annuity import RATE, rate
from rustsheet_tools.accuracy.defs.student_t import T_DIST, T_INV, cdf, density, quantile
from rustsheet_tools.accuracy.model import (
    REGISTRY,
    Definition,
    DomainError,
    Interval,
    MathArg,
    NoValueError,
    Param,
)
from rustsheet_tools.accuracy.sampler import STRATA, Sampler, point_count, seed, stratum_counts
from rustsheet_tools.accuracy.tables import batch_files, compute, draw_points, verify_batch, write_batch
from rustsheet_tools.fitlab.sweep import RefusedSweepError, load_sweep, refuse_accuracy
from rustsheet_tools.oracle import cli as oracle_cli
from rustsheet_tools.oracle.backends import OracleInfo


def mp(x: str | float) -> mpmath.mpf:
    return mpmath.mpf(x)


def value(definition: Definition, args: Sequence[literal.Arg]) -> mpmath.mpf:
    result = compute(definition, args)
    assert result.value is not None, result
    with mpmath.workdps(80):  # all 50 digits
        return mpmath.mpf(result.value)


def close(a: mpmath.mpf, b: str | float | mpmath.mpf, rel: float) -> bool:
    with mpmath.workdps(80):
        return bool(abs(a - mpmath.mpf(b)) <= rel * abs(mpmath.mpf(b)))


# -- literals ---------------------------------------------------------------------------------


def test_literals_have_at_most_15_digits_and_stay_writable() -> None:
    assert literal.text(1.87e-297) == "1.87E-297"
    assert literal.text(3937) == "3937"
    assert literal.text(True) == "TRUE"
    assert literal.text(10**16) == "1E+16"
    assert literal.round15(0.1 + 0.2) == 0.3
    assert literal.round15(9.999999999999999e307) == literal.MAX_LITERAL  # never 1E+308
    assert literal.round15(5e-324) == literal.MIN_LITERAL  # never a subnormal
    assert literal.neighbour(1.0, up=False) == 0.999999999999999
    assert literal.neighbour(1.0, up=True) == 1.00000000000001
    assert literal.neighbour(0.0, up=True) == literal.MIN_LITERAL
    assert literal.neighbour(1e10, up=True) == 10000000000.0001


# -- sampler ----------------------------------------------------------------------------------


def test_seed_is_sha256_of_function_and_batch() -> None:
    assert seed("T.INV", "s1") == int.from_bytes(hashlib.sha256(b"T.INV/s1").digest(), "big") % 2**63
    assert seed("T.INV", "s1") != seed("T.INV", "s2")


def test_strata_shares_and_counts() -> None:
    assert stratum_counts(120) == {"typical": 48, "tail": 18, "edge": 18, "extreme": 24, "log-uniform": 12}
    assert stratum_counts(300) == {"typical": 120, "tail": 45, "edge": 45, "extreme": 60, "log-uniform": 30}
    for n in (1, 7, 121, 200):
        assert sum(stratum_counts(n).values()) == n
    assert (point_count(T_DIST), point_count(T_INV), point_count(RATE)) == (120, 120, 300)


def _significant_digits(x: float) -> int:
    return len(Decimal(repr(x)).normalize().as_tuple().digits)


def test_drawn_arguments_are_writable_literals_inside_the_domain_except_edges() -> None:
    sampler = Sampler(T_INV, random.Random(1))
    for stratum in STRATA:
        for i in range(60):
            args = sampler.draw(stratum, i)
            for param, a in zip(T_INV.params, args, strict=True):
                f = float(a)
                assert f == 0 or literal.MIN_LITERAL <= abs(f) <= literal.MAX_LITERAL
                assert _significant_digits(f) <= 15
                assert float(literal.text(a)) == f
                if stratum != "edge":
                    assert param.domain.contains(f), (stratum, param.name, a)
            if stratum == "tail":  # p is T.INV's only bounded real: every tail is near 0 or 1
                p = float(args[0])
                assert min(p, 1 - p) <= 0.1 + 1e-12
    # the edges cover each bound, its inside neighbour and the invalid side
    edges = {(i, v) for i, v in sampler.edges}
    assert {(0, 0.0), (0, literal.MIN_LITERAL), (0, 1.0), (0, 0.999999999999999)} <= edges
    assert {(1, 1), (1, 2), (1, 0), (1, 10**10), (1, 10**10 - 1), (1, 10**10 + 1)} <= edges


def test_tail_map_puts_a_distribution_point_in_its_tail() -> None:
    sampler = Sampler(T_DIST, random.Random(3))
    tails = [sampler.draw("tail", i) for i in range(20)]
    with mpmath.workdps(30):
        probabilities = [cdf(mp(float(x)), mp(int(df))) for x, df, _ in tails]
    assert all(min(p, 1 - p) < 0.11 for p in probabilities)


# -- definitions against published values -----------------------------------------------------


def test_t_dist_matches_the_closed_forms_at_df_1_and_2() -> None:
    # df = 1 is the Cauchy distribution, df = 2 has F(x) = 1/2 + x / (2 sqrt(2 + x²)) (A&S 26.7).
    with mpmath.workdps(80):
        for x in (2.0, -3.5, 1e-5, -1e30):  # the doubles, as the table reads them
            cauchy = mpmath.mpf(1) / 2 + mpmath.atan(mp(x)) / mpmath.pi
            assert close(value(T_DIST, [x, 1, True]), cauchy, 1e-45)
            # 1/2 - |x|/(2s) = 1/(s(s + |x|)) with s = sqrt(2 + x²), free of cancellation in the tail.
            root = mpmath.sqrt(2 + mp(x) ** 2)
            lower = 1 / (root * (root + abs(mp(x))))
            assert close(value(T_DIST, [x, 2, True]), lower if x < 0 else 1 - lower, 1e-45)
            assert close(value(T_DIST, [x, 1, False]), 1 / (mpmath.pi * (1 + mp(x) ** 2)), 1e-45)
    # Excel's T.DIST(2,1,TRUE) = 0.8524163823495667 (statistics.md, T-1043), the double nearest the math.
    assert float(value(T_DIST, [2.0, 1, True])) == 0.8524163823495667


def test_t_dist_domain() -> None:
    assert compute(T_DIST, [2.0, 0, True]).why is not None  # df < 1
    assert compute(T_DIST, [2.0, 10**10 + 1, True]).why is not None  # cumulative cut at 1E10
    assert compute(T_DIST, [2.0, 10**10 + 1, False]).value is not None  # the density stays finite
    assert close(value(T_DIST, [2.0, 10**10, True]), "0.97724986803832", 1e-13)  # Excel 0.977249868038323


def test_t_inv_against_published_quantiles() -> None:
    with mpmath.workdps(80):
        assert close(value(T_INV, [0.75, 2]), mpmath.sqrt(mp(2) / 3), 1e-45)  # df 2 closed form
    assert close(value(T_INV, [0.975, 10]), "2.228138851986273", 1e-15)  # the t table's t(0.975, 10)
    # SPEC-0029's test point: T.INV(1.87E-297,3937) ≈ -40.26 (decision 0188 item 3).
    assert close(value(T_INV, [1.87e-297, 3937]), "-40.26", 1e-3)
    # df 1: the quantile is -cot(π p) exactly; Excel's T.INV(1E-300,1) = -3.183098861837907E299 (T-1531).
    with mpmath.workdps(80):
        assert close(value(T_INV, [1e-300, 1]), -mpmath.cot(mpmath.pi * mp(1e-300)), 1e-45)
    assert compute(T_INV, [0.0, 5]).why is not None
    assert compute(T_INV, [1.0, 5]).why is not None
    assert compute(T_INV, [0.5, 10**10 + 1]).why is not None
    with mpmath.workdps(60):
        assert close(cdf(quantile(mp("0.3"), mp(7)), mp(7)), "0.3", 1e-55)
        assert density(mp(0), mp(1)) == 1 / mpmath.pi


def test_rate_against_published_values() -> None:
    # Microsoft's RATE example: 4-year loan of 8000 at 200 a month is 0.77% a month.
    assert close(value(RATE, [48, -200.0, 8000.0, 0.0, 0]), "0.00770147248820165", 1e-13)
    # Decision 0188 item 2: Excel's 0.0431054691928255, the engine's #NUM!.
    got = value(RATE, [239, -14841.27102594, 344293.4825124, -155496.6323522, 0])
    assert close(got, "0.0431054691928255", 1e-14)
    # The root really zeroes the annuity's value (FV form) at 50 digits.
    with mpmath.workdps(80):
        r, n = got, 239
        fv_residual = 344293.4825124 * (1 + r) ** n - 14841.27102594 * ((1 + r) ** n - 1) / r - 155496.6323522
        assert abs(fv_residual) < mpmath.mpf(10) ** -40
    assert compute(RATE, [10, 100.0, 1000.0, 0.0, 0]).why is not None  # one sign: no rate
    with pytest.raises(NoValueError):  # two sign changes: no evidenced root choice
        rate(3, mp(-100), mp(100), mp(150), False)
    assert abs(value(RATE, [12, -100.0, 1200.0, 0.0, 0])) < 1e-300  # pv + n·pmt = 0: the root is 0
    # type 1 (payments at the start): 900 = 100·(1+r)·(1 - (1+r)^-10)/r
    due = value(RATE, [10, -100.0, 900.0, 0.0, 1])
    with mpmath.workdps(80):
        assert abs(900 - 100 * (1 + due) * (1 - (1 + due) ** -10) / due) < mpmath.mpf(10) ** -45


def test_definitions_follow_the_catalog_parameter_order() -> None:
    catalog = yaml.safe_load((repo_root() / "eval" / "catalog" / "functions.yaml").read_text())
    params = {f["name"]: [p["name"] for p in f.get("params", [])] for f in catalog["functions"]}
    for name, definition in REGISTRY.items():
        mine = [p.name for p in definition.params]
        assert params[name][: len(mine)] == mine, name


# -- batches ----------------------------------------------------------------------------------


def test_generation_is_deterministic() -> None:
    a = batch_files(T_DIST, "t", draw_points(T_DIST, "t", 30))
    b = batch_files(T_DIST, "t", draw_points(T_DIST, "t", 30))
    assert a == b
    assert a != batch_files(T_DIST, "u", draw_points(T_DIST, "u", 30))


def test_batch_files_follow_the_spec(tmp_path: Path) -> None:
    points = draw_points(T_INV, "s1", 10)
    case_path, math_path = write_batch(tmp_path, T_INV, "s1", points)
    case, sidecar = json.loads(case_path.read_text()), json.loads(math_path.read_text())
    assert case["format"] == "rustsheet-case/v1"
    assert case["check"] == [f"Calc!A{n}" for n in range(1, len(points) + 1)]
    assert sidecar["format"] == "rustsheet-math/v1"
    assert sidecar["source"]["seed"] == seed("T.INV", "s1")
    assert sidecar["source"]["input_sha256"] == hashlib.sha256(case_path.read_bytes()).hexdigest()
    for address, cell in sidecar["sheets"]["Calc"].items():
        formula = case["sheets"][0]["cells"][address]
        assert cell["f"] == formula
        assert cell["s"] in STRATA
        assert case["hypotheses"][f"Calc!{address}"].startswith(f"gate-a/{cell['s']}/")
        if cell["t"] == "n":  # 50 significant digits (fewer only when the value's digits end early)
            assert len(Decimal(cell["v"]).normalize().as_tuple().digits) <= 50
        else:
            assert cell["why"]
    with pytest.raises(FileExistsError):  # a batch is never rewritten
        write_batch(tmp_path, T_INV, "s1", [])


def test_verify_reports_a_changed_value(tmp_path: Path) -> None:
    case_path, math_path = write_batch(tmp_path, RATE, "s1", draw_points(RATE, "s1", 6))
    assert verify_batch(RATE, case_path, math_path) == []
    sidecar = json.loads(math_path.read_text())
    cell = next(c for c in sidecar["sheets"]["Calc"].values() if c["t"] == "n")
    cell["v"] = str(mpmath.mpf(cell["v"]) * (1 + mpmath.mpf("1e-30")))
    math_path.write_text(json.dumps(sidecar))
    problems = verify_batch(RATE, case_path, math_path)
    assert len(problems) == 1 and "recomputed" in problems[0]
    assert accuracy_cli.main(["--tables", str(tmp_path), "verify", "RATE"]) == 1


def _flaky(_args: Sequence[MathArg]) -> mpmath.mpf:
    # Converged at neither precision: the value moves with the working precision.
    return mpmath.mpf(1) + mpmath.mpf(10) ** (-mpmath.mp.dps + 20)


def _no_convergence(_args: Sequence[MathArg]) -> mpmath.mpf:
    raise mpmath.libmp.NoConvergence("series")


def _overflow(_args: Sequence[MathArg]) -> mpmath.mpf:
    return mpmath.mpf("1e400")


def test_points_that_do_not_converge_are_dropped_and_logged() -> None:
    param = (Param("x", "real", Interval(0, 1), typical=(0.1, 0.9)),)
    for kernel, why in (
        (_flaky, "not converged"),
        (_no_convergence, "did not converge"),
        (_overflow, "redraws"),
    ):
        log = io.StringIO()
        points = draw_points(
            Definition("FAKE", param, kernel, ("test",)), "s1", 3, stratum="typical", log=log
        )
        assert points == []
        assert why in log.getvalue()


# -- fit lab and oracle fill ------------------------------------------------------------------


def test_fit_lab_refuses_gate_a_tables_as_a_sweep() -> None:
    with pytest.raises(RefusedSweepError):
        load_sweep(["accuracy/T.INV/s1"], r"^=T\.INV\((.*)\)$")
    with pytest.raises(RefusedSweepError):
        load_sweep(["probes-numeric/x"], r"x", calls=["accuracy/RATE/s1"])
    refuse_accuracy(["accuracy/T.INV/s1"], holdout_only=True)
    refuse_accuracy(["probes-numeric/t1043-t"], holdout_only=False)


class _FakeExcel:
    name = "excel-graph"

    def available(self) -> tuple[bool, str]:
        return True, "fake"


def test_fill_writes_excels_values_and_leaves_the_rest_pending(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    binary = snapshot.eval_binary()
    monkeypatch.setenv("RUSTSHEET_EVAL_BIN", str(binary))
    (tmp_path / "eval" / "suites").mkdir(parents=True)
    accuracy = tmp_path / "eval" / "accuracy"
    accuracy.mkdir()
    (accuracy / "suite.toml").write_text((repo_root() / "eval" / "accuracy" / "suite.toml").read_text())
    write_batch(accuracy / "tables", T_INV, "s1", draw_points(T_INV, "s1", 4, stratum="typical"))
    monkeypatch.setattr(snapshot, "repo_root", lambda: tmp_path)

    excel = {"Calc": {"A1": {"t": "n", "v": -1.5}, "A3": {"t": "e", "v": "#NUM!"}}}  # A2, A4 refused

    def recalc(_backend: Any, _xlsx: bytes) -> tuple[dict[str, Any], OracleInfo, str]:
        return excel, OracleInfo(kind="excel", application="Microsoft Excel", version="16"), "ab"

    monkeypatch.setattr(oracle_cli, "_backend", lambda _name: _FakeExcel())
    monkeypatch.setattr(oracle_cli, "_recalc_cached", recalc)
    args = argparse.Namespace(
        backend="excel-graph", suite="accuracy", case=None, limit=0, force=False, include_cached=False
    )
    assert oracle_cli.cmd_fill(args) == 0
    entry = snapshot.suite_cases("accuracy")[0]
    assert entry["id"] == "accuracy/T.INV/s1.case.json"
    sidecar = json.loads(Path(entry["sidecar"]).read_text())
    assert sidecar["sheets"] == excel
    # The check reads A2 and A4 as pending: no Excel value, checked against the math only.
    out = subprocess.run(
        [str(binary), "--root", str(tmp_path), "accuracy", "--json", str(tmp_path / "a.json")],
        capture_output=True,
        text=True,
        check=False,
    )
    assert "4 points in 1 tables (pending 2)" in out.stdout, out.stdout + out.stderr


def test_cli_lists_and_generates(tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    assert accuracy_cli.main(["list"]) == 0
    assert capsys.readouterr().out.split() == ["RATE", "T.DIST", "T.INV"]
    assert accuracy_cli.main(["--tables", str(tmp_path), "generate", "t.dist", "--count", "5"]) == 0
    assert (tmp_path / "T.DIST" / "s1.case.json").exists()
    assert accuracy_cli.main(["--tables", str(tmp_path), "verify", "T.DIST"]) == 0
    with pytest.raises(DomainError):
        T_DIST.evaluate([mp(1), mp(0), True])
