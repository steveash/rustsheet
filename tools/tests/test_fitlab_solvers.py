"""The solver fits (T-1525): sweep literals, the XIRR model, and the scores the docs quote."""

from __future__ import annotations

import datetime
import math
from collections.abc import Callable

import pytest

from rustsheet_tools.fitlab import fit
from rustsheet_tools.fitlab.kernels import irr_fallback, irr_secant_d, registry, solvers, xirr, yield_caret
from rustsheet_tools.fitlab.sweep import (
    Arg,
    Cell,
    UnparsedError,
    Workbook,
    load_sweep,
    parse_arg,
    serial_date,
)


def test_parse_arg_reads_array_literals_and_dates() -> None:
    book = Workbook({}, {})
    assert parse_arg('{-1,1.5;"a",TRUE}', "S", book) == [-1.0, 1.5, "a", True]
    assert parse_arg("DATE(2024,3,1)", "S", book) == 45352.0
    assert parse_arg("DATE(1900,3,1)", "S", book) == 61.0  # after Excel's phantom 1900-02-29
    with pytest.raises(UnparsedError):
        parse_arg("DATE(2024,2,30)", "S", book)


def test_parse_arg_reads_a_nested_call_from_the_calls_cases() -> None:
    book = Workbook({}, {}, {"PMT(0.2,1,10000)": -12000.0})
    assert parse_arg("PMT(0.2, 1, 10000)", "S", book) == -12000.0  # whitespace is not part of the key
    with pytest.raises(UnparsedError):
        parse_arg("PMT(0.2,2,10000)", "S", book)


def test_rate_secant_needs_excel_operators_in_the_residual() -> None:
    # m9-fin-irr-rate A218, RATE(1,PMT(0.2,1,10000,-500,1),10000,-500,1), with Excel's PMT
    # (m9-fin-pmt A2): Excel 0.19999999999999138; FV's once-rounded residual is 157 ulp off (T-2527).
    args: tuple[Arg, ...] = (1.0, -9583.333333333332, 10000.0, -500.0, 1.0)
    assert solvers.rate_secant(solvers.fv_residual(solvers.TWICE))(*args) == 0.19999999999999138
    assert solvers.rate_secant(solvers.fv_residual())(*args) == 0.19999999999998702
    with pytest.raises(solvers.NoResultError):  # residual 0 everywhere: the flat secant, not the guess
        solvers.rate_secant(solvers.fv_residual(solvers.TWICE))(1.0, 0.0, 0.0, 0.0)
    with pytest.raises(solvers.ExcelError, match="guess"):
        solvers.rate_secant(solvers.fv_residual())(1.0, -1.0, 1.0, 0.0, 0.0, -1.0)


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
    got = xirr.xirr_model([-1000.0, 1100.0], [45000.0, 45365.0], 0.1, xirr.fitted_stop())
    assert got == 3355443 / 2**25
    with pytest.raises(solvers.NoResultError):  # guess -0.5 walks only toward -1
        xirr.xirr_model([-1000.0, 1100.0], [45000.0, 45365.0], -0.5, xirr.fitted_stop())


def test_xirr_first_value_sign_with_a_zero_first_value() -> None:
    # T-1583: a zero first value replaces the upper end every step, so the bracket [0, 0.1] is bisected
    # down to 0.1 / 2^25 (xirr-early-dates A10), where the first-value-blind rule converges to 0.1.
    values, dates = [0.0, -1000.0, 1100.0], [45000.0, 45000.0, 45365.0]
    got = xirr.xirr_model(values, dates, 0.1, xirr.span_stop(), first_value_sign=True)
    assert got == 0.1 / 2**25
    assert xirr.xirr_model(values, dates, 0.1, xirr.h_test()) == 3355443 / 2**25


def test_rpow_rejects_a_complex_power() -> None:
    with pytest.raises(solvers.NoResultError):
        solvers.rpow(-0.5, 0.5)


def _scores(name: str) -> dict[str, int]:
    result = fit.run(registry()[name], engine=False)
    return {s.name: s.exact for s in result.scores} | {"cells": len(result.sweep.cells)}


