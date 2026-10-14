"""T-1525: score solver models for XIRR, IRR, RATE and YIELD against Excel (excel-graph).

Research record, not a tool: re-implements each candidate in IEEE double (Python floats) and counts
the oracle cells it reproduces bit for bit. Run from the repo root:

    python3 docs/research/t1525-solver-fit.py

Data: probes-numeric/m9-fin-irr-rate (T-1009) and probes/solver-iteration-traces (T-1525).
Findings are recorded in docs/excel-semantics/financial.md ("XIRR is a bracketing bisection",
"IRR, RATE and XIRR stop early", "YIELD with one coupon period left is closed form").
"""

import json
import math
import re
from collections.abc import Callable

SUITES = "eval/suites/"
CASES = [
    "probes-numeric/cases/m9-fin-irr-rate",
    "probes/cases/solver-iteration-traces",
    # Out-of-sample XIRR literals from earlier probes (the fit used only the two cases above).
    "probes/cases/xirr-xnpv-whole-column-nogaps",
    "probes/cases/fin-argument-modes",
    "probes/cases/date-system-1900-financial-daycount",
    "probes/cases/xirr-mismatched-empties",
    "probes/cases/xirr-early-dates",
]
Cell = tuple[str, str, object]


def cells() -> list[Cell]:
    """(id, formula, oracle value or 'ERR') for every cell of the two cases."""
    out: list[Cell] = []
    for case in CASES:
        spec = json.load(open(SUITES + case + ".case.json"))
        oracle = json.load(open(SUITES + case + ".oracle.json"))["sheets"]
        for sheet in spec["sheets"]:
            for addr, formula in sheet["cells"].items():
                if not isinstance(formula, str):
                    continue
                got = oracle.get(sheet["name"], {}).get(addr)
                value = got["v"] if got and got["t"] == "n" else "ERR"
                out.append((f"{case.split('/')[-1]}!{sheet['name']}!{addr}", formula, value))
    return out


def nums(text: str) -> list[float]:
    return [float(x) for x in text.split(",")]


# ----------------------------------------------------------------------------- XIRR


def xnpv(rate: float, values: list[float], dates: list[float]) -> float | None:
    """Excel's XNPV sum; None where a power is not real."""
    total = 0.0
    for value, date in zip(values, dates, strict=True):
        base = 1 + rate
        if base <= 0:
            return None
        try:
            total += value / base ** ((date - dates[0]) / 365)
        except (ZeroDivisionError, OverflowError):
            return None
    return total


def bracket(values: list[float], dates: list[float], guess: float) -> tuple[float, float] | None:
    """The first pair of consecutive chain points whose XNPV signs differ.

    The chain starts at 0. Upward it is s, 2s, 4s, ... with s = guess (1e-5 for guess 0); it is
    walked only for guess >= 0. Downward it is -s (or the guess when the guess is negative), then
    each point halves its distance to -1: p -> (p - 1) / 2. Returns (previous point, new point).
    """
    f0 = xnpv(0.0, values, dates)
    if f0 is None:
        return None
    step = guess if guess != 0 else 1e-5

    def walk(first: float, nxt: Callable[[float], float]) -> tuple[float, float] | None:
        prev, point = 0.0, first
        for _ in range(100):  # the expansion cap is not probed (T-1525 Outcome)
            fp = xnpv(point, values, dates)
            if fp is None:
                return None
            if (fp > 0) != (f0 > 0):
                return prev, point
            prev, point = point, nxt(point)
        return None

    if guess >= 0:
        found = walk(step, lambda p: p * 2)
        if found:
            return found
        if step >= 1:
            return None
        return walk(-step, lambda p: (p - 1) / 2)
    return walk(guess, lambda p: (p - 1) / 2)


def xirr_model(
    values: list[float],
    dates: list[float],
    guess: float,
    stop: Callable[[float, float, float, list[float]], bool],
) -> float | None:
    """Bisection on the bracket; `a` starts at its upper end, m = a + (b - a) / 2.

    Each step replaces the end whose XNPV has the sign of f(m). Starting `a` at the lower end, or
    m = (a + b) / 2, moves some midpoints by an ulp (X!D9, X!D11, Calc!A643).
    """
    found = bracket(values, dates, guess)
    if found is None:
        return None
    b, a = sorted(found)
    fa = xnpv(a, values, dates)
    assert fa is not None
    for _ in range(200):
        m = a + (b - a) / 2
        fm = xnpv(m, values, dates)
        if fm is None:
            return None
        if stop(m, abs(b - a) / 2, fm, values):
            return m
        if (fm > 0) == (fa > 0):
            a, fa = m, fm
        else:
            b = m
    return None


