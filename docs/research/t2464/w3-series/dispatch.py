"""TOMS 708 bratio dispatch (R toms708.c), shared by bratio_map.py and bup.py."""


EPS = 1e-15


def branch(a, b, x, y):
    """(branch name, a0, b0, x0, y0, swapped) following bratio for one cell."""
    if min(a, b) <= 1:
        sw = x > 0.5
        a0, b0, x0, y0 = (b, a, y, x) if sw else (a, b, x, y)
        if b0 < min(EPS, EPS * a0):
            return "fpser", a0, b0, x0, y0, sw
        if a0 < min(EPS, EPS * b0) and b0 * x0 <= 1:
            return "apser", a0, b0, x0, y0, sw
        if max(a0, b0) > 1:
            if b0 <= 1:
                return "w_bpser", a0, b0, x0, y0, sw
            if x0 >= 0.29:
                return "w1_bpser", a0, b0, x0, y0, sw
            if x0 < 0.1 and (x0 * b0) ** a0 <= 0.7:
                return "w_bpser", a0, b0, x0, y0, sw
            return ("bgrat" if b0 > 15 else "bup+bgrat"), a0, b0, x0, y0, sw
        if a0 >= min(0.2, b0) or x0**a0 <= 0.9:
            return "w_bpser", a0, b0, x0, y0, sw
        if x0 >= 0.3:
            return "w1_bpser", a0, b0, x0, y0, sw
        return "bup+bgrat", a0, b0, x0, y0, sw
    lam = (a + b) * y - b if a > b else a - (a + b) * x
    sw = lam < 0
    a0, b0, x0, y0 = (b, a, y, x) if sw else (a, b, x, y)
    lam = abs(lam)
    if b0 < 40:
        return ("w_bpser" if b0 * x0 <= 0.7 else "bup+bfrac"), a0, b0, x0, y0, sw
    if a0 > b0:
        if b0 <= 100 or lam > b0 * 0.03:
            return "bfrac", a0, b0, x0, y0, sw
    elif a0 <= 100 or lam > a0 * 0.03:
        return "bfrac", a0, b0, x0, y0, sw
    return "basym", a0, b0, x0, y0, sw


