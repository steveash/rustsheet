"""Generate probes-numeric/t2733-price-r4-fresh (T-2733 R4.2 fresh sweep, written after the order was frozen).

Every row: PRICE (A), DURATION (B), MDURATION (C) and the coupon parts COUPDAYBS (D), COUPDAYS (E),
COUPDAYSNC (F), COUPNUM (G) of one bond. Inputs are log-uniform over the domain and stratified 6 x 50 by
coupon-rate band (1E-4 to 1, 1 to 1E4) and yield band (1E-4 to 1E-1, 1E-1 to 10, 10 to 1E4). Fixed seed.
Usage: python3 gen_fresh.py OUT.case.json [n1]

With `n1` it writes probes-numeric/t2733-price-n1-fresh instead: 200 bonds with one coupon left (term up to one
coupon period), stratified 4 x 50 by rate band (1E-4 to 1, 1 to 1E4) and yield band (1E-4 to 1, 1 to 1E4), drawn
after the single-coupon accrued order was fixed in review (seed T-2733/PRICE/n1-fresh-after-review).
"""

import hashlib
import json
import math
import random
import sys

SEED = "T-2733/PRICE/r4-fresh-after-freeze"


def lu(rng: random.Random, lo: float, hi: float) -> float:
    return float(f"{math.exp(rng.uniform(math.log(lo), math.log(hi))):.6g}")


def main() -> None:
    n1 = len(sys.argv) > 2 and sys.argv[2] == "n1"
    seed = "T-2733/PRICE/n1-fresh-after-review" if n1 else SEED
    rng = random.Random(int(hashlib.sha256(seed.encode()).hexdigest(), 16))
    cells: dict[str, str] = {}
    row = 0
    yld_bands = ((1e-4, 1.0), (1.0, 1e4)) if n1 else ((1e-4, 1e-1), (1e-1, 10.0), (10.0, 1e4))
    for rate_band in ((1e-4, 1.0), (1.0, 1e4)):
        for yld_band in yld_bands:
            for _ in range(50):
                row += 1
                settle = rng.randint(36526, 47482)  # 2000-01-01 .. 2029-12-31
                if n1:
                    freq = rng.choice((1, 2, 4))
                    mat = settle + rng.randint(1, 360 // freq)
                else:
                    years = lu(rng, 0.1, 30.0)
                    mat = settle + max(1, int(years * 365.25))
                rate = lu(rng, *rate_band)
                yld = lu(rng, *yld_band)
                red = lu(rng, 1.0, 1e4)
                if not n1:
                    freq = rng.choice((1, 2, 4))
                basis = rng.randint(0, 4)
                d = f"{settle},{mat}"
                cells[f"A{row}"] = f"=PRICE({d},{rate:.6g},{yld:.6g},{red:.6g},{freq},{basis})"
                cells[f"B{row}"] = f"=DURATION({d},{rate:.6g},{yld:.6g},{freq},{basis})"
                cells[f"C{row}"] = f"=MDURATION({d},{rate:.6g},{yld:.6g},{freq},{basis})"
                for col, fn in zip("DEFG", ("COUPDAYBS", "COUPDAYS", "COUPDAYSNC", "COUPNUM")):
                    cells[f"{col}{row}"] = f"={fn}({d},{freq},{basis})"
    description = (
        "T-2733 review round 2 (decision 0188 R4.2): fresh sweep for PRICE's single-coupon accrued order "
        "(`coupon * (A/E)`, restored in review after t2733-price-r4-fresh), generated after that commit and never "
        "used to choose or tune it. 200 bonds with one coupon left, rows 1-200: A PRICE, B DURATION, C MDURATION, "
        "D COUPDAYBS, E COUPDAYS, F COUPDAYSNC, G COUPNUM. Settlement uniform over 2000-2029, maturity 1 to 360/f "
        "days later, coupon rate, yield and redemption log-uniform, stratified 4 x 50 by rate band (1E-4 to 1, "
        "1 to 1E4) and yield band (1E-4 to 1, 1 to 1E4); frequency 1/2/4 and basis 0-4 uniform. Seed " + seed
        + ". Generator docs/research/t2733/gen_fresh.py n1."
    ) if n1 else None
    case = {
        "format": "rustsheet-case/v1",
        "description": description or (
            "T-2733 review (decision 0188 R4.2, reviewer check 7): fresh sweep for the PRICE, DURATION and "
            "MDURATION coupon-sum order, generated after the order was frozen and never used to choose or tune it. "
            "300 bonds, rows 1-300: A PRICE, B DURATION, C MDURATION, D COUPDAYBS, E COUPDAYS, F COUPDAYSNC, "
            "G COUPNUM. Settlement uniform over 2000-2029, term log-uniform 0.1 to 30 years, coupon rate, yield "
            "and redemption log-uniform, stratified 6 x 50 by rate band (1E-4 to 1, 1 to 1E4) and yield band "
            "(1E-4 to 1E-1, 1E-1 to 10, 10 to 1E4); frequency 1/2/4 and basis 0-4 uniform. Seed " + SEED
            + ". Generator docs/research/t2733/gen_fresh.py."
        ),
        "tags": ["M9", "financial", "probe", "gate-a-fresh"],
        "expected": {},
        "sheets": [{"name": "Calc", "cells": cells}],
    }
    with open(sys.argv[1], "w") as f:
        json.dump(case, f, indent=2, ensure_ascii=False)
        f.write("\n")


if __name__ == "__main__":
    main()
