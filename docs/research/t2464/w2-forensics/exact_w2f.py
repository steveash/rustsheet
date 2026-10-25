"""Exact I_x(a,b) and density (mpmath, 40 digits) for cells_w2f.tsv -> exact_w2f.tsv.
Columns: case, cell, x, a, b, cum, excel, exact (30 digits), rel (Excel's relative error in units of 2^-53), err_ulp."""
import csv, math, sys
from multiprocessing import Pool
import mpmath

def work(r):
    mpmath.mp.dps = 40
    x, a, b = float(r[0]), float(r[1]), float(r[2])
    cum = r[3] == "True"
    X, A, B = mpmath.mpf(x), mpmath.mpf(a), mpmath.mpf(b)
    if cum:
        v = mpmath.betainc(A, B, 0, X, regularized=True)
    else:
        v = mpmath.exp((A - 1) * mpmath.log(X) + (B - 1) * mpmath.log1p(-X) - mpmath.log(mpmath.beta(A, B)))
    return v

if __name__ == "__main__":
    here = __file__.rsplit("/", 1)[0]
    rows = []
    for line in open(f"{here}/cells_w2f.tsv").read().splitlines()[1:]:
        fn, case, cell, args, excel = line.split("\t")
        x, a, b, cum = args.split("|")
        rows.append((case, cell, x, a, b, cum, excel))
    with Pool(4) as p:
        vals = p.map(work, [r[2:6] for r in rows], chunksize=200)
    mpmath.mp.dps = 40
    with open(f"{here}/exact_w2f.tsv", "w") as f:
        f.write("case\tcell\tx\ta\tb\tcum\texcel\texact\trel\terr_ulp\n")
        for r, v in zip(rows, vals):
            e = float(r[6])
            rel = float((mpmath.mpf(e) - v) / v) / 2.0**-53
            ev = float(v)
            ulp = (e - ev) / math.ulp(ev)
            f.write("\t".join([r[0].split("/")[-1].replace(".case.json", ""), r[1], r[2], r[3], r[4], "1" if r[5] == "True" else "0",
                               r[6], mpmath.nstr(v, 30), f"{rel:.4f}", f"{ulp:.2f}"]) + "\n")
    print(len(rows))
