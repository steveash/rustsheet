"""Second-round variants of the power-sum form l = 2*(r + sum_k c_k * p_k), p_k = r*t2^k, c_k = RN(1/(2k+1)):
term order, how p_k is formed, the factor 2 placement, and N.  Usage: tail_scan2.py"""
import gen_scan as G

M, A_, D_ = G.M, G.A_, G.D_


def make(order="fwd", pform="chain", outer="2(r+s)", N=14, scale=1.0, first=1):
    cs = [scale / (2 * k + 1) for k in range(N + 1)]

    def f(r):
        t2 = M(r, r)
        terms = []
        p, q = r, 1.0
        for k in range(first, N + 1):
            if pform == "chain":
                p = M(p, t2)
                pk = p
            elif pform == "q":  # q = t2^k, p_k = r*q
                q = M(q, t2) if k > 1 else t2
                pk = M(r, q)
            terms.append(M(pk, cs[k]))
        if order == "rev":
            terms = terms[::-1]
        s = terms[0]
        for tm in terms[1:]:
            s = A_(s, tm)
        if outer == "2(r+s)":
            return 2 * A_(r, s)
        if outer == "2r+s":  # scale=2 coefficients
            return A_(2 * r, s)
        if outer == "r2+s, 2r*c":
            return A_(2 * r, s)
        raise KeyError(outer)
    return f


VARS = {
    "fwd chain 2(r+s)": make(),
    "rev chain 2(r+s)": make(order="rev"),
    "fwd q 2(r+s)": make(pform="q"),
    "fwd chain 2r+s (c=2/(2k+1))": make(outer="2r+s", scale=2.0),
    "fwd chain N=11": make(N=11),
    "fwd chain N=12": make(N=12),
}

if __name__ == "__main__":
    rows = G.load_rows()
    print("%-30s" % "variant" + "".join("%12s" % f"<{e}" for e in G.EDGES[1:]) + "      even        odd")
    for nm, f in VARS.items():
        per, half, miss = G.score(f, rows)
        print("%-30s" % nm + "".join("%12s" % f"{a}/{b}" for a, b in per) + "".join("%11s" % f"{a}/{b}" for a, b in half))
        if nm.startswith("fwd chain 2(r+s)"):
            print("   misses:", [round(p, 6) for p, _ in miss])