@pytest.mark.usefixtures("real_cases")
def test_xirr_fit_scores() -> None:
    s = _scores("xirr")
    assert s["cells"] == 227  # 208 numbers and 19 `#NUM!` cells
    assert s["bisect, stop h<5e-9+1e-8|m| & |f|<2e-8*max|v| (fitted)"] == 227
    assert s["bisect, stop h<5e-9+1e-8|m| only"] == 214
    assert s["bisect fitted, m=(a+b)/2"] == 225
    assert s["bisect, first-value sign, span-scaled |f| bound (T-1583)"] == 227
    oos = _scores("xirr-oos")
    # T-1591 added 30 cells (probes/xirr-span-zero-first), T-2045 29 (probes/xirr-back-dated-stop, whose two
    # range cells A2 and A31 the literal-array filter leaves out)
    assert oos["cells"] == 81
    assert oos["bisect, stop h<5e-9+1e-8|m| & |f|<2e-8*max|v| (fitted)"] == 46
    # T-1583 misses xirr-early-dates A7 (input check) and the root-0.1 cells at 20, 40 and 120 years (T-1591)
    assert oos["bisect, first-value sign, span-scaled |f| bound (T-1583)"] == 66
    assert oos["bisect, first-value sign, fitted stop"] == 48
    # T-2328: XNPV discounted with Excel's `^` puts the root-0.1 cells on Excel's side of the exact root.
    # The span factor is still needed (A8, 124 years), the step test too.
    assert s["bisect, first-value sign, span-scaled |f| bound, Excel ^ (T-2328)"] == 227
    assert oos["bisect, first-value sign, span-scaled |f| bound, Excel ^ (T-2328)"] == 69
    assert oos["bisect, first-value sign, fitted stop, Excel ^"] == 51
    assert oos["bisect, first-value sign, step test only, Excel ^"] == 74
    # T-2045: a span that counts flows dated before the first fits the back-dated cells but
    # whole-column-nogaps A4 and its copies (xirr-back-dated-stop A4, A5, A23).
    for span in ("earliest-to-latest years", "first-to-farthest years"):
        name = f"bisect, first-value sign, |f| bound scaled by {span}, Excel ^ (T-2045)"
        assert s[name] == 227
        assert oos[name] == 76


def test_xnpv_with_excels_power_at_an_exact_root() -> None:
    # probes/xirr-exact-root-sign A4 and A6: XNPV at XIRR's chain point 0.1, the exact root of the
    # literal flows (cut to 15 digits), is 2.22e-16 at 20 years and -3.33e-15 at 80 in Excel. The sign
    # picks the bracket [0.1, 0.2] or [0, 0.1]; libm's power gives -1.1e-16 at 20 years, the lower one.
    flows = parse_arg("{-1,6.727499949325611}", "S", Workbook({}, {}))
    assert flows == [-1.0, 6.72749994932561]
    values = solvers.floats(flows)
    assert xirr.xnpv(0.1, values, [45000.0, 52300.0], solvers.caret_pow) == 2.220446049250313e-16
    assert xirr.xnpv(0.1, values, [45000.0, 52300.0]) == -1.1102230246251565e-16
    long = [-1.0, 2048.40021458547]  # 2048.4002145854793, cut
    assert xirr.xnpv(0.1, long, [45000.0, 74200.0], solvers.caret_pow) == -3.3306690738754696e-15


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
def test_irr_secant_fit_scores() -> None:
    s = _scores("irr")
    assert s["secant in v, fitted"] == 189
    assert s["secant in v, no residual test"] == 181
    assert s["secant in v, no retry"] == 187
    assert s["Newton in v=1/(1+r), converged"] == 74


def test_irr_secant_d_needs_twice_rounded_npv() -> None:
    # T-3205: m9-fin-irr-rate Calc!A214 and gen-m9 67a4c03afd63f988 Calc!A395. With NPV in plain doubles
    # both stop one secant iterate early, 16 ulp off Excel.
    cells = [
        ([-1000.0, 230.0, 200.0, 125.0, 200.0, 10.0], 5.0, -0.10150067605359347),
        ([0.5, 0.1, -0.5], 0.5, -0.09501243788791103),
    ]
    plain = irr_secant_d.irr_secant_d(irr_secant_d.npv_v_running_plain)
    for flows, guess, excel in cells:
        assert irr_secant_d.irr_secant_d()(flows, guess, None) == excel
        assert plain(flows, guess, None) != excel


@pytest.mark.usefixtures("real_cases")
def test_irr_secant_d_fit_scores() -> None:
    s = _scores("irr")
    assert s["secant in d, shipped (T-3205)"] == 263
    assert s["secant in d, NPV in plain doubles (T-3120)"] == 262
    assert s["secant in d, unconditional zero-step stop"] == 257


