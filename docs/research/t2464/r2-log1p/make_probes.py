"""Build the r2-log1p oracle probes (eval/suites/probes-numeric/cases/t2464-r2-log1p-*.case.json).

Each row: A = p (exact double literal), B = BINOM.DIST(0,n,A,FALSE) with n = 2^k (so n*l is exact),
C = BETA.DIST(A,1,n+1,FALSE) (density with b-1 = n, same ln(1-p)).  Usage: make_probes.py pilot|runs|grid|hold"""
import math
import sys

import r2lib as R


def sets(name):
    if name == "pilot":
        ps = [0.1, 0.2, 0.3] + R.run(0.3, 64) + R.run(0.375, 64, below=32) + R.run(0.0123456789, 64)
    elif name == "runs":
        ps = []
        for i in range(16):  # 16 irregular run starts over (0, 0.375)
            ps += R.run(0.0031415 + 0.0230137 * i, 256)
        ps += R.run(0.375, 400, below=200)
        for b in (0.25, 0.125, 0.0625, 2**-5):  # just below / above powers of two
            ps += R.run(b, 160, below=128)
        for s in (1e-3, 1.37e-5, 3.1e-8):
            ps += R.run(s, 64)
    elif name == "grid":
        ps = [0.000137 + 0.0000713113 * k for k in range(5250)] + [0.3751 + 0.000613 * k for k in range(200)]
    elif name == "hold":
        ps = [0.00041 + 0.0000698713 * k for k in range(4400)]
        ps += [10 ** (-12 + 10.4 * k / 1000) for k in range(1000)]  # 1e-12 .. 0.025
    else:
        raise SystemExit(name)
    return ps


def main(name):
    ps = sets(name)
    A, B, C = [], [], []
    for i, p in enumerate(ps):
        n = R.binom_n(p)
        A.append("=" + R.lit(p))
        B.append(f"=BINOM.DIST(0,{n},A{i + 1},FALSE)")
        C.append(f"=BETA.DIST(A{i + 1},1,{n + 1},FALSE)")
    path = R.write_case(name, f"T-2464 r2-log1p {name}: BINOM.DIST(0,2^k,p,FALSE) and BETA.DIST(p,1,2^k+1,FALSE) "
                        "isolate Excel's ln(1-p) (log1p) per p", {"A": A, "B": B, "C": C})
    print(path, len(ps))


if __name__ == "__main__":
    main(sys.argv[1])
