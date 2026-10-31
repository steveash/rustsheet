"""Score T-3354's cases against Excel and 50-digit mpmath (decision 0188 R2's Gate A envelope).

Usage, from the repository root after `cargo build --release -p rustsheet_eval`:
    python3 docs/research/t3354/score.py probes-numeric/t3354-f-dist-sweep.case.json
    python3 docs/research/t3354/score.py probes/t3354-f-dist-tiny-x-huge-d2.case.json

A point is inside the envelope when the engine is within 15 significant digits of Excel's value (half a unit of the
15th digit) or within 1e-9 relative of the math (absolute 1e-300 near 0). Excel truncates d1 and d2 to integers.
Rows group into strata of 80, 60, 40 and 20 for the sweep; the probe case reports as one stratum.
"""

import re
import signal
import subprocess
import sys

from mpmath import betainc, exp, floor, log, loggamma, mp, mpf

mp.dps = 50
EPS = mpf("1e-9")
FLOOR = mpf("1e-300")
CELL = re.compile(
    r"S!([A-Z]+)(\d+)\s+\w+\n\s+formula : =(F\.DIST(?:\.RT)?)\(([^)]*)\)\n\s+expected: ([^\n]*)\n\s+actual  : ([^\n]*)"
)


SECONDS = 30


def _timeout(_signum: int, _frame: object) -> None:
    raise TimeoutError


def math(fn: str, a: list[str]) -> mpf:
    x, d1, d2 = mpf(a[0]), floor(mpf(a[1])), floor(mpf(a[2]))
    p, q = d1 / 2, d2 / 2
    z = d1 * x / (d1 * x + d2)
    if fn == "F.DIST.RT" or a[3] == "TRUE":
        # Above the mean, sum the complement's series: mpmath converges far faster there at huge shapes.
        if z > p / (p + q):
            upper = betainc(q, p, 0, d2 / (d1 * x + d2), regularized=True)
            return upper if fn == "F.DIST.RT" else 1 - upper
        lower = betainc(p, q, 0, z, regularized=True)
        return 1 - lower if fn == "F.DIST.RT" else lower
    return exp(
        (d1 / 2) * log(d1) + (d2 / 2) * log(d2) + (d1 / 2 - 1) * log(x) - ((d1 + d2) / 2) * log(d1 * x + d2)
        + loggamma((d1 + d2) / 2) - loggamma(d1 / 2) - loggamma(d2 / 2)
    )


def number(text: str) -> mpf | None:
    try:
        return mpf(text)
    except (ValueError, TypeError):
        return None


def within_15(actual: mpf, excel: mpf) -> bool:
    if excel == 0:
        return actual == 0
    return abs(actual - excel) <= mpf(10) ** (floor(log(abs(excel), 10)) - 14) / 2


def within_math(actual: mpf, truth: mpf) -> bool:
    return abs(actual - truth) <= max(EPS * abs(truth), FLOOR)


def stratum(case: str, row: int) -> str:
    if "sweep" not in case:
        return "all"
    return "1-80" if row <= 80 else "81-140" if row <= 140 else "141-180" if row <= 180 else "181-200"


def main() -> None:
    case = sys.argv[1]
    shown = subprocess.run(
        ["target/release/rustsheet-eval", "show", "--all", case], capture_output=True, text=True, check=True
    ).stdout
    signal.signal(signal.SIGALRM, _timeout)
    stats: dict[str, list[int]] = {}
    no_math = 0
    for _col, row, fn, args, expected, actual in CELL.findall(shown):
        s = stats.setdefault(stratum(case, int(row)), [0, 0, 0, 0, 0])
        s[0] += 1
        excel, ours = number(expected), number(actual)
        if (excel is None) != (ours is None):
            s[4] += 1
            continue
        if excel is None or ours is None:
            s[1] += expected == actual
            s[3] += expected != actual
            continue
        e15 = within_15(ours, excel)
        signal.alarm(SECONDS)
        try:
            em = within_math(ours, math(fn, [p.strip() for p in args.split(",")]))
        except (TimeoutError, ValueError):  # over the time limit, or mpmath cannot converge
            em, no_math = False, no_math + 1
        finally:
            signal.alarm(0)
        s[1] += e15
        s[2] += em
        s[3] += not (e15 or em)
    print("stratum  points  within-15-of-Excel  within-1e-9-of-math  outside-both  error-vs-number")
    for name, (n, e15, em, out, err) in stats.items():
        print(f"{name:8} {n:6} {e15:19} {em:20} {out:13} {err:16}")
    print(f"points with no math value (mpmath over {SECONDS} s or not converging): {no_math}")


if __name__ == "__main__":
    main()