def test_irr_secant_on_a_two_flow_root() -> None:
    # {-1, 1.1}: one secant step on a line lands on v = 1/1.1; the next step is below 1e-7.
    got = solvers.irr_secant_v()([-1.0, 1.1], 0.1, None)
    assert math.isclose(got, 0.1, rel_tol=1e-14)
    with pytest.raises(solvers.NoResultError):
        solvers.irr_secant_v(retry=False)([-20.0, -5.0], 0.1, None)


@pytest.mark.usefixtures("real_cases")
def test_rate_results_are_rates() -> None:
    cells = fit.run(registry()["rate"], engine=False, only="^$").sweep.cells
    assert len(cells) == 782  # nested PMT included (T-2527), error cells too
    assert sum(c.error is not None for c in cells) == 77
    assert sum((1 + c.expected) - 1 == c.expected for c in cells) == 40


@pytest.mark.usefixtures("real_cases")
def test_rate_secant_model_is_bit_exact_on_the_fit_set() -> None:
    # financial.md "RATE is a secant on the FV residual" quotes this score (T-2527).
    result = fit.run(registry()["rate"], engine=False, only=r"Excel operators, sum pv,pmt,fv$")
    (s,) = result.scores
    assert (s.exact, s.total) == (782, 782)


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
    bads: list[Arg] = [[1.0, "2"], [True], 3.0, [None]]
    for bad in bads:
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


@pytest.mark.usefixtures("real_cases")
def test_yield_back_chain_shares_a_frequency_independent_iterate() -> None:
    # T-1592: one analytic-Newton step back from Excel's YIELD gives the same iterate, to 1e-10,
    # for yield-solver-sweep S!A1 (frequency 2, 9 coupons) and S!F1 (frequency 1, 5 coupons).
    cells = fit.run(registry()["yield"], engine=False, only="^$").sweep.cells
    by_key = {c.key: c for c in cells if c.case.endswith("yield-solver-sweep.case.json")}
    a1, f1 = (solvers.newton_back_chain(by_key[k].args, by_key[k].expected, 1)[1] for k in ("S!A1", "S!F1"))
    assert abs(a1 - 0.0552344627) < 1e-10
    assert abs(a1 - f1) < 1e-10
    # Two and three steps back: S!E10 (redemption 100.5), and the three frequencies of Y!C2:E2.
    assert abs(solvers.newton_back_chain(by_key["S!E10"].args, by_key["S!E10"].expected, 2)[2] - a1) < 1e-8
    traces = {c.key: c for c in cells if c.case.endswith("solver-iteration-traces.case.json")}
    starts = [
        solvers.newton_back_chain(traces[k].args, traces[k].expected, 3)[3] for k in ("Y!C2", "Y!D2", "Y!E2")
    ]
    assert max(starts) - min(starts) < 1e-6


@pytest.mark.usefixtures("real_cases")
def test_yield_start_sweep_newton_from_a_shared_start() -> None:
    # T-2187: yield-start-sweep A1 (frequency 2) and B1 (frequency 1) step back to the same iterate as
    # yield-solver-sweep S!A1; C1 (frequency 4) is at PRICE's root. Run forward from that start,
    # analytic Newton returning the iterate before the first step below the cut gives Excel's value on
    # every cell of columns A to C (redemption 100 to 101 at frequency 2, 1 and 4) to 5e-14 relative (worst 1.2e-14).
    # T-2330: the stop is T-1592's short stop (a step below 1e-10 is not taken); the 6e-6 to 7e-6 "last steps"
    # T-2187 read as a cut are the steps before it.
    cells = load_sweep(["probes/yield-start-sweep"], r"^=YIELD\((.*)\)$").cells
    by_key = {c.key: c for c in cells}
    a1, b1 = (solvers.newton_back_chain(by_key[k].args, by_key[k].expected, 1)[1] for k in ("S!A1", "S!B1"))
    assert abs(a1 - 0.0552344627) < 1e-10
    assert abs(a1 - b1) < 1e-10
    price_of, _, pr = solvers._bond_price(by_key["S!C1"].args)
    assert abs(price_of(by_key["S!C1"].expected) - pr) < 1e-12
    swept = [c for c in cells if c.address[0] in "ABC"]
    assert len(swept) == 63
    newton = solvers._newton_yield_short(lambda *_args: a1, 1e-10)
    for cell in swept:
        assert abs(newton(*cell.args) - cell.expected) < 5e-14 * cell.expected, cell.key


