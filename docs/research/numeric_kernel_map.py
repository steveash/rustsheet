"""Rank the shared numeric kernels by the near-miss cells they could fix (T-1522).

Usage (from the repository root, after `make eval`):

    python3 docs/research/numeric_kernel_map.py [eval/runs/latest/report.json]

Each failing cell of the eval report carries its formula and verdict. A cell is a numeric near miss
when its verdict is `numeric_ulp` (<= 4 ulp), `numeric_close` (beyond 4 ulp, within 1e-9 relative)
or `numeric_mismatch`. The cell is attributed to the kernel of every numeric function its formula
calls (wrappers such as DATE, IF and INDEX are ignored). "only" counts cells whose numeric
functions all use one kernel: fixing that kernel alone can fix them.
"""

from __future__ import annotations

import collections
import json
import re
import sys
from pathlib import Path

# Kernel -> functions that reach it. Mapped from the engine sources (crates/rustsheet-engine/src/
# functions); see docs/research/numeric-kernel-map.md for the call sites.
KERNELS: dict[str, list[str]] = {
    "normal-cdf/erfc": [
        "NORM.S.DIST",
        "NORM.DIST",
        "NORMSDIST",
        "NORMDIST",
        "LOGNORM.DIST",
        "LOGNORMDIST",
        "ERF",
        "ERF.PRECISE",
        "ERFC",
        "ERFC.PRECISE",
        "GAUSS",
        "Z.TEST",
        "ZTEST",
    ],
    "normal-pdf": ["PHI"],
    "normal-inverse": [
        "NORM.S.INV",
        "NORM.INV",
        "NORMSINV",
        "NORMINV",
        "LOGNORM.INV",
        "LOGINV",
        "CONFIDENCE.NORM",
        "CONFIDENCE",
    ],
    "gamma/lngamma": ["GAMMA", "GAMMALN", "GAMMALN.PRECISE", "FACT", "FACTDOUBLE", "BETA"],
    "incomplete-beta": [
        "BETA.DIST",
        "BETADIST",
        "F.DIST",
        "F.DIST.RT",
        "FDIST",
        "T.DIST",
        "T.DIST.RT",
        "T.DIST.2T",
        "TDIST",
        "T.TEST",
        "TTEST",
        "F.TEST",
        "FTEST",
        "BINOM.DIST",
        "BINOMDIST",
        "BINOM.DIST.RANGE",
        "NEGBINOM.DIST",
        "NEGBINOMDIST",
    ],
    "inverse-beta": [
        "BETA.INV",
        "BETAINV",
        "F.INV",
        "F.INV.RT",
        "FINV",
        "T.INV",
        "T.INV.2T",
        "TINV",
        "CONFIDENCE.T",
        "CRITBINOM",
        "BINOM.INV",
    ],
    "incomplete-gamma": [
        "GAMMA.DIST",
        "GAMMADIST",
        "CHISQ.DIST",
        "CHISQ.DIST.RT",
        "CHIDIST",
        "CHISQ.TEST",
        "CHITEST",
        "POISSON.DIST",
        "POISSON",
    ],
    "inverse-gamma": ["GAMMA.INV", "GAMMAINV", "CHISQ.INV", "CHISQ.INV.RT", "CHIINV"],
    "weibull/expon": ["WEIBULL.DIST", "WEIBULL", "EXPON.DIST", "EXPONDIST"],
    "hypergeometric": ["HYPGEOM.DIST", "HYPGEOMDIST", "MULTINOMIAL", "COMBIN", "COMBINA"],
    "annuity (1+r)^n": [
        "PMT",
        "PV",
        "FV",
        "IPMT",
        "PPMT",
        "NPER",
        "CUMIPMT",
        "CUMPRINC",
        "NPV",
        "MIRR",
        "EFFECT",
        "NOMINAL",
        "FVSCHEDULE",
        "RRI",
        "PDURATION",
    ],
    "solvers": ["RATE", "IRR", "XIRR", "YIELD", "ODDFYIELD", "ODDLYIELD"],
    "bond-pricing": [
        "PRICE",
        "ODDFPRICE",
        "ODDLPRICE",
        "DURATION",
        "MDURATION",
        "TBILLEQ",
        "TBILLYIELD",
        "TBILLPRICE",
        "DISC",
        "INTRATE",
        "RECEIVED",
        "PRICEDISC",
        "PRICEMAT",
        "YIELDDISC",
        "YIELDMAT",
        "ACCRINT",
        "ACCRINTM",
        "DB",
        "DDB",
        "VDB",
        "SLN",
        "SYD",
        "AMORLINC",
        "AMORDEGRC",
    ],
    "trig reduction": ["SIN", "COS", "TAN", "SEC", "CSC", "COT", "PI", "RADIANS", "DEGREES"],
    "inverse trig": ["ASIN", "ACOS", "ATAN", "ATAN2", "ACOT"],
    "hyperbolic": ["SINH", "COSH", "TANH", "COTH", "CSCH", "SECH", "ASINH", "ACOSH", "ATANH", "ACOTH"],
    "bessel": ["BESSELJ", "BESSELY", "BESSELI", "BESSELK"],
    "complex": [],  # every IM* function, matched by prefix below
    "pow/exp/ln/sqrt": ["POWER", "EXP", "LN", "LOG", "LOG10", "SQRT", "SQRTPI", "GEOMEAN", "^"],
}
FUNCTION_KERNEL = {f: k for k, fs in KERNELS.items() for f in fs}
NUMERIC = {"numeric_ulp", "numeric_close", "numeric_mismatch"}
CALL = re.compile(r"(?<![A-Za-z0-9_.])(?:_XLFN\.|_XLWS\.|_XLPM\.)*([A-Z][A-Z0-9.]*)\(")


