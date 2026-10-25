"""More tail forms: power sums without the r factor, multiplied in afterwards; scored on -r > 0.1 rows.
Usage: sum_scan2.py"""
import gen_scan as G
from sum_scan import CS, N, fwd

M, A_ = G.M, G.A_


def qs(t2, start):
    q, out = start, []
    for k in range(1, N + 1):
        out.append(q)
        q = M(q, t2)
    return out


def make(kind):
    def f(r):
        t2 = M(r, r)
        if kind == "r * sum c_k t2^k":
            s = M(r, fwd([M(q, CS[k + 1]) for k, q in enumerate(qs(t2, t2))]))
        elif kind == "(r*t2) * sum c_k t2^(k-1)":
            s = M(M(r, t2), fwd([M(q, CS[k + 1]) for k, q in enumerate(qs(t2, 1.0))]))
        elif kind == "r3 = (r*r)*r chain":
            p = M(t2, r)
            ts = []
            for k in range(1, N + 1):
                ts.append(M(p, CS[k]))
                p = M(p, t2)
            s = fwd(ts)
        elif kind == "2r + sum (2c_k) p_k":
            p, ts = r, []
            for k in range(1, N + 1):
                p = M(p, t2)
                ts.append(M(p, 2 * CS[k]))
            return A_(2 * r, fwd(ts))
        elif kind == "2r + sum c_k (2r t2^k)":
            p, ts = 2 * r, []
            for k in range(1, N + 1):
                p = M(p, t2)
                ts.append(M(p, CS[k]))
            return A_(2 * r, fwd(ts))
        else:
            raise KeyError(kind)
        return 2 * A_(r, s)
    return f


KINDS = ["r * sum c_k t2^k", "(r*t2) * sum c_k t2^(k-1)", "r3 = (r*r)*r chain", "2r + sum (2c_k) p_k",
         "2r + sum c_k (2r t2^k)"]
if __name__ == "__main__":
    rows = [x for x in G.load_rows() if -x[1] > 0.1]
    for kd in KINDS:
        per, half, miss = G.score(make(kd), rows)
        print("%-28s even %d/%d odd %d/%d" % (kd, *half[0], *half[1]), flush=True)