@pytest.mark.usefixtures("real_cases")
def test_yield_start_one_input_two_steps_back() -> None:
    # T-2740: on yield-start-one-input every row with both twins below PRICE's root (rows 22-96) steps
    # back two analytic-Newton iterates to one start shared by frequency 2 (A) and 1 (B), to 1.3e-8 (asserted to 2e-8).
    # The start is the coupon rate at price 100 (row 41), linear in the rate (rows 42-66: 0.0046704412
    # + 1.01128042 R, to 6e-9), and falls with the term (row 96, maturity 2126).
    cells = load_sweep(["probes/yield-start-one-input"], r"^=YIELD\((.*)\)$").cells
    by_key = {c.key: c for c in cells}

    def start(row: int) -> float:
        a, b = (
            solvers.newton_back_chain(by_key[f"S!{c}{row}"].args, by_key[f"S!{c}{row}"].expected, 2)[2]
            for c in "AB"
        )
        assert abs(a - b) < 2e-8, row
        return (a + b) / 2

    assert abs(start(41) - 0.05) < 1e-8
    assert abs(start(42) - 0.0046704412) < 1e-8
    assert abs(start(52) - 0.0552344627) < 1e-8
    assert abs(start(66) - (0.0046704412 + 1.01128042 * 0.12)) < 1e-8
    assert abs(start(96) - 0.0511796361) < 1e-8
    # Exactly two: on row 52 the twins are 1.9e-5 apart one step back and 6.8e-4 three steps back.
    a, b = (solvers.newton_back_chain(by_key[f"S!{c}52"].args, by_key[f"S!{c}52"].expected, 3) for c in "AB")
    assert abs(a[1] - b[1]) > 1e-5
    assert abs(a[3] - b[3]) > 1e-4


def test_finite_difference_newton_reaches_the_analytic_root() -> None:
    # T-1592: every finite-difference variant (forward, central, absolute and relative step) converges on
    # yield-start-sweep A1 to the root analytic Newton finds, within the derivative's error.
    cell = next(
        c for c in load_sweep(["probes/yield-start-sweep"], r"^=YIELD\((.*)\)$").cells if c.key == "S!A1"
    )
    root = solvers._newton_yield(0.1, 1e-14)(*cell.args)
    for start, h, relative, central, tol in solvers.FD_NEWTON_YIELDS:
        got = solvers._fd_newton_yield(start, h, relative, central, tol)(*cell.args)
        assert abs(got - root) < 1e-9, (start, h, relative, central)


def test_yield_start_moves_with_the_basis_days_over_its_year() -> None:
    # T-1592: yield-start-sweep D2 (basis 1) and D11 (basis 0) step back to hidden starts 1.04e-6 apart.
    # A frequency-free annuity model over T = settlement-to-maturity days in the basis over 360 (basis 0)
    # or 365 (basis 1), 1559/360 against 1581/365, moves by the same amount to 1e-8, whatever its
    # compounding; over 366 or 365.25 days it would move 1e-6 to 1e-5 more.
    cells = load_sweep(["probes/yield-start-sweep"], r"^=YIELD\((.*)\)$").cells
    by_key = {c.key: c for c in cells}
    d2, d11 = (solvers.newton_back_chain(by_key[k].args, by_key[k].expected, 1)[1] for k in ("S!D2", "S!D11"))
    for m in (0, 1, 2, 4):
        model = solvers._annuity_model_root(m)
        moved = model(*by_key["S!D2"].args) - model(*by_key["S!D11"].args)
        assert abs(moved - (d2 - d11)) < 3e-8, m


def test_caret_priced_prices_with_excels_power() -> None:
    # T-2207: PRICE(DATE(2020,3,1),DATE(2030,3,1),0.06,0.07,130,1,0) is 108.22689722310892 in Excel
    # (ironcalc-calc FINANCIAL/PRICE_YIELD.xlsx PRICE_YIELD!H2); libm `pow` gives ...889. The power
    # goes back to libm once the `^`-priced call returns.
    bond = (0.06, 130.0, 1, 0.0, 360.0, 360.0, 10)
    assert yield_caret.caret_priced(lambda: solvers.price(0.07, bond))() == 108.22689722310892
    assert solvers.price(0.07, bond) == 108.22689722310889


