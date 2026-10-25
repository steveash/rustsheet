"""T-2464 r2-bratio: TOMS 708's special functions (alnrel ... erfc1) as a mixin of bratio_excel.Kernel, and the
x87 stored operation `sx`. Split out of bratio_excel.py to keep it under the repo's file-size limit; read it as part
of that module. The mixin uses self.c (the Cfg), self.exp / self.log (Excel's or libm's) and self.gammaln_xl
(Excel's GAMMALN), all set by Kernel.__init__.
"""

from __future__ import annotations

import math

import mpmath
import numpy as np

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


class TomsSpecial:
    """TOMS 708's elementary pieces; method names are TOMS's."""

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
            return self.gammaln_xl(a)
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
        """ln Gamma(a + b) for 1 <= a, b <= 2 (Excel mode: self.gammaln_xl(a + b))."""
        if self.c.gamln == "excel":
            return self.gammaln_xl(a + b)
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
            return sx("-", self.gammaln_xl(b), self.gammaln_xl(a + b))
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
