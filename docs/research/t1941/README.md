# T-1941 research notes: Excel GAMMALN, GAMMA and LN

Data: `eval/suites/probes-numeric/cases/kmyst-gam-*`.
- r1xx: 16,384 consecutive doubles from irregular starts.
- r2xx: 8,192 consecutive doubles centred on candidate boundaries.
- Columns: A = x, B = GAMMALN.PRECISE(x), C = LN(x), D = GAMMA(x).

`extract_runs.py OUT` dumps the rows as "x_hex B_hex C_hex D_hex case", with nan for errors. Run it from the repo root.

## Findings so far
- **LN is bit-exact (x87 FLDLN2; FYL2X).** LN(x) = RN53(RN64(RN64(ln 2) * log2(x))).
  - It matches 114,687 of 114,688 points (`lnmodels.py`).
  - The one miss, x = 0.612354100000274, has its exact value 0.008 of a 64-bit ulp from a 64-bit midpoint, an
    FYL2X near-tie like EXP's F2XM1 near-ties (T-1778).
  - Plain double rounding RN53(RN64(ln x)) gives 114,674; correctly rounded gives 114,658.
- **GAMMALN for tiny x is fl(-LN(x) + fl(x*(-gamma)))**, with LN the model above and gamma Euler's constant
  as a double. That is 16,384 of 16,384 points near x = 1.37e-9 (`small.py`). It is Cody's ALGAMA form,
  -log(x) + x*(D1 + x*R(x)) with D1 = -gamma, where x*R is negligible at that x.
- **GAMMA is a separate kernel**, not EXP(GAMMALN).
  - Near x = 1.37e-9 it runs 1.5-2.4 ulps high. The implied numerator g*x is about 1.2*2^-52 above
    Gamma(1+x), consistent with a stored Gamma(1+x)-like value about 3 ulps off, then divided by x.
- Helpers: `rnd.py` (exact RN53/RN64), `exp_v6.py` (Excel EXP, T-1778).
