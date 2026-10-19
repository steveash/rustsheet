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
- **GAMMALN for x >= 8 is bit-exact: Stirling with exactly 10 Bernoulli terms (W3, verified by the
  coordinator).** Every op is fl53(fl64(.)):
  - corr = Excel LN(y); a = y - 0.5; b = a*corr; d = b - y; e = d + 0.9189385332046727 (ln sqrt(2pi));
  - z = 1/y; z2 = z*z; r = B10; then r = r*z2 + B_k for k = 9..1; s = r*z; GAMMALN = e + s;
  - B_k = B_2k/(2k(2k-1)) correctly rounded to double (1/12, -1/360, ..., -174611/125400).
  - Score: 291,957 of 291,957 points (`gammaln_big_check.c`, `USEC=1 NT=10 ./glc rows.txt 8 1e308`), from 8 to 7.8e299. That includes
    r118-r125, r206-r212 at x >= 8, r301-r303 (up to 7.8e299), r402-r403 and the r401 sweep.
  - 11 terms give 291,931 (12 misses at 8.03 alone), and 9 terms fewer. Cody's ALGAMA coefficients and his divide-by-y^2 form are not
    used. The cut is exactly at 8: below it, r206 matches only 45%.
  - With this machine's hardware FYL2X for LN, 3 points miss. At those, RN64 of the log lands exactly on a
    53-bit tie, where the exact LN model gives Excel's value, so the kernel takes Excel's LN unchanged.
- **GAMMALN for 3 <= x < 8 (W2, structure).** Excel reduces t into [2,3) and forms the ascending stored product
  p = t*(t+1)*...; then GAMMALN = fl(G(t) + LN(p)), with G a base kernel on [2,3) that is still open. With
  G replaced by the exact value it reaches 91-94% exact; the product order matters.
- **GAMMA (W4, structure).** It follows Cephes `gamma`: the downward product while x >= 3, division while x < 2,
  then fl(fl(z*P(t))/Q(t)) on [2,3). It is bit-exact at x near 1.37e-9. Excel's P/Q coefficients are its own,
  running 5-9e-17 above Gamma.
- Helpers: `rnd.py` (exact RN53/RN64), `exp_v6.py` (Excel EXP, T-1778).