def kernels_of(formula: str) -> set[str]:
    found: set[str] = set()
    for name in CALL.findall(formula.upper()):
        if name.startswith("IM") and name not in {"IMAGE"}:
            found.add("complex")
        elif name in FUNCTION_KERNEL:
            found.add(FUNCTION_KERNEL[name])
    if "^" in formula:
        found.add("pow/exp/ln/sqrt")
    return found or {"(no numeric kernel)"}


def main() -> None:
    path = Path(sys.argv[1] if len(sys.argv) > 1 else "eval/runs/latest/report.json")
    report = json.loads(path.read_text(encoding="utf-8"))
    touch: collections.Counter[str] = collections.Counter()
    only: collections.Counter[str] = collections.Counter()
    by_verdict: dict[str, collections.Counter[str]] = collections.defaultdict(collections.Counter)
    by_suite: dict[str, collections.Counter[str]] = collections.defaultdict(collections.Counter)
    total: collections.Counter[str] = collections.Counter()
    for case in report["cases"]:
        suite = case["id"].split("/", 1)[0]
        for cell in case["cells"]:
            verdict = cell["verdict"]
            if verdict not in NUMERIC or not cell.get("from_excel", True):
                continue
            total[verdict] += 1
            ks = kernels_of(cell.get("formula") or "")
            for k in ks:
                touch[k] += 1
                by_verdict[k][verdict] += 1
                by_suite[k][suite] += 1
            if len(ks) == 1:
                only[next(iter(ks))] += 1
    print(f"numeric near-miss cells: {sum(total.values())} ({dict(total)})")
    print("| kernel | cells | only this kernel | ulp | close | mismatch | suites |")
    print("|---|---|---|---|---|---|---|")
    for k, n in touch.most_common():
        v = by_verdict[k]
        suites = ", ".join(f"{s} {c}" for s, c in by_suite[k].most_common(3))
        print(
            f"| {k} | {n} | {only[k]} | {v['numeric_ulp']} | {v['numeric_close']} | "
            f"{v['numeric_mismatch']} | {suites} |"
        )


if __name__ == "__main__":
    main()
