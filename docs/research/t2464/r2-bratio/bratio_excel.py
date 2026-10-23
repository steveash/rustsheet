"""T-2464 r2-bratio: one end-to-end model of Excel's incomplete beta I_x(a, b) as a TOMS 708 `bratio` port.

This module is meant to be read top to bottom by whoever ports the kernel to the engine. It is a complete port of
TOMS 708 bratio (Didonato & Morris 1992; structure from R's src/nmath/toms708.c and the original Fortran), with every
place where round 1 found Excel deviating from plain TOMS behind a switch in `Cfg`:

  elem      exp/log inside TOMS: "libm" (glibc, as R) or "excel" (Excel's EXP model V6 and LN = FLDLN2/FYL2X)
  gamln     TOMS gamln/gsumln, or Excel's GAMMALN (engine model, via the harness) wherever betaln calls gamln
  x29       bratio's min(a,b) <= 1 switch to w1 = bpser(b0,a0,y0): 0.29 (R, PR#13786) or 0.3 (TOMS 1992)
  closed    w5's closed forms after the swap (b0 == 1, and a0 == 1 with y0 <= 0.9)
  bpser     "toms" (TOMS bpser) or "w3" (forward x87 series from 1/a, I = EXP(z)*s, z = fl(a LN x) - L)
  w3_z      the w3 EXP argument: "z53" (stored double) or "z64" (extended, EXP of a 64-bit argument)
  brcmp1    bup's prefactor: "toms" or "pdf" (Excel's density times x times y, w4)
  brcomp    bfrac's prefactor: "toms" or "pdf"
  bup_arith "double" (TOMS in SSE doubles) or "w4" (ratios stored, ratio*y in an x87 register, d and w stored)
  bgrat     "orig" (1992 Fortran: r = b(1+gam1 b) z^b x^a x^(bm1/2), u = r exp(-u)) or "r" (R's log_r form;
            "r" also selects R's algdiv t = 1/(b*b), so R_LIKE reproduces R's binary)
  bup_break TOMS's relative stop d <= eps*w in bup (True) or summing every term (False, as w4 did)
  w3_lg, w3_order, w3_big   the ln B inside w3's bpser (see lnb_w3); algdiv, pdf_lnb   negative-result switches

The defaults (BEST) are the round-2 integrated model; RESULTS.md has the attribution ladder, one switch at a time.
Pieces that are still stand-ins, not Excel's: log1p(-x) in the density (correctly rounded; Excel's is unidentified),
the density for min(a, b) > 1 (correctly rounded), psi in apser (mpmath digamma; apser needs a0 < 1e-15), and
w3's ln B for max(a, b) > 8 (accurate value, a proxy for algdiv).

`kernel(a, b, x, y, cfg)` returns (w, w1, branch): w = I_x(a,b), w1 = 1 - I_x(a,b), the branch that fired (with
"/sw" when bratio swapped). The caller forms x and y (BETA.DIST: y = fl(1-x); see w5 for F, T, BINOM).

Excel's elementary functions:
  EXP(v)  Excel's EXP (model V6), for a double v; EXPX(v) the same for an extended (mpmath) argument
  LN(v)   Excel's LN, FLDLN2/FYL2X, rounded to double
  GAMMALN Excel's GAMMALN: the oracle's own value where probe t2464-r2-bratio-gammaln measured it, else the engine
          model from the harness (cached in gammaln_cache.pkl; a missing value is recorded, a placeholder (lgamma)
          is returned, and `run()` fetches the missing ones and re-evaluates).
Run with /home/user/rustsheet/bindings/python/.venv/bin/python (x86-64: numpy longdouble is the x87 format).
"""

from __future__ import annotations

import importlib.util
import math
import os
import pickle
import sys
from dataclasses import dataclass, replace
from pathlib import Path

import mpmath
import numpy as np

HERE = Path(__file__).resolve().parent
T2464 = HERE.parent
sys.path.insert(0, str(T2464))
import excelmath as em  # noqa: E402


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# w1-density's mpmath models of Excel's EXP (V6, extended input allowed) and LN; they match the engine harness on
# 5,000/5,000 and 4,000/4,000 inputs (w1 ctl_elem.py, re-checked by control_elem in score_lib.py).
W1 = _load("w1lib_r2", T2464 / "w1-density" / "lib.py")
mpf = mpmath.mpf
LD = np.longdouble
assert np.finfo(LD).nmant == 63, "needs the x87 80-bit long double"


# ---------------------------------------------------------------------------------------------- arithmetic
def sx(op: str, a, b) -> float:
    """A stored x87 operation, fl53(fl64(a op b)) (playbook section 1, item 1)."""
    p, q = LD(a), LD(b)
    if op == "+":
        return float(p + q)
    if op == "-":
        return float(p - q)
    if op == "*":
        return float(p * q)
    return float(p / q)


# ---------------------------------------------------------------------------------------------- Excel functions
_EXPM: dict = {}
_LNM: dict = {}


def EXP(v: float) -> float:
    r = _EXPM.get(v)
    if r is None:
        r = _EXPM[v] = W1.exp_v6(mpf(v)) if math.isfinite(v) else (0.0 if v < 0 else math.inf)
    return r


def EXPX(v) -> float:
    """Excel's EXP of an extended (64-bit) argument, as when the argument stays in an x87 register."""
    return W1.exp_v6(v)


def LN(v: float) -> float:
    r = _LNM.get(v)
    if r is None:
        r = _LNM[v] = W1.ln_x87(v)
    return r


_GLF = HERE / "gammaln_cache.pkl"
_GL: dict = pickle.loads(_GLF.read_bytes()) if _GLF.exists() else {}
_GL_MISSING: set = set()
# Excel's own GAMMALN at the kernel's arguments (probe t2464-r2-bratio-gammaln, check_gl.py): the engine model misses
# 512 of 5,094 (mostly v < 1 and the 1-ulp-step sweeps near 1.5, 2.5, 3), so measured values override it.
USE_PROBED_GL = os.environ.get("R2_PROBED_GL", "1") == "1"  # R2_PROBED_GL=0: engine model only (ablation)


def probed_gammaln() -> dict:
    """Excel's GAMMALN values read from the probe output (argument list t2464-r2-bratio-gammaln.args.json)."""
    import json

    args_f, out_f = HERE / "t2464-r2-bratio-gammaln.args.json", HERE / "t2464-r2-bratio-gammaln.out.json"
    if not (args_f.exists() and out_f.exists()):
        return {}
    cells = json.loads(out_f.read_text())["sheets"]["Calc"]
    got = {}
    for i, v in enumerate(json.loads(args_f.read_text())):
        c = cells.get(f"A{i + 1}")
        if c and c.get("t") == "n":
            got[v] = c["v"]
    return got