def test_yield_fitted_start_and_short_stop() -> None:
    # T-1592: the two-steps-back starts of yield-start-one-input rows 22-96 (but 67) are the fitted
    # start to 2e-8 (the back-chain's precision), and Newton from it, returning the iterate whose next
    # step is below 1e-10 without taking that step, gives Excel's bits on the `^`-priced PRICE: A1 of
    # yield-solver-sweep (one step), yield-start-sweep C18 (a 6.66e-6 step taken, a 9.67e-11 one not)
    # and yield-start-one-input B21 (a 1.01e-10 step taken).
    one_input = {c.key: c for c in load_sweep(["probes/yield-start-one-input"], r"^=YIELD\((.*)\)$").cells}
    for row in (22, 41, 42, 52, 66, 68, 96):
        cell = one_input[f"S!A{row}"]
        back = solvers.newton_back_chain(cell.args, cell.expected, 2)[2]
        assert abs(back - solvers.fitted_yield_start(*cell.args)) < 2e-8, row
    model = yield_caret.caret_priced(solvers._newton_yield_short(solvers.fitted_yield_start, 1e-10))
    sweep = {c.key: c for c in load_sweep(["probes/yield-solver-sweep"], r"^=YIELD\((.*)\)$").cells}
    start = {c.key: c for c in load_sweep(["probes/yield-start-sweep"], r"^=YIELD\((.*)\)$").cells}
    for cell in (sweep["S!A1"], start["S!C18"], one_input["S!B21"]):
        assert model(*cell.args) == cell.expected, cell.key


def test_yield_unrounded_term_start_sits_one_ulp_above_rounded_once() -> None:
    # T-2974 attempts 2 and 4 (T-3152 names them in the fit): yield-solver-sweep S!C7 takes one step
    # from the start, so its bits show the start's last bit. The rounded-once start (decision 0162)
    # is one ulp below Excel's; the unrounded-term start (decision 0166, the engine's) is Excel's.
    sweep = {c.key: c for c in load_sweep(["probes/yield-solver-sweep"], r"^=YIELD\((.*)\)$").cells}
    cell = sweep["S!C7"]
    once = yield_caret.rounded_once_yield_start(*cell.args)
    assert yield_caret.unrounded_term_yield_start(*cell.args) == math.nextafter(once, math.inf)
    for start, exact in (
        (yield_caret.rounded_once_yield_start, False),
        (yield_caret.unrounded_term_yield_start, True),
    ):
        model = yield_caret.caret_priced(solvers._newton_yield_short(start, 1e-10))
        assert (model(*cell.args) == cell.expected) == exact, start.__name__


@pytest.mark.usefixtures("real_cases")
def test_yield_short_stop_cut_is_bracketed_by_two_cells() -> None:
    # T-2330: from the fitted start, yield-start-sweep C18 does not take a 9.67e-11 step (bit-exact) and
    # yield-start-one-input A5 takes a 1.0063e-10 one (1 ulp off), so the cut lies between them; 1e-10 is
    # inside. yield-start-rate-term B2 and A26, whose 7.54e-6 and 6.87e-6 steps ruled out the absolute and
    # relative cuts of T-2187, stop before a 9.13e-11 and a 6.52e-11 step not taken (1 ulp off each).
    def cell(case: str, key: str) -> Cell:
        return next(c for c in load_sweep([case], r"^=YIELD\((.*)\)$").cells if c.key == key)

    c18 = cell("probes/yield-start-sweep", "S!C18")
    a5 = cell("probes/yield-start-one-input", "S!A5")
    b2, a26 = (cell("probes/yield-start-rate-term", k) for k in ("S!B2", "S!A26"))

    def model(cut: float) -> Callable[..., float]:
        return yield_caret.caret_priced(solvers._newton_yield_short(solvers.fitted_yield_start, cut))

    def ulps(value: float, cell: Cell) -> float:
        return abs(value - cell.expected) / math.ulp(cell.expected)

    for cut in (9.68e-11, 1e-10, 1.0062e-10):
        assert model(cut)(*c18.args) == c18.expected
        assert ulps(model(cut)(*a5.args), a5) <= 1
        assert ulps(model(cut)(*b2.args), b2) <= 1
        assert ulps(model(cut)(*a26.args), a26) <= 1
    assert ulps(model(9.66e-11)(*c18.args), c18) > 1e6  # C18 would take its step
    assert ulps(model(1.0064e-10)(*a5.args), a5) > 1e6  # A5 would stop before its step


