# coherence objective: amplitude-weighted mean cos(2pi dphi) over runs x lines; demod vectors from a reference law
import sys, pickle, os

exec(open("jointA.py").read())


def build(xref, runs, lines=LINES):
    items = []
    for a, yv, g, rd in runs:
        n = len(yv)
        mid = n // 2
        for ch, j, typ in lines:
            th0, row, e = theta_and_row(xref, ch, j, typ, a, Fr(yv[mid]))
            th1, _, _ = theta_and_row(xref, ch, j, typ, a, Fr(yv[mid + 1]))
            thm, _, _ = theta_and_row(xref, ch, j, typ, a, Fr(yv[mid - 1]))
            f = float(((th1 - thm) / 2) % 1)
            fdd = float(th1 - 2 * th0 + thm)
            t0 = float(th0 - math.floor(th0))
            k = np.arange(n) - mid
            Ez = np.exp(-2j * np.pi * (t0 + f * k + 0.5 * fdd * k * k)) * np.hanning(n)
            items.append((Ez, np.dot(rd - rd.mean(), Ez)))
    return items


def coherence(x, runs, items, nl):
    tot = 0.0
    wsum = 0.0
    it = 0
    for a, yv, g, rd in runs:
        q = model(x, yv)
        rs = rd + (q - g) / ULP
        rs = rs - rs.mean()
        for l in range(nl):
            Ez, Ad = items[it]
            it += 1
            As = np.dot(rs, Ez)
            w = abs(Ad)
            if w < 50 or abs(As) < 1e-9:
                continue
            tot += w * np.real(Ad * np.conj(As)) / (abs(Ad) * abs(As))
            wsum += w
    return tot / wsum
