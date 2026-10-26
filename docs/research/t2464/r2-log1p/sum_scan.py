"""Ways to form the tail s ~ atanh(r) - r from terms T_k = c_k * r*t2^k (c_k = RN(1/(2k+1))), then
l = 2*(r + s), all x87 stored; scored on -r > 0.1 rows (even/odd).  Usage: sum_scan.py"""
import gen_scan as G

M, A_, D_ = G.M, G.A_, G.D_
N = 14
CS = [1.0 / (2 * k + 1) for k in range(N + 1)]


def terms(r, t2, first_div=False):
    p, out = r, []
    for k in range(1, N + 1):
        p = M(p, t2)
        out.append(D_(p, 3.0) if (k == 1 and first_div) else M(p, CS[k]))
    return out


def fwd(ts):
    s = ts[0]
    for t in ts[1:]:
        s = A_(s, t)
    return s


def rev(ts):
    return fwd(ts[::-1])


def make(kind):
    def f(r):
        t2 = M(r, r)
        if kind == "fwd":
            s = fwd(terms(r, t2))
        elif kind == "fwd, T1 by /3":
            s = fwd(terms(r, t2, True))
        elif kind == "T1 + rev(rest)":
            ts = terms(r, t2)
            s = A_(ts[0], rev(ts[1:]))
        elif kind == "T1 + fwd(rest)":
            ts = terms(r, t2)
            s = A_(ts[0], fwd(ts[1:]))
        elif kind == "(T1+T2) + fwd(rest)":
            ts = terms(r, t2)
            s = A_(A_(ts[0], ts[1]), fwd(ts[2:]))
        elif kind == "pairwise":
            ts = terms(r, t2)
            while len(ts) > 1:
                ts = [A_(ts[i], ts[i + 1]) if i + 1 < len(ts) else ts[i] for i in range(0, len(ts), 2)]
            s = ts[0]
        elif kind == "Horner tail p1*H":
            h = CS[N]
            for k in range(N - 1, 0, -1):
                h = A_(M(h, t2), CS[k])
            s = M(M(r, t2), h)
        elif kind == "Horner r*(t2*H)":
            h = CS[N]
            for k in range(N - 1, 0, -1):
                h = A_(M(h, t2), CS[k])
            s = M(r, M(t2, h))
        else:
            raise KeyError(kind)
        return 2 * A_(r, s)
    return f


KINDS = ["fwd", "fwd, T1 by /3", "T1 + rev(rest)", "T1 + fwd(rest)", "(T1+T2) + fwd(rest)", "pairwise",
         "Horner tail p1*H", "Horner r*(t2*H)"]

if __name__ == "__main__":
    rows = [x for x in G.load_rows() if -x[1] > 0.1]
    for kd in KINDS:
        per, half, miss = G.score(make(kd), rows)
        print("%-22s even %d/%d odd %d/%d" % (kd, *half[0], *half[1]), flush=True)