def test_yield_start_term_whole_years_then_leftover_days() -> None:
    # T-2777: the start's term is whole years, then the leftover days over 360, 365 (basis 3) or the
    # maturity's calendar year (basis 1). On probe cells the days-over-a-year term misses visibly (a
    # 29 February settlement on bases 1, 0 and 4, rows 5, 37 and 41; 28 February settlements on basis 0,
    # rows 46 and 47; basis 1 to a leap-year maturity, row 8) the fitted term lands within 4 ulp (the
    # lab's last bit varies with the platform's libm) and the old term at least 1e-13 relative off.
    probe = {c.key: c for c in load_sweep(["probes/yield-start-term-basis"], r"^=YIELD\((.*)\)$").cells}
    model = yield_caret.caret_priced(solvers._newton_yield_short(solvers.fitted_yield_start, 1e-10))

    def old_start(*args: Arg) -> float:
        settle, maturity, rate, pr, _, _, basis = solvers._bond(args)
        years = solvers._years_to_maturity(settle, maturity, basis)
        return (100 * rate + (100 - pr) / years) / ((100 + pr) / 2 - (100 - pr) / (4 * years))

    old = yield_caret.caret_priced(solvers._newton_yield_short(old_start, 1e-10))
    for key in ("S!B5", "S!C5", "S!A8", "S!B8", "S!A37", "S!C37", "S!A41", "S!C41", "S!A46", "S!C47"):
        cell = probe[key]
        assert abs(model(*cell.args) - cell.expected) <= 4 * math.ulp(cell.expected), key
        assert abs(old(*cell.args) - cell.expected) >= 1e-13 * abs(cell.expected), key
    assert solvers.start_years(datetime.date(2010, 1, 31), datetime.date(2020, 1, 30), 1) == 9 + 364 / 366
    assert solvers.start_years(datetime.date(2012, 2, 29), datetime.date(2019, 8, 15), 0) == 7 + 164 / 360
    assert solvers.start_term(datetime.date(2012, 2, 29), datetime.date(2019, 8, 15), 0) == (7, 164, 360)


def test_irr_fallback_newton_matches_the_engine_and_keeps_the_selection_rule() -> None:
    # The engine's fallback digits (T-2928): `{42,1,1,-1}` converges to -0.7467044766537408, the
    # correctly rounded root, where Excel returns -0.7467044766537371 (probes/irr-fallback-roots A12).
    engine = irr_fallback.irr_fallback_newton()
    assert engine([42.0, 1.0, 1.0, -1.0]) == -0.7467044766537408
    assert engine([0.0, 2.0, -1.0]) == pytest.approx(-0.5)  # leading zeros dropped (T-2727)
    with pytest.raises(solvers.NoResultError):  # P's root nearest v = 0 is negative: #NUM! (T-2727)
        engine([1.0, 3.0, -1.0])
    # A residual test on |P| makes the run on the 1E6-scaled {-656,1,1,1,1,1} reach the cap, as
    # Excel does (t2927-irr-fallback-sweep A103), where the engine's step test returns.
    scaled = [-656e6, 1e6, 1e6, 1e6, 1e6, 1e6]
    assert engine(scaled) == pytest.approx(-0.7072347712570631)
    with pytest.raises(solvers.NoResultError):
        irr_fallback.irr_fallback_newton(residual=1e-7)(scaled)


@pytest.mark.usefixtures("real_cases")
def test_yield_start_denominator_is_100t_minus_qt_over_2() -> None:
    # T-3215: yield-start-rate-term S!B30 and yield-start-term-basis S!C47 want a start one ulp below
    # the unrounded-term start (decision 0166), yield-one-step-start S!A37 one ulp above it. The
    # denominator `(100 T - q T/2) - q/4` in doubles gives all three Excel's bits.
    cases = ("yield-start-rate-term", "yield-start-term-basis", "yield-one-step-start")
    cells = {
        f"{case}!{c.key}": c
        for case in cases
        for c in load_sweep([f"probes/{case}"], r"^=YIELD\((.*)\)$").cells
    }
    old = yield_caret.caret_priced(solvers._newton_yield_short(yield_caret.unrounded_term_yield_start, 1e-10))
    new = yield_caret.caret_priced(
        solvers._newton_yield_short(yield_caret.hundred_t_minus_qt_yield_start, 1e-10)
    )
    for key, step in (
        ("yield-start-rate-term!S!B30", -1),
        ("yield-start-term-basis!S!C47", -1),
        ("yield-one-step-start!S!A37", 1),
    ):
        cell = cells[key]
        before = yield_caret.unrounded_term_yield_start(*cell.args)
        after = yield_caret.hundred_t_minus_qt_yield_start(*cell.args)
        assert after == math.nextafter(before, step * math.inf), key
        assert old(*cell.args) != cell.expected, key
        assert new(*cell.args) == cell.expected, key
