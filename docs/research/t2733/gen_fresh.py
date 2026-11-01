"""Generate probes-numeric/t2733-price-r4-fresh (T-2733 R4.2 fresh sweep, written after the order was frozen).

Every row: PRICE (A), DURATION (B), MDURATION (C) and the coupon parts COUPDAYBS (D), COUPDAYS (E),
COUPDAYSNC (F), COUPNUM (G) of one bond. Inputs are log-uniform over the domain and stratified 6 x 50 by
coupon-rate band (1E-4 to 1, 1 to 1E4) and yield band (1E-4 to 1E-1, 1E-1 to 10, 10 to 1E4). Fixed seed.
Usage: python3 gen_fresh.py OUT.case.json
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
    rng = random.Random(int(hashlib.sha256(SEED.encode()).hexdigest(), 16))
    cells: dict[str, str] = {}
    row = 0
    for rate_band in ((1e-4, 1.0), (1.0, 1e4)):
        for yld_band in ((1e-4, 1e-1), (1e-1, 10.0), (10.0, 1e4)):
            for _ in range(50):
                row += 1
                settle = rng.randint(36526, 47482)  # 2000-01-01 .. 2029-12-31
                years = lu(rng, 0.1, 30.0)
                mat = settle + max(1, int(years * 365.25))
                rate = lu(rng, *rate_band)
                yld = lu(rng, *yld_band)
                red = lu(rng, 1.0, 1e4)
                freq = rng.choice((1, 2, 4))
                basis = rng.randint(0, 4)
                d = f"{settle},{mat}"
                cells[f"A{row}"] = f"=PRICE({d},{rate:.6g},{yld:.6g},{red:.6g},{freq},{basis})"
                cells[f"B{row}"] = f"=DURATION({d},{rate:.6g},{yld:.6g},{freq},{basis})"
                cells[f"C{row}"] = f"=MDURATION({d},{rate:.6g},{yld:.6g},{freq},{basis})"
                for col, fn in zip("DEFG", ("COUPDAYBS", "COUPDAYS", "COUPDAYSNC", "COUPNUM")):
                    cells[f"{col}{row}"] = f"={fn}({d},{freq},{basis})"
    case = {
        "format": "rustsheet-case/v1",
        "description": (
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