if USE_PROBED_GL:
    _GL.update(probed_gammaln())


def GAMMALN(v: float) -> float:
    r = _GL.get(v)
    if r is None:
        _GL_MISSING.add(v)
        return math.lgamma(v)
    return r


def fetch_missing() -> int:
    """Fetch the GAMMALN values recorded as missing from the engine harness; returns how many were fetched."""
    need = sorted(_GL_MISSING)
    _GL_MISSING.clear()
    for i in range(0, len(need), 20000):
        chunk = need[i : i + 20000]
        for v, r in zip(chunk, em.excel("GAMMALN", chunk)):
            _GL[v] = r
    if need:
        model = pickle.loads(_GLF.read_bytes()) if _GLF.exists() else {}
        model.update({v: _GL[v] for v in need})
        _GLF.write_bytes(pickle.dumps(model))
    return len(need)


def log1p_cr(v: float) -> float:
    """Correctly rounded log1p: the stand-in for Excel's log1p, which is faithful but not correctly rounded and is
    still unidentified (w1 finding 4, w5's BINOM readout at p = 0.3)."""
    return float(mpmath.log1p(mpf(v)))


# ---------------------------------------------------------------------------------------------- configuration
@dataclass(frozen=True)
class Cfg:
    elem: str = "excel"
    gamln: str = "excel"
    x29: float = 0.3  # Excel uses the 1992 value (probe t2464-r2-bratio-x29, x29_test.py: 284 vs 85 of 432)
    closed: bool = True
    bpser: str = "w3"
    w3_z: str = "z64"
    brcmp1: str = "pdf"
    brcomp: str = "pdf"
    bup_arith: str = "w4"
    bgrat: str = "orig"
    bup_break: bool = True
    w3_lg: str = "toms"  # w3's ln Gamma(v < 1): "acc" (accurate, 64-bit), "gl" (Excel GAMMALN), "toms" (gamln1(v) - LN v)
    w3_order: str = "arg"  # w3's ln B sum order: "arg" (lg(a) + (lg(b) - lg(a+b))) or "min" (smaller shape first)
    w3_big: str = "acc"  # w3's ln B for max(a, b) > 8: "acc" (accurate, w3's proxy) or "toms" (betaln, Excel pieces)
    algdiv: str = "toms"  # "toms", "gl" (ln(Gamma(b)/Gamma(a+b)) = fl(GAMMALN(b) - GAMMALN(a+b))), "gl_bgrat" (gl in bgrat)
    pdf_lnb: str = "w1"  # lnB inside the density used as a prefactor: "w1" (M1's betaln) or "gl" (always GL form)


# Plain TOMS 708 as R builds it: the positive control.
R_LIKE = Cfg(
    elem="libm",
    gamln="toms",
    x29=0.29,
    closed=False,
    bpser="toms",
    brcmp1="toms",
    brcomp="toms",
    bup_arith="double",
    bgrat="r",
)
# Plain TOMS 708 (1992 Fortran thresholds) with Excel's EXP/LN/GAMMALN substituted, nothing else.
TOMS_XL = replace(R_LIKE, elem="excel", gamln="excel", x29=0.3, bgrat="orig")
# The round-2 integrated model (best per-branch choices; see RESULTS.md).
BEST = Cfg()


