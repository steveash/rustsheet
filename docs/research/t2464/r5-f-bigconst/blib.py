"""T-2586 r5-f-bigconst: shared helpers (everything Excel-specific is imported, nothing copied).

Imports r5-f-combine `clib` (Excel's admissible extended exponent per cell by interval inversion of the final step,
pieces P / H, the stored-sum chain, deepest interval overlap), and through it r4-f-small-df `slib` / `sdmodel`
(Excel LN, log1p, EXP V6, side and r as Excel forms them, M-sd's constant delta) and r3-fdensity's probe helpers.
"""

from __future__ import annotations

import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
T = HERE.parent
for p in (T / "r5-f-combine",):
    if str(p) not in sys.path:
        sys.path.insert(0, str(p))

import clib as C  # noqa: E402

SL, SD, F, R3 = C.SL, C.SD, C.F, C.R3
LDB, st, mpf, mp = C.LDB, C.st, C.mpf, C.mpmath
U = C.U
MP = F.MP
R3P = SL.B._load("r5b_r3mkprobe", T / "r3-fdensity" / "mkprobe.py")
_MK = SL.B._load("r5b_mkprobe", HERE / "mkprobe.py")


def load(name: str):
    """'ba' / 'bhold' = this directory's probes; anything else goes to clib.load."""
    if name in ("ba", "bhold"):
        return F.with_exact(_MK.rows(name[1:]))
    return C.load(name)
