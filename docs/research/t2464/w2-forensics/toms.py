"""Call R's toms708 bratio (ext/bratio_sse) in batch."""
import subprocess
from pathlib import Path
EXE = Path(__file__).resolve().parent / "ext" / "bratio_sse"

def bratio(rows):
    """rows of (a, b, x, y) -> list of (w, w1, ierr)"""
    inp = "".join(f"{float(a).hex()} {float(b).hex()} {float(x).hex()} {float(y).hex()}\n" for a, b, x, y in rows)
    out = subprocess.run([str(EXE)], input=inp, capture_output=True, text=True, check=True).stdout.split("\n")
    res = []
    for line in out[:len(rows)]:
        w, w1, e = line.split()
        res.append((float.fromhex(w), float.fromhex(w1), int(e)))
    return res


def call(fn, rows, exe="toms_sse"):
    """rows of (a, b, x, y); fn in bratio, bratio1, brcomp, betaln, gamln, algdiv, bcorr, bpser, bfrac, dbeta_brc."""
    p = Path(__file__).resolve().parent / "ext" / exe
    inp = "".join(f"{fn} {float(a).hex()} {float(b).hex()} {float(x).hex()} {float(y).hex()}\n" for a, b, x, y in rows)
    out = subprocess.run([str(p)], input=inp, capture_output=True, text=True, check=True).stdout.split()
    return [float.fromhex(v) for v in out]


def rcall(fn, rows, exe="rdbeta_sse"):
    """R 3.6 nmath: fn in dbeta, lbeta, lgamma, stirlerr; rows of (a, b, x, y)."""
    p = Path(__file__).resolve().parent / "ext" / "r36" / exe
    inp = "".join(f"{fn} {float(a).hex()} {float(b).hex()} {float(x).hex()} {float(y).hex()}\n" for a, b, x, y in rows)
    out = subprocess.run([str(p)], input=inp, capture_output=True, text=True, check=True).stdout.split()
    return [float.fromhex(v) for v in out]