class Kernel:
    """TOMS 708 bratio and its subroutines. Method names are TOMS's; comments name the Excel deviations."""

    EPS = 1e-15  # bratio: eps = max(2^-52, 1e-15)

    def __init__(self, cfg: Cfg):
        self.c = cfg
        if cfg.elem == "excel":
            self.exp, self.log = EXP, LN
        else:
            self.exp, self.log = math.exp, math.log

    # ------------------------------------------------------------------ elementary TOMS pieces
    def alnrel(self, a: float) -> float:
        """ln(1 + a). Above |a| = 0.375 TOMS calls log(1 + a); below it a (3,3) rational in t = a/(a+2)."""
        if abs(a) > 0.375:
            return self.log(1.0 + a)
        t = a / (a + 2.0)
        t2 = t * t
        num = ((-0.0178874546012214 * t2 + 0.405303492862024) * t2 + -1.29418923021993) * t2 + 1.0
        den = ((-0.0845104217945565 * t2 + 0.747811014037616) * t2 + -1.62752256355323) * t2 + 1.0
        return t * 2.0 * (num / den)

    def rlog1(self, x: float) -> float:
        """x - ln(1 + x)."""
        if x < -0.39 or x > 0.57:
            return x - self.log(x + 0.5 + 0.5)
        if x < -0.18:
            h = (x + 0.3) / 0.7
            w1 = 0.0566749439387324 - h * 0.3
        elif x > 0.18:
            h = x * 0.75 - 0.25
            w1 = 0.0456512608815524 + h / 3.0
        else:
            h, w1 = x, 0.0
        r = h / (h + 2.0)
        t = r * r
        w = ((0.00620886815375787 * t + -0.224696413112536) * t + 0.333333333333333) / (
            (0.354508718369557 * t + -1.27408923933623) * t + 1.0
        )
        return t * 2.0 * (1.0 / (1.0 - r) - r * w) + w1

    def rexpm1(self, x: float) -> float:
        if abs(x) <= 0.15:
            num = (0.0238082361044469 * x + 9.14041914819518e-10) * x + 1.0
            den = (
                ((5.95130811860248e-4 * x + -0.0119041179760821) * x + 0.107141568980644) * x
                + -0.499999999085958
            ) * x + 1.0
            return x * (num / den)
        w = self.exp(x)
        return w * (0.5 - 1.0 / w + 0.5) if x > 0 else w - 0.5 - 0.5

    def esum(self, mu: int, x: float) -> float:
        """exp(mu + x)."""
        if x > 0:
            if mu > 0 or mu + x < 0:
                return self.exp(float(mu)) * self.exp(x)
        else:
            if mu < 0 or mu + x > 0:
                return self.exp(float(mu)) * self.exp(x)
        return self.exp(mu + x)

    @staticmethod
    def exparg(l: int) -> float:
        m = 1024 if l == 0 else -1021 - 1
        return m * 0.69314718055995 * 0.99999

    def gam1(self, a: float) -> float:
        """1/Gamma(a+1) - 1 for -0.5 <= a <= 1.5."""
        d = a - 0.5
        t = d - 0.5 if d > 0 else a
        if t < 0:
            r = (
                -1.32674909766242e-4,
                2.66505979058923e-4,
                0.00223047661158249,
                -0.0118290993445146,
                9.30357293360349e-4,
                0.118378989872749,
                -0.244757765222226,
                -0.771330383816272,
                -0.422784335098468,
            )
            top = r[0]
            for k in r[1:]:
                top = top * t + k
            w = top / ((0.0559398236957378 * t + 0.273076135303957) * t + 1.0)
            return t * w / a if d > 0 else a * (w + 0.5 + 0.5)
        if t == 0:
            return 0.0
        p = (
            5.89597428611429e-4,
            -0.00514889771323592,
            0.0076696818164949,
            0.0597275330452234,
            -0.230975380857675,
            -0.409078193005776,
            0.577215664901533,
        )
        top = p[0]
        for k in p[1:]:
            top = top * t + k
        bot = (
            ((0.00423244297896961 * t + 0.0261132021441447) * t + 0.158451672430138) * t + 0.427569613095214
        ) * t + 1.0
        w = top / bot
        return t / a * (w - 0.5 - 0.5) if d > 0 else a * w

    def gamln1(self, a: float) -> float:
        """ln Gamma(1 + a) for -0.2 <= a <= 1.25."""
        if a < 0.6:
            p = (
                -0.00271935708322958,
                -0.0673562214325671,
                -0.402055799310489,
                -0.780427615533591,
                -0.168860593646662,
                0.844203922187225,
                0.577215664901533,
            )
            q = (
                6.67465618796164e-4,
                0.0325038868253937,
                0.361951990101499,
                1.56875193295039,
                3.12755088914843,
                2.88743195473681,
                1.0,
            )
            num, den = p[0], q[0]
            for pk, qk in zip(p[1:], q[1:]):
                num, den = num * a + pk, den * a + qk
            return -a * (num / den)
        x = a - 0.5 - 0.5
        r = (
            4.97958207639485e-4,
            0.017050248402265,
            0.156513060486551,
            0.565221050691933,
            0.848044614534529,
            0.422784335098467,
        )
        s = (
            1.16165475989616e-4,
            0.00713309612391,
            0.10155218743983,
            0.548042109832463,
            1.24313399877507,
            1.0,
        )
        num, den = r[0], s[0]
        for rk, sk in zip(r[1:], s[1:]):
            num, den = num * x + rk, den * x + sk
        return x * (num / den)

    def gamln(self, a: float) -> float:
        """ln Gamma(a) for a > 0. With cfg.gamln == "excel", Excel's GAMMALN (round 1: Excel's beta code inherits
        GAMMALN's errors for v >= 1; for v < 1 w3 found an accurate value instead, see lnb_w3)."""
        if self.c.gamln == "excel":
            return GAMMALN(a)
        if a <= 0.8:
            return self.gamln1(a) - self.log(a)
        if a <= 2.25:
            return self.gamln1(a - 0.5 - 0.5)
        if a < 10.0:
            n = int(a - 1.25)
            t, w = a, 1.0
            for _ in range(n):
                t -= 1.0
                w *= t
            return self.gamln1(t - 1.0) + self.log(w)
        t = 1.0 / (a * a)
        w = (
            (
                (
                    ((-0.00165322962780713 * t + 8.37308034031215e-4) * t + -5.9520293135187e-4) * t
                    + 7.9365066682539e-4
                )
                * t
                + -0.00277777777760991
            )
            * t
            + 0.0833333333333333
        ) / a
        return 0.418938533204673 + w + (a - 0.5) * (self.log(a) - 1.0)

    def gsumln(self, a: float, b: float) -> float:
        """ln Gamma(a + b) for 1 <= a, b <= 2 (Excel mode: GAMMALN(a + b))."""
        if self.c.gamln == "excel":
            return GAMMALN(a + b)
        x = a + b - 2.0
        if x <= 0.25:
            return self.gamln1(x + 1.0)
        if x <= 1.25:
            return self.gamln1(x) + self.alnrel(x)
        return self.gamln1(x - 1.0) + self.log(x * (x + 1.0))

    @staticmethod
    def _stirling_del(c: float, x: float, b: float, r_algdiv: bool = False) -> float:
        """The del(b) correction shared by algdiv and bcorr: c/b times the series in 1/b^2 with s3..s11 in x."""
        x2 = x * x
        s, acc = [], 1.0
        for _ in range(5):  # s3 = x + x2 + 1, s5 = x + x2 s3 + 1, ..., s11
            acc = x + x2 * acc + 1.0
            s.append(acc)
        s3, s5, s7, s9, s11 = s
        if r_algdiv:  # R's algdiv: t = 1/(b*b); TOMS 1992 and bcorr: t = (1/b)^2
            t = 1.0 / (b * b)
        else:
            t = 1.0 / b
            t *= t
        w = (
            (
                ((-0.00165322962780713 * s11 * t + 8.37308034031215e-4 * s9) * t + -5.9520293135187e-4 * s7)
                * t
                + 7.9365066682539e-4 * s5
            )
            * t
            + -0.00277777777760991 * s3
        ) * t + 0.0833333333333333
        return w * (c / b)

    def algdiv(self, a: float, b: float, in_bgrat: bool = False) -> float:
        """ln(Gamma(b)/Gamma(a+b)) for b >= 8."""
        if self.c.algdiv == "gl" or (self.c.algdiv == "gl_bgrat" and in_bgrat):
            return sx("-", GAMMALN(b), GAMMALN(a + b))
        if a > b:
            h = b / a
            c, x, d = 1.0 / (h + 1.0), h / (h + 1.0), a + (b - 0.5)
        else:
            h = a / b
            c, x, d = h / (h + 1.0), 1.0 / (h + 1.0), b + (a - 0.5)
        w = self._stirling_del(c, x, b, self.c.bgrat == "r")
        u = d * self.alnrel(a / b)
        v = a * (self.log(b) - 1.0)
        return w - v - u if u > v else w - u - v

    def bcorr(self, a0: float, b0: float) -> float:
        a, b = min(a0, b0), max(a0, b0)
        h = a / b
        w = self._stirling_del(h / (h + 1.0), 1.0 / (h + 1.0), b)
        t = 1.0 / a
        t *= t
        return (
            (
                (
                    ((-0.00165322962780713 * t + 8.37308034031215e-4) * t + -5.9520293135187e-4) * t
                    + 7.9365066682539e-4
                )
                * t
                + -0.00277777777760991
            )
            * t
            + 0.0833333333333333
        ) / a + w

    def betaln(self, a0: float, b0: float) -> float:
        a, b = min(a0, b0), max(a0, b0)
        if a >= 8.0:
            w = self.bcorr(a, b)
            h = a / b
            u = -(a - 0.5) * self.log(h / (h + 1.0))
            v = b * self.alnrel(h)
            return (
                self.log(b) * -0.5 + 0.918938533204673 + w - v - u
                if u > v
                else self.log(b) * -0.5 + 0.918938533204673 + w - u - v
            )
        if a < 1.0:
            return (
                self.gamln(a) + (self.gamln(b) - self.gamln(a + b))
                if b < 8.0
                else self.gamln(a) + self.algdiv(a, b)
            )
        w = 0.0
        if a < 2.0:
            if b <= 2.0:
                return self.gamln(a) + self.gamln(b) - self.gsumln(a, b)
            if b >= 8.0:
                return self.gamln(a) + self.algdiv(a, b)
        elif b > 1000.0:
            n = int(a - 1.0)
            w = 1.0
            for _ in range(n):
                a -= 1.0
                w *= a / (a / b + 1.0)
            return self.log(w) - n * self.log(b) + (self.gamln(a) + self.algdiv(a, b))
        else:
            n = int(a - 1.0)
            w = 1.0
            for _ in range(n):
                a -= 1.0
                h = a / b
                w *= h / (h + 1.0)
            w = self.log(w)
            if b >= 8.0:
                return w + self.gamln(a) + self.algdiv(a, b)
        n = int(b - 1.0)  # L40: 1 < a <= b < 8, reduction of b
        z = 1.0
        for _ in range(n):
            b -= 1.0
            z *= b / (a + b)
        return w + self.log(z) + (self.gamln(a) + (self.gamln(b) - self.gsumln(a, b)))

    def psi(self, x: float) -> float:
        """Digamma (only apser needs it; apser fires for a0 < 1e-15 only)."""
        return float(mpmath.digamma(x))

    # TOMS erf / erfc1 coefficients: |x| <= 0.5 (A/B), 0.5 < |x| <= 4 (P/Q), |x| > 4 (R/S in 1/x^2)
    _EA = (7.7105849500132e-5, -0.00133733772997339, 0.0323076579225834, 0.0479137145607681, 0.128379167095513)
    _EB = (0.00301048631703895, 0.0538971687740286, 0.375795757275549)
    _EP = (-1.36864857382717e-7, 0.564195517478974, 7.21175825088309, 43.1622272220567, 152.98928504694,
           339.320816734344, 451.918953711873, 300.459261020162)
    _EQ = (1.0, 12.7827273196294, 77.0001529352295, 277.585444743988, 638.980264465631, 931.35409485061,
           790.950925327898, 300.459260956983)
    _ER = (2.10144126479064, 26.2370141675169, 21.3688200555087, 4.6580782871847, 0.282094791773523)
    _ES = (94.153775055546, 187.11481179959, 99.0191814623914, 18.0124575948747)

    @staticmethod
    def _horner(cs, t: float, last: float | None = None) -> float:
        v = cs[0]
        for k in cs[1:]:
            v = v * t + k
        return v if last is None else v * t + last

    def _erf_small(self, x: float) -> float:
        t = x * x
        return x * ((self._horner(self._EA, t) + 1.0) / self._horner(self._EB, t, 1.0))

    def _erfc_mid(self, ax: float) -> float:
        return self._horner(self._EP, ax) / self._horner(self._EQ, ax)

    def _erfc_tail(self, t: float) -> float:
        return self._horner(self._ER, t) / self._horner(self._ES, t, 1.0)

    def erf(self, x: float) -> float:
        ax = abs(x)
        if ax <= 0.5:
            return self._erf_small(x)
        if ax <= 4.0:
            r = 0.5 - self.exp(-x * x) * self._erfc_mid(ax) + 0.5
        elif ax >= 5.8:
            return 1.0 if x > 0 else -1.0
        else:
            x2 = x * x
            t = (0.564189583547756 - self._erfc_tail(1.0 / x2) / x2) / ax
            r = 0.5 - self.exp(-x2) * t + 0.5
        return -r if x < 0 else r

    def erfc1(self, ind: int, x: float) -> float:
        """erfc(x), or exp(x^2) erfc(x) when ind != 0."""
        ax = abs(x)
        if ax <= 0.5:
            r = 0.5 - self._erf_small(x) + 0.5
            return self.exp(x * x) * r if ind else r
        if ax <= 4.0:
            r = self._erfc_mid(ax)
        else:
            if x <= -5.6:
                return self.exp(x * x) * 2.0 if ind else 2.0
            if ind == 0 and (x > 100.0 or x * x > -self.exparg(1)):
                return 0.0
            t = 1.0 / (x * x)
            r = (0.564189583547756 - t * self._erfc_tail(t)) / ax
        if ind:
            return self.exp(x * x) * 2.0 - r if x < 0 else r
        w = x * x
        r = (0.5 - (w - w) + 0.5) * self.exp(-w) * r
        return 2.0 - r if x < 0 else r

    # ------------------------------------------------------------------ Excel's density (w1 model M1)
    def density(self, x: float, a: float, b: float) -> tuple[float, bool]:
        """Excel's BETA.DIST(x, a, b, FALSE). Returns (value, modelled). w1's M1 for min(a, b) <= 1: LN(x) stored;
        ln(1-x) = log1p(-x) for x <= 0.375 else LN(fl(1-x)); tx = fl(fl(a-1) LN x), ty = fl(fl(b-1) ly); E =
        tx + (ty - lnB) in extended (b == 1: fl(tx - lnB)); EXP of E. Elsewhere (min > 1) the composition is
        unsolved (Loader-like, w2/w4/w6) and the correctly rounded density stands in (modelled = False)."""
        if min(a, b) > 1.0:
            X, A, B = mpf(x), mpf(a), mpf(b)
            return float(
                mpmath.exp(
                    (A - 1) * mpmath.log(X) + (B - 1) * mpmath.log1p(-X) - mpmath.log(mpmath.beta(A, B))
                )
            ), False
        lnb = mpf(self.lnb_m1(a, b))
        lnx = mpf(LN(x))
        ly = mpf(log1p_cr(-x) if x <= 0.375 else LN(1.0 - x))
        tx = mpf(sx("*", sx("-", a, 1.0), lnx))
        ty = mpf(sx("*", sx("-", b, 1.0), ly))
        if b == 1.0:
            e = mpf(em.fl53(tx - lnb))
        else:
            e = em.fl64(tx + em.fl64(ty - lnb))
        return EXPX(e), True

    def lnb_m1(self, a: float, b: float):
        """M1's lnB: GL form fl(GL(min) + fl(GL(max) - GL(a+b))) for min < 1, max <= 8 (8 inclusive); a = 1 (or b)
        with 2 < max < 8: TOMS L60 with an extended product, = -LN(max); else TOMS betaln with Excel GAMMALN/LN."""
        lo, hi = min(a, b), max(a, b)
        if self.c.pdf_lnb == "gl" or (lo < 1.0 and hi <= 8.0):
            return sx("+", GAMMALN(lo), sx("-", GAMMALN(hi), GAMMALN(a + b)))
        if lo == 1.0 and 2.0 < hi < 8.0:
            return -LN(hi)
        return Kernel(replace(self.c, elem="excel", gamln="excel")).betaln(a, b)

    def pdf_xy(self, a: float, b: float, x: float, y: float) -> float:
        """w4: brcomp/brcmp1 = Excel's density at x times x times y, as fl(P * fl(x*y))."""
        p, _ = self.density(x, a, b)
        return p * (x * y)

    # ------------------------------------------------------------------ prefactors
    def brcomp(self, a: float, b: float, x: float, y: float) -> float:
        """x^a y^b / Beta(a, b)."""
        if x == 0.0 or y == 0.0:
            return 0.0
        if self.c.brcomp == "pdf":
            return self.pdf_xy(a, b, x, y)
        return self.brcmp1(0, a, b, x, y, toms=True)

    def brcmp1(self, mu: int, a: float, b: float, x: float, y: float, toms: bool = False) -> float:
        """exp(mu) x^a y^b / Beta(a, b). brcomp is the mu = 0 case (TOMS duplicates the code; esum(0, z) =
        exp(z) exactly, so one body serves both)."""
        if not toms and self.c.brcmp1 == "pdf":
            return self.pdf_xy(a, b, x, y) * (self.exp(float(mu)) if mu else 1.0)
        a0 = min(a, b)
        if a0 < 8.0:
            if x <= 0.375:
                lnx, lny = self.log(x), self.alnrel(-x)
            elif y > 0.375:
                lnx, lny = self.log(x), self.log(y)
            else:
                lnx, lny = self.alnrel(-y), self.log(y)
            z = a * lnx + b * lny
            if a0 >= 1.0:
                return self.esum(mu, z - self.betaln(a, b))
            b0 = max(a, b)
            if b0 >= 8.0:
                u = self.gamln1(a0) + self.algdiv(a0, b0)
                return a0 * self.esum(mu, z - u)
            if b0 <= 1.0:
                ans = self.esum(mu, z)
                if ans == 0.0:
                    return 0.0
                apb = a + b
                zz = (self.gam1(apb - 1.0) + 1.0) / apb if apb > 1.0 else self.gam1(apb) + 1.0
                c = (self.gam1(a) + 1.0) * (self.gam1(b) + 1.0) / zz
                return ans * (a0 * c) / (a0 / b0 + 1.0)
            u = self.gamln1(a0)
            n = int(b0 - 1.0)
            if n >= 1:
                c = 1.0
                for _ in range(n):
                    b0 -= 1.0
                    c *= b0 / (a0 + b0)
                u = self.log(c) + u
            z -= u
            b0 -= 1.0
            apb = a0 + b0
            t = (self.gam1(apb - 1.0) + 1.0) / apb if apb > 1.0 else self.gam1(apb) + 1.0
            return a0 * self.esum(mu, z) * (self.gam1(b0) + 1.0) / t
        apb = a + b
        lam = a - apb * x if a <= b else apb * y - b
        if a <= b:
            h = a / b
            x0, y0 = h / (h + 1.0), 1.0 / (h + 1.0)
        else:
            h = b / a
            x0, y0 = 1.0 / (h + 1.0), h / (h + 1.0)
        e = -lam / a
        u = e - self.log(x / x0) if abs(e) > 0.6 else self.rlog1(e)
        e = lam / b
        v = e - self.log(y / y0) if abs(e) > 0.6 else self.rlog1(e)
        z = self.esum(mu, -(a * u + b * v))
        return 0.398942280401433 * math.sqrt(b * x0) * z * self.exp(-self.bcorr(a, b))

    # ------------------------------------------------------------------ series and fractions
    def fpser(self, a: float, b: float, x: float, eps: float) -> float:
        ans = 1.0
        if a > eps * 0.001:
            t = a * self.log(x)
            if t < self.exparg(1):
                return 0.0
            ans = self.exp(t)
        ans *= b / a
        tol, an, t = eps / a, a + 1.0, x
        s = t / an
        while True:
            an += 1.0
            t = x * t
            c = t / an
            s += c
            if abs(c) <= tol:
                break
        return ans * (a * s + 1.0)

    def apser(self, a: float, b: float, x: float, eps: float) -> float:
        g = 0.577215664901533
        bx = b * x
        t = x - bx
        c = self.log(x) + self.psi(b) + g + t if b * eps <= 0.02 else self.log(bx) + g + t
        tol, j, s = eps * 5.0 * abs(c), 1.0, 0.0
        while True:
            j += 1.0
            t *= x - bx / j
            aj = t / j
            s += aj
            if abs(aj) <= tol:
                break
        return -a * (c + s)

    def bpser(self, a: float, b: float, x: float, eps: float) -> float:
        """I_x(a, b) by the power series, for b <= 1 or b x <= 0.7."""
        if x == 0.0:
            return 0.0
        if self.c.bpser == "w3":
            return self.bpser_w3(a, b, x)
        a0 = min(a, b)
        if a0 >= 1.0:
            ans = self.exp(a * self.log(x) - self.betaln(a, b)) / a
        else:
            b0 = max(a, b)
            if b0 >= 8.0:
                u = self.gamln1(a0) + self.algdiv(a0, b0)
                ans = a0 / a * self.exp(a * self.log(x) - u)
            elif b0 <= 1.0:
                ans = math.pow(x, a) if self.c.elem == "libm" else EXP(a * LN(x))
                if ans == 0.0:
                    return 0.0
                apb = a + b
                z = (self.gam1(a + b - 1.0) + 1.0) / apb if apb > 1.0 else self.gam1(apb) + 1.0
                c = (self.gam1(a) + 1.0) * (self.gam1(b) + 1.0) / z
                ans *= c * (b / apb)
            else:
                u = self.gamln1(a0)
                m = int(b0 - 1.0)
                if m >= 1:
                    c = 1.0
                    for _ in range(m):
                        b0 -= 1.0
                        c *= b0 / (a0 + b0)
                    u += self.log(c)
                z = a * self.log(x) - u
                b0 -= 1.0
                apb = a0 + b0
                t = (self.gam1(a0 + b0 - 1.0) + 1.0) / apb if apb > 1.0 else self.gam1(apb) + 1.0
                ans = self.exp(z) * (a0 / a) * (self.gam1(b0) + 1.0) / t
        if ans == 0.0 or a <= eps * 0.1:
            return ans
        tol, n, s, c = eps / a, 0.0, 0.0, 1.0
        while True:
            n += 1.0
            c *= (0.5 - b / n + 0.5) * x
            w = c / (a + n)
            s += w
            if not (n < 1e7 and abs(w) > tol):
                break
        return ans * (a * s + 1.0) if a * s > -1.0 else 0.0

    def lnb_w3(self, a: float, b: float):
        """w3's ln B for bpser, an extended value: fl64(lg(a) + fl64(lg(b) - lg(a+b))), lg = accurate ln Gamma
        (64-bit) for v < 1 and Excel GAMMALN for v >= 1; max(a, b) > 8: the accurate ln B (proxy for algdiv)."""
        if max(a, b) > 8.0:
            if self.c.w3_big == "toms":  # TOMS betaln (gamln(min) + algdiv) with Excel GAMMALN/LN, a double
                return mpf(self.betaln(a, b))
            return em.fl64(mpmath.log(mpmath.beta(mpf(a), mpf(b))))

        def lg(v):
            if v >= 1.0 or self.c.w3_lg == "gl":
                return mpf(GAMMALN(v))
            if self.c.w3_lg == "toms":  # TOMS gamln for v <= 0.8: gamln1(v) - ln v, kept extended
                return em.fl64(mpf(self.gamln1(v)) - mpf(LN(v)))
            return em.fl64(mpmath.loggamma(mpf(v)))

        p, q = (min(a, b), max(a, b)) if self.c.w3_order == "min" else (a, b)
        return em.fl64(lg(p) + em.fl64(lg(q) - lg(sx("+", a, b))))

    def bpser_w3(self, a: float, b: float, x: float) -> float:
        """w3: s = fl(1/a) + sum c_n/(a+n), c_n = (c_{n-1} ((0.5 - b/n) + 0.5)) x, every op stored x87, summed until
        s stops changing; z = fl(a LN x) - L; I = fl(EXP(z) s)."""
        s = sx("/", 1.0, a)
        c, n = 1.0, 0.0
        for _ in range(200000):
            n += 1.0
            c = sx("*", sx("*", c, sx("+", sx("-", 0.5, sx("/", b, n)), 0.5)), x)
            s2 = sx("+", s, sx("/", c, sx("+", a, n)))
            if s2 == s:
                break
            s = s2
        z = mpf(sx("*", a, LN(x))) - self.lnb_w3(a, b)
        e = EXP(em.fl53(z)) if self.c.w3_z == "z53" else EXPX(em.fl64(z))
        return sx("*", e, s)

    def bup(self, a: float, b: float, x: float, y: float, n: int, eps: float) -> float:
        """I_x(a, b) - I_x(a + n, b) for a positive integer n."""
        apb, ap1 = a + b, a + 1.0
        mu, d = 0, 1.0
        if n > 1 and a >= 1.0 and apb >= ap1 * 1.1:
            mu = min(int(abs(self.exparg(1))), int(self.exparg(0)))
            d = self.exp(-float(mu))
        if self.c.brcmp1 == "pdf":
            # w4: bup = fl(fl(P fl(x y)) / a) W with P Excel's density (no exp(mu) scaling in the density path)
            mu, d = 0, 1.0
        ret = self.brcmp1(mu, a, b, x, y) / a
        if n == 1 or ret == 0.0:
            return ret
        nm1 = n - 1
        k = 0
        if b > 1.0:
            if y > 1e-4:
                r = (b - 1.0) * x / y - a
                if r >= 1.0:
                    k = int(r) if r < nm1 else nm1
            else:
                k = nm1
        w = d
        for i in range(nm1):
            d = self._bup_term(d, apb, ap1, float(i), x)
            w = sx("+", w, d) if self.c.bup_arith == "w4" else w + d
            if i >= k and self.c.bup_break and d <= eps * w:
                break
        return ret * w

    def _bup_term(self, d: float, apb: float, ap1: float, l: float, x: float) -> float:
        if self.c.bup_arith == "w4":
            r = LD(float(LD(apb + l) / LD(ap1 + l)))  # apb + l, ap1 + l and the ratio stored as doubles
            return float(LD(d) * (r * LD(x)))  # ratio * x kept in an x87 register, d stored
        return d * ((apb + l) / (ap1 + l) * x)

    def bfrac(self, a: float, b: float, x: float, y: float, lam: float, eps: float) -> float:
        """Continued fraction for I_x(a, b), a, b > 1, lambda = (a + b) y - b."""
        brc = self.brcomp(a, b, x, y)
        if brc == 0.0:
            return 0.0
        c, c0, c1, yp1 = lam + 1.0, b / a, 1.0 / a + 1.0, y + 1.0
        n, p, s, an, bn, anp1 = 0.0, 1.0, a + 1.0, 0.0, 1.0, 1.0
        bnp1 = c / c1
        r = c1 / c
        while n < 1000:
            n += 1.0
            t = n / a
            r_style = self.c.bgrat == "r"  # R: w = n x (b - n), beta = w/s + (n + e(...)); 1992: N*(B-N)*X, N + W/S + E*(...)
            w = n * x * (b - n) if r_style else n * (b - n) * x
            e = a / s
            alpha = p * (p + c0) * e * e * (w * x)
            e = (t + 1.0) / (c1 + t + t)
            beta = w / s + (n + e * (c + n * yp1)) if r_style else n + w / s + e * (c + n * yp1)
            p = t + 1.0
            s += 2.0
            t = alpha * an + beta * anp1
            an, anp1 = anp1, t
            t = alpha * bn + beta * bnp1
            bn, bnp1 = bnp1, t
            r0, r = r, anp1 / bnp1
            if abs(r - r0) <= eps * r:
                break
            an /= bnp1
            bn /= bnp1
            anp1, bnp1 = r, 1.0
        return brc * r

    def grat1(self, a: float, x: float, r: float, eps: float) -> float:
        """Q(a, x) for a <= 1 (TOMS grat1's q; r = exp(-x) x^a / Gamma(a))."""
        if a * x == 0.0:
            return 1.0 if x <= a else 0.0
        if a == 0.5:
            if x < 0.25:
                return 0.5 + (0.5 - self.erf(math.sqrt(x)))
            return self.erfc1(0, math.sqrt(x))
        if x < 1.1:
            an, c = 3.0, x
            s = x / (a + 3.0)
            tol = 0.1 * eps / (a + 1.0)
            while True:
                an += 1.0
                c = -c * (x / an)
                t = c / (a + an)
                s += t
                if abs(t) <= tol:
                    break
            j = a * x * ((s / 6.0 - 0.5 / (a + 2.0)) * x + 1.0 / (a + 1.0))
            z = a * self.log(x)
            h = self.gam1(a)
            g = 1.0 + h
            r_style = self.c.bgrat == "r"  # R tests z > -0.13394 for every x and forms (l + 0.5 + 0.5)
            if (x >= 0.25 and a < x / 2.59) or ((r_style or x < 0.25) and z > -0.13394):
                l = self.rexpm1(z)
                q = (((l + 0.5 + 0.5) if r_style else (0.5 + (0.5 + l))) * j - l) * g - h
                return max(q, 0.0)
            p = self.exp(z) * g * (0.5 + (0.5 - j))
            return 0.5 + (0.5 - p)
        a2nm1, a2n, b2nm1, b2n, c = 1.0, 1.0, x, x + (1.0 - a), 1.0
        while True:
            a2nm1 = x * a2n + c * a2nm1
            b2nm1 = x * b2n + c * b2nm1
            am0 = a2nm1 / b2nm1
            c += 1.0
            cma = c - a
            a2n = a2nm1 + cma * a2n
            b2n = b2nm1 + cma * b2n
            an0 = a2n / b2n
            if abs(an0 - am0) < eps * an0:
                break
        return r * an0

    def bgrat(self, a: float, b: float, x: float, y: float, w: float, eps: float) -> float:
        """w + I_x(a, b) by the asymptotic expansion for a >= 15, b <= 1 (returns the new w)."""
        bm1 = b - 0.5 - 0.5
        nu = a + 0.5 * bm1
        lnx = self.log(x) if y > 0.375 else self.alnrel(-y)
        z = -nu * lnx
        if b * z == 0.0:
            return w
        if self.c.bgrat == "orig":
            r = b * (1.0 + self.gam1(b)) * self.exp(b * self.log(z))
            r = r * self.exp(a * lnx) * self.exp(0.5 * bm1 * lnx)
            u = self.algdiv(b, a, True) + b * self.log(nu)
            u = r * self.exp(-u)
            if u == 0.0:
                return w
            q_r = self.grat1(b, z, r, eps) / r
        else:  # R: log_r = log(b) + log1p(gam1(b)) + b log(z) + nu lnx
            log_r = self.log(b) + math.log1p(self.gam1(b)) + b * self.log(z) + nu * lnx
            log_u = log_r - (self.algdiv(b, a, True) + b * self.log(nu))
            u = self.exp(log_u)
            q_r = self.grat_r(b, z, log_r, eps)
        v = 0.25 * (1.0 / nu) ** 2 if self.c.bgrat == "orig" else 0.25 / (nu * nu)
        t2 = lnx * 0.25 * lnx if self.c.bgrat == "r" else 0.25 * lnx * lnx
        l = w / u if w != 0.0 else 0.0
        j = q_r
        s = j
        t, cn, n2 = 1.0, 1.0, 0.0
        cs, ds = [], []
        for n in range(1, 31):
            bp2n = b + n2
            j = (bp2n * (bp2n + 1.0) * j + (z + bp2n + 1.0) * t) * v
            n2 += 2.0
            t *= t2
            cn /= n2 * (n2 + 1.0)
            cs.append(cn)
            acc = 0.0
            coef = b - n
            for i in range(1, n):
                acc += coef * cs[i - 1] * ds[n - 1 - i]
                coef += b
            ds.append(bm1 * cn + acc / n)
            dj = ds[-1] * j
            s += dj
            if s <= 0.0 or abs(dj) <= eps * (s + l):
                break
        return w + u * s

    def grat_r(self, a: float, x: float, log_r: float, eps: float) -> float:
        """R's grat_r = Q(a, x)/r (for the R-like control only)."""
        if a * x == 0.0:
            return math.exp(-log_r) if x <= a else 0.0
        if a == 0.5:
            if x < 0.25:
                return (0.5 - self.erf(math.sqrt(x)) + 0.5) * math.exp(-log_r)
            sx_ = math.sqrt(x)
            return self.erfc1(1, sx_) / sx_ * 1.772453850905516  # M_SQRT_PI
        if x < 1.1:
            return self.grat1(a, x, 1.0, eps) * self.exp(-log_r)
        return self.grat1(a, x, 1.0, eps)

    def basym(self, a: float, b: float, lam: float, eps: float) -> float:
        """Asymptotic expansion for large a and b (lambda >= 0, a, b >= 15)."""
        e0, e1 = 1.12837916709551, 0.353553390593274
        f = a * self.rlog1(-lam / a) + b * self.rlog1(lam / b)
        t = self.exp(-f)
        if t == 0.0:
            return 0.0
        z0 = math.sqrt(f)
        z = z0 / e1 * 0.5
        z2 = f + f
        if a < b:
            h = a / b
            r0, r1, w0 = 1.0 / (h + 1.0), (b - a) / b, 1.0 / math.sqrt(a * (h + 1.0))
        else:
            h = b / a
            r0, r1, w0 = 1.0 / (h + 1.0), (b - a) / a, 1.0 / math.sqrt(b * (h + 1.0))
        A = [0.0] * 21
        B = [0.0] * 21
        C = [0.0] * 21
        D = [0.0] * 21
        A[0] = r1 * 0.66666666666666663
        C[0] = A[0] * -0.5
        D[0] = -C[0]
        j0 = 0.5 / e0 * self.erfc1(1, z0)
        j1 = e1
        s = j0 + D[0] * w0 * j1
        ss, h2, hn, w, znm1, zn = 1.0, h * h, 1.0, w0, z, z2
        for n in range(2, 21, 2):
            hn *= h2
            A[n - 1] = r0 * 2.0 * (h * hn + 1.0) / (n + 2.0)
            np1 = n + 1
            ss += hn
            A[np1 - 1] = r1 * 2.0 * ss / (n + 3.0)
            for i in range(n, np1 + 1):
                r = (i + 1.0) * -0.5
                B[0] = r * A[0]
                for m in range(2, i + 1):
                    bsum = 0.0
                    for jj in range(1, m):
                        bsum += (jj * r - (m - jj)) * A[jj - 1] * B[m - jj - 1]
                    B[m - 1] = r * A[m - 1] + bsum / m
                C[i - 1] = B[i - 1] / (i + 1.0)
                dsum = 0.0
                for jj in range(1, i):
                    dsum += D[i - jj - 1] * C[jj - 1]
                D[i - 1] = -(dsum + C[i - 1])
            j0 = e1 * znm1 + (n - 1.0) * j0
            j1 = e1 * zn + n * j1
            znm1 *= z2
            zn *= z2
            w *= w0
            t0 = D[n - 1] * w * j0
            w *= w0
            t1 = D[np1 - 1] * w * j1
            s += t0 + t1
            if abs(t0) + abs(t1) <= eps * s:
                break
        return e0 * t * self.exp(-self.bcorr(a, b)) * s

    # ------------------------------------------------------------------ bratio
    def closed(self, a0: float, b0: float, x0: float, y0: float):
        """w5's closed forms on the oriented shapes: (w, w1, name) or None."""
        if b0 == 1.0:
            w = EXP(a0 * LN(x0))
            return w, (0.5 - w) + 0.5, "closed:b0=1"
        if a0 == 1.0 and y0 <= 0.9:
            w1 = EXP(b0 * LN(1.0 - x0))
            return (0.5 - w1) + 0.5, w1, "closed:a0=1"
        return None