XIRR_STOPS: dict[str, Callable[[float, float, float, list[float]], bool]] = {
    # The fitted rule: half-width (= |m - previous midpoint|) and XNPV both small.
    "h<5e-9+1e-8|m| and |f|<2e-8*max|v|": lambda m, h, f, v: (
        h < 5e-9 + 1e-8 * abs(m) and abs(f) < 2e-8 * max(map(abs, v))
    ),
    "h<5e-9+1e-8|m| and |f|<1.5e-8*sum|v|": lambda m, h, f, v: (
        h < 5e-9 + 1e-8 * abs(m) and abs(f) < 1.5e-8 * sum(map(abs, v))
    ),
    # Rejected single-criterion rules.
    "h<5e-9+1e-8|m|": lambda m, h, f, v: h < 5e-9 + 1e-8 * abs(m),
    "h<1e-8*|m|": lambda m, h, f, v: h < 1e-8 * abs(m),
    "h<6e-9": lambda m, h, f, v: h < 6e-9,
    "|f|<2e-8*max|v|": lambda m, h, f, v: abs(f) < 2e-8 * max(map(abs, v)),
}


def score_xirr(all_cells: list[Cell]) -> None:
    pat = re.compile(r"=XIRR\(\{([^}]*)\},\{([^}]*)\}(?:,([^)]*))?\)")
    rows = []
    for cid, formula, value in all_cells:
        m = pat.fullmatch(formula)
        if m:
            guess = float(m.group(3)) if m.group(3) else 0.1
            rows.append((cid, nums(m.group(1)), nums(m.group(2)), guess, value))
    print(f"XIRR: {len(rows)} cells ({sum(r[4] == 'ERR' for r in rows)} #NUM!)")
    for name, stop in XIRR_STOPS.items():
        exact = 0
        for _cid, values, dates, guess, value in rows:
            got = xirr_model(values, dates, guess, stop)
            exact += (got is None and value == "ERR") or got == value
        print(f"  {exact:4d}/{len(rows)}  bisection, stop {name}")


# ----------------------------------------------------------------------------- IRR


def npv_v(values: list[float], v: float) -> float:
    total = 0.0
    for c in reversed(values):
        total = total * v + c
    return total


def dnpv_v(values: list[float], v: float) -> float:
    total = 0.0
    for i in range(len(values) - 1, 0, -1):
        total = total * v + i * values[i]
    return total


def npv_r(values: list[float], r: float) -> float:
    return sum(c / (1 + r) ** i for i, c in enumerate(values))


def dnpv_r(values: list[float], r: float) -> float:
    return sum(-i * c / (1 + r) ** (i + 1) for i, c in enumerate(values))


def iterates(step: Callable[[float], float], start: float, out: Callable[[float], float]) -> list[float]:
    """Up to 60 iterates of `step`, mapped by `out`; stops at the first failure."""
    seq, u = [], start
    for _ in range(60):
        try:
            u = step(u)
            seq.append(out(u))
        except (ZeroDivisionError, OverflowError, ValueError):
            break
        if not math.isfinite(u):
            break
    return seq


def irr_candidates(values: list[float], g: float) -> dict[str, list[float]]:
    """Every iterate of each candidate; a cell counts if Excel's value is any iterate."""
    v0, x0 = 1 / (1 + g), 1 + g
    cands = {
        "Newton in r": iterates(lambda r: r - npv_r(values, r) / dnpv_r(values, r), g, lambda r: r),
        "Newton in v=1/(1+r)": iterates(
            lambda v: v - npv_v(values, v) / dnpv_v(values, v), v0, lambda v: 1 / v - 1
        ),
        "Newton in x=1+r": iterates(
            lambda x: x - npv_r(values, x - 1) / dnpv_r(values, x - 1), x0, lambda x: x - 1
        ),
    }
    for h in (1e-4, 1e-6, 1e-8):
        cands[f"FD Newton in v, h={h}v"] = iterates(
            lambda v, h=h: v - npv_v(values, v) * v * h / (npv_v(values, v * (1 + h)) - npv_v(values, v)),
            v0,
            lambda v: 1 / v - 1,
        )
    return cands


