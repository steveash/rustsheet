"""The solver fits (T-1525): sweep literals, the XIRR model, and the scores the docs quote."""

from __future__ import annotations

import datetime
import math

import pytest

from rustsheet_tools.fitlab import fit
from rustsheet_tools.fitlab.kernels import registry, solvers
from rustsheet_tools.fitlab.sweep import UnparsedError, Workbook, parse_arg, serial_date


def test_parse_arg_reads_array_literals_and_dates() -> None:
    book = Workbook({}, {})
    assert parse_arg('{-1,1.5;"a",TRUE}', "S", book) == [-1.0, 1.5, "a", True]
    assert parse_arg("DATE(2024,3,1)", "S", book) == 45352.0
    assert parse_arg("DATE(1900,3,1)", "S", book) == 61.0  # after Excel's phantom 1900-02-29
    with pytest.raises(UnparsedError):
        parse_arg("DATE(2024,2,30)", "S", book)


def test_day_counts_and_schedule() -> None:
    settle, maturity = datetime.date(2024, 3, 1), datetime.date(2024, 8, 31)
    assert solvers.days360(datetime.date(2023, 8, 31), settle, european=False) == 181
    assert solvers.days360(settle, maturity, european=False) == 180  # d2 = 31 stays when d1 < 30
    assert solvers.days360(settle, maturity, european=True) == 179
    assert solvers.schedule(settle, maturity, 1) == (datetime.date(2023, 8, 31), maturity, 1)
    # YIELD's one-period day counts on basis 0: A 181, E 360, DSR 180 (not E - A = 179).
    assert solvers.day_counts(settle, maturity, 1, 0, period_e=True)[:4] == (181.0, 360.0, 180.0, 179.0)


def test_xirr_model_on_a_known_root() -> None:
    # Default guess, root exactly 0.1: the bracket [0, 0.1] bisected to 3355443 / 2^25.
    got = solvers.xirr_model([-1000.0, 1100.0], [45000.0, 45365.0], 0.1, solvers.fitted_stop())
    assert got == 3355443 / 2**25
    with pytest.raises(solvers.NoResultError):  # guess -0.5 walks only toward -1
        solvers.xirr_model([-1000.0, 1100.0], [45000.0, 45365.0], -0.5, solvers.fitted_stop())


def test_rpow_rejects_a_complex_power() -> None:
    with pytest.raises(solvers.NoResultError):
        solvers.rpow(-0.5, 0.5)


def _scores(name: str) -> dict[str, int]:
    result = fit.run(registry()[name], engine=False)
    return {s.name: s.exact for s in result.scores} | {"cells": len(result.sweep.cells)}


@pytest.mark.usefixtures("real_cases")
def test_xirr_fit_scores() -> None:
    s = _scores("xirr")
    assert s["cells"] == 208
    assert s["bisect, stop h<5e-9+1e-8|m| & |f|<2e-8*max|v| (fitted)"] == 208
    assert s["bisect, stop h<5e-9+1e-8|m| only"] == 195
    assert s["bisect fitted, m=(a+b)/2"] == 206
    oos = _scores("xirr-oos")
    assert oos["cells"] == 11
    assert oos["bisect, stop h<5e-9+1e-8|m| & |f|<2e-8*max|v| (fitted)"] == 8


@pytest.mark.usefixtures("real_cases")
def test_xirr_num_errors_agree() -> None:
    agree, wrong = solvers.xirr_error_agreement(solvers.FIT_CASES)
    assert (agree, wrong) == (19, [])
    agree, wrong = solvers.xirr_error_agreement(solvers.XIRR_OOS_CASES)
    assert agree == 8  # the model solves three cells Excel rejects in its input checks
    assert len(wrong) == 3


@pytest.mark.usefixtures("real_cases")
def test_yield_one_period_closed_form_is_exact_on_every_basis() -> None:
    s = _scores("yield-one-period")
    assert s["cells"] == 60
    assert s["closed form, DSR, E = period on bases 1-3 (fitted)"] == 60


@pytest.mark.usefixtures("real_cases")
def test_irr_results_are_x_minus_one() -> None:
    cells = fit.run(registry()["irr"], engine=False, only="^$").sweep.cells
    assert len(cells) == 263
    assert all((1 + c.expected) - 1 == c.expected for c in cells)


@pytest.mark.usefixtures("real_cases")
def test_rate_results_are_rates() -> None:
    cells = fit.run(registry()["rate"], engine=False, only="^$").sweep.cells
    assert len(cells) == 136
    assert sum((1 + c.expected) - 1 == c.expected for c in cells) == 7


def test_iterate_converges_or_reports_no_result() -> None:
    root = solvers.iterate(lambda u: (u + 2 / u) / 2, 1.0, None)  # a fixed point of the step
    assert abs(root - math.sqrt(2)) <= math.ulp(root) and (root + 2 / root) / 2 == root
    assert solvers.iterate(lambda u: u / 2, 1.0, 3) == 0.125
    with pytest.raises(solvers.NoResultError):
        solvers.iterate(lambda u: u * 1e200, 1.0, None)  # overflows to infinity
    with pytest.raises(solvers.NoResultError):
        solvers.iterate(lambda u: -u, 1.0, None)  # oscillates for ever


def test_floats_rejects_text_bool_and_scalars() -> None:
    assert solvers.floats([1.0, -2.5]) == [1.0, -2.5]
    for bad in ([1.0, "2"], [True], 3.0, [None]):
        with pytest.raises(TypeError):
            solvers.floats(bad)


def test_regula_falsi_finds_a_price_root() -> None:
    bond: solvers.Bond = (0.05, 100.0, 2, 61.0, 180.0, 119.0, 21)
    y = 0.07861731922256075
    target = solvers.price(y, bond)
    got = solvers.regula_falsi(lambda r: solvers.price(r, bond), target)
    assert abs(got - y) < 1e-12


def test_schedule_and_serial_bounds() -> None:
    with pytest.raises(UnparsedError):
        solvers.schedule(datetime.date(2024, 3, 1), datetime.date(2024, 8, 31), 3)
    with pytest.raises(UnparsedError):
        serial_date(math.inf)
    with pytest.raises(UnparsedError):
        serial_date(1e12)
    assert serial_date(45352.0) == datetime.date(2024, 3, 1)