def _finish(swap: bool, w: float, w1: float, name: str):
    return (w1, w, name + "/sw") if swap else (w, w1, name)


def kernel(a: float, b: float, x: float, y: float, cfg: Cfg = BEST):
    """Excel's incomplete-beta kernel K(a, b, x, y): (I_x(a,b), 1 - I_x(a,b), branch)."""
    k = _KERNELS.get(cfg)
    if k is None:
        k = _KERNELS[cfg] = Kernel(cfg)
    eps = Kernel.EPS
    if x == 0.0:
        return 0.0, 1.0, "x=0"
    if y == 0.0:
        return 1.0, 0.0, "y=0"
    if max(a, b) < eps * 0.001:
        return b / (a + b), a / (a + b), "tiny"
    if min(a, b) <= 1.0:
        swap = x > 0.5
        a0, b0, x0, y0 = (b, a, y, x) if swap else (a, b, x, y)
        if cfg.closed:
            cf = k.closed(a0, b0, x0, y0)
            if cf is not None:
                return _finish(swap, *cf)
        if b0 < min(eps, eps * a0):
            w = k.fpser(a0, b0, x0, eps)
            return _finish(swap, w, 0.5 - w + 0.5, "fpser")
        if a0 < min(eps, eps * b0) and b0 * x0 <= 1.0:
            w1 = k.apser(a0, b0, x0, eps)
            return _finish(swap, 0.5 - w1 + 0.5, w1, "apser")
        n = 0
        if max(a0, b0) > 1.0:
            if b0 <= 1.0:
                return _w_bpser(k, swap, a0, b0, x0, eps, "w_bpser")
            if x0 >= cfg.x29:
                return _w1_bpser(k, swap, a0, b0, y0, eps)
            if x0 < 0.1 and math.pow(x0 * b0, a0) <= 0.7:
                return _w_bpser(k, swap, a0, b0, x0, eps, "w_bpser")
            if b0 > 15.0:
                w1 = k.bgrat(b0, a0, y0, x0, 0.0, 15.0 * eps)
                return _finish(swap, 0.5 - w1 + 0.5, w1, "bgrat")
        else:
            if a0 >= min(0.2, b0) or math.pow(x0, a0) <= 0.9:
                return _w_bpser(k, swap, a0, b0, x0, eps, "w_bpser")
            if x0 >= 0.3:
                return _w1_bpser(k, swap, a0, b0, y0, eps)
        n = 20
        w1 = k.bup(b0, a0, y0, x0, n, eps)
        w1 = k.bgrat(b0 + n, a0, y0, x0, w1, 15.0 * eps)
        return _finish(swap, 0.5 - w1 + 0.5, w1, "bup+bgrat")
    lam = (a + b) * y - b if a > b else a - (a + b) * x
    swap = lam < 0.0
    lam = abs(lam)
    a0, b0, x0, y0 = (b, a, y, x) if swap else (a, b, x, y)
    if b0 < 40.0:
        if b0 * x0 <= 0.7:
            return _w_bpser(k, swap, a0, b0, x0, eps, "bpser(a,b>1)")
        n = int(b0)  # L140
        b0 -= n
        if b0 == 0.0:
            n -= 1
            b0 = 1.0
        w = k.bup(b0, a0, y0, x0, n, eps)
        if x0 <= 0.7:
            w = w + k.bpser(a0, b0, x0, eps)
            return _finish(swap, w, 0.5 - w + 0.5, "L140 bup+bpser")
        name = "L140 bup+bgrat"
        if a0 <= 15.0:
            w += k.bup(a0, b0, x0, y0, 20, eps)
            a0 += 20
            name = "L140 bup+bup+bgrat"
        w = k.bgrat(a0, b0, x0, y0, w, 15.0 * eps)
        return _finish(swap, w, 0.5 - w + 0.5, name)
    if (a0 > b0 and (b0 <= 100.0 or lam > b0 * 0.03)) or (a0 <= b0 and (a0 <= 100.0 or lam > a0 * 0.03)):
        w = k.bfrac(a0, b0, x0, y0, lam, 15.0 * eps)
        return _finish(swap, w, 0.5 - w + 0.5, "bfrac")
    w = k.basym(a0, b0, lam, 100.0 * eps)
    return _finish(swap, w, 0.5 - w + 0.5, "basym")


def _w_bpser(k: Kernel, swap, a0, b0, x0, eps, name):
    w = k.bpser(a0, b0, x0, eps)
    return _finish(swap, w, 0.5 - w + 0.5, name)


def _w1_bpser(k: Kernel, swap, a0, b0, y0, eps):
    w1 = k.bpser(b0, a0, y0, eps)
    return _finish(swap, 0.5 - w1 + 0.5, w1, "w1_bpser")


_KERNELS: dict = {}


def run(points, cfg: Cfg = BEST):
    """kernel() over [(a, b, x, y)], fetching missing GAMMALN values from the harness and re-evaluating."""
    for _ in range(4):
        out = [kernel(a, b, x, y, cfg) for a, b, x, y in points]
        if not _GL_MISSING:
            return out
        fetch_missing()
    return out