def score_irr(all_cells: list[Cell]) -> None:
    pat = re.compile(r"=IRR\(\{([^}]*)\}(?:,([^)]*))?\)")
    rows = []
    for _cid, formula, value in all_cells:
        m = pat.fullmatch(formula)
        if m and value != "ERR":
            rows.append((nums(m.group(1)), float(m.group(2)) if m.group(2) else 0.1, value))
    x_exact = 0
    for _values, _g, value in rows:
        assert isinstance(value, float)
        x = 1 + value
        x_exact += (x - 1) == value
    print(f"IRR: {len(rows)} numeric cells; value + 1 - 1 == value (result is x - 1): {x_exact}")
    totals: dict[str, int] = {}
    for values, g, value in rows:
        for name, seq in irr_candidates(values, g).items():
            totals[name] = totals.get(name, 0) + (value in seq)
    for name, hits in totals.items():
        print(f"  {hits:4d}/{len(rows)}  {name} (Excel's value is some iterate)")


# ----------------------------------------------------------------------------- RATE


def rate_f(r: float, n: float, pmt: float, pv: float, fv: float, t: float) -> float:
    x = (1 + r) ** n
    return pv * x + pmt * (1 + r * t) * (x - 1) / r + fv


def rate_df(r: float, n: float, pmt: float, pv: float, fv: float, t: float) -> float:
    x, x1 = (1 + r) ** n, (1 + r) ** (n - 1)
    return pv * n * x1 + pmt * t * (x - 1) / r + pmt * (1 + r * t) * (n * x1 * r - (x - 1)) / (r * r)


def score_rate(all_cells: list[Cell]) -> None:
    pat = re.compile(r"=RATE\(([^()A-Z]*)\)")
    rows = []
    for _cid, formula, value in all_cells:
        m = pat.fullmatch(formula)
        if m and value != "ERR":
            args = nums(m.group(1))
            args += [0.0, 0.0, 0.1][len(args) - 3 :]
            rows.append((args, value))
    hits = {"Newton in r (Excel's FV form)": 0, "Newton in r, FD h=1e-8": 0}
    for (n, pmt, pv, fv, t, g), value in rows:
        a = (n, pmt, pv, fv, t)
        if value in iterates(lambda r: r - rate_f(r, *a) / rate_df(r, *a), g, lambda r: r):
            hits["Newton in r (Excel's FV form)"] += 1
        fd = iterates(
            lambda r: r - rate_f(r, *a) * 1e-8 / (rate_f(r + 1e-8, *a) - rate_f(r, *a)), g, lambda r: r
        )
        hits["Newton in r, FD h=1e-8"] += value in fd
    print(f"RATE: {len(rows)} numeric literal cells")
    for name, count in hits.items():
        print(f"  {count:4d}/{len(rows)}  {name} (Excel's value is some iterate)")


# ----------------------------------------------------------------------------- YIELD


def score_yield_one_period(all_cells: list[Cell]) -> None:
    """Sheet Y columns A and B: one coupon period left (30/360 day counts written out)."""
    shapes = {
        "Y!A": (0.05, 2, 61.0, 180.0, 119.0),  # 2024-03-01 -> 2024-06-30, previous coupon 2023-12-30
        "Y!B": (0.06, 1, 181.0, 360.0, 180.0),  # 2024-03-01 -> 2024-08-31, DSR = DAYS360 = 180, not E - A
    }
    exact = total = 0
    for cid, formula, value in all_cells:
        for key, (rate, freq, a, e, dsr) in shapes.items():
            if f"!{key}" in cid and re.match(r"=YIELD\(", formula):
                pr = float(formula.split(",")[7])
                y = (
                    ((100 / 100 + rate / freq) - (pr / 100 + (a / e * rate / freq)))
                    / (pr / 100 + (a / e) * (rate / freq))
                    * (freq * e / dsr)
                )
                total += 1
                exact += y == value
    print(f"YIELD one coupon period, documented closed form: {exact}/{total} bit-exact")


if __name__ == "__main__":
    data = cells()
    score_xirr(data)
    score_irr(data)
    score_rate(data)
    score_yield_one_period(data)
