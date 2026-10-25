"""Probe t2464-r2-log1p-hix: does the density use alnrel(-y) for ln x when y = 1-x <= 0.375 (TOMS brcomp)?
A = x in (0.625, 1), B = BETA.DIST(A,1025,1,FALSE) (= 1025*x^1024), C = BETA.DIST(A,1025,1.5,FALSE).
Usage: make_probe_hi.py"""
import r2lib as R


def sets():
    xs = [0.6250731 + 0.0000713113 * k for k in range(5200)]
    return [x for x in xs if x < 1] + R.run(0.625, 64, below=32)


if __name__ == "__main__":
    xs = sets()
    A = ["=" + R.lit(x) for x in xs]
    B = [f"=BETA.DIST(A{i + 1},1025,1,FALSE)" for i in range(len(xs))]
    C = [f"=BETA.DIST(A{i + 1},1025,1.5,FALSE)" for i in range(len(xs))]
    print(R.write_case("hix", "T-2464 r2-log1p hix: BETA.DIST(x,1025,1|1.5,FALSE) for x in (0.625,1): ln x as LN(x) "
                       "or alnrel(-(1-x))", {"A": A, "B": B, "C": C}), len(xs))
