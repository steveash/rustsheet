# T-2464: cracking Excel's regularized incomplete beta

Goal: a model of Excel's I_x(a, b) (BETA.DIST cdf) and of its beta density (BETA.DIST pdf) that is bit-exact, or at
least within half a unit of the 15th significant digit, on the oracle data below. The same kernel drives F.DIST,
F.DIST.RT, F.TEST, T.DIST*, T.TEST, BINOM.DIST, NEGBINOM.DIST and the inverses (about 5,600 visibly-off cells,
31% of all, on 2026-10-23).

Read first: docs/research/excel-kernel-playbook.md (priors, probe design, traps). Then statistics.md entries
"Incomplete beta: Excel's closed forms and the best candidate (T-1744)", "T.DIST.RT is BETA.DIST's incomplete beta,
switched at the mean", "Student t prefactor per df" (B(a,b) = EXP(GAMMALN(a) + (GAMMALN(b) - GAMMALN(a+b))) for
a < 1, b <= 8: the TOMS 708 betaln structure), "T.DIST.RT's noise beyond t = 1 grows with a ln y", and
"BETA.INV solves Excel's own CDF to the crossing".

## Results (2026-10-23, round 3 2026-10-24)
The findings are in docs/excel-semantics/statistics.md, in nine "(T-2464)" entries. They start at "Incomplete beta is
TOMS 708 `bratio`" and cover:
- the dispatch and the callers' arguments;
- the oriented closed forms;
- Excel's log1p;
- the density for min(a,b) <= 1 (M1) and for min(a,b) > 1 (Loader, PARTIAL);
- bpser;
- L140 and bfrac;
- the Student t tails;
- the integrated model, the open items and the rejected structures.

Engine tasks: T-2645 and T-2502 to T-2511 (plan in r2-impact section 4, plus r2-tdist). Research follow-ups: T-2512 and T-2513.

Each worker has a `RESULTS.md` in its directory. Its `verify-*` twin holds an independent re-score. Where the two
disagree, the verifier wins.
- Round 1:
  - `w1-density`: density M1;
  - `w2-forensics`: probes, tie response;
  - `w3-series`: bpser;
  - `w4-fractions`: L140, bfrac, Loader lead;
  - `w5-triangulation`: one kernel, closed forms, argument forms, `ix_evidence.tsv`;
  - `w6-literature`: source survey, lnB cut 8.
- Round 2:
  - `r2-bratio`: the integrated TOMS 708 port `bratio_excel.py`, x0 >= 0.3;
  - `r2-log1p`: Excel's log1p;
  - `r2-density-large`: Loader density;
  - `r2-bpser-small`: GL53 lnB, no fpser or apser;
  - `r2-tdist`: T call form, bgrat at b = 1/2;
  - `r2-impact`: engine plan.
- Verifier overrides:
  - r2-bratio rounded formula literals to 15 digits where Excel truncates them. That refutes its "(7.3, 0.6) lnB +24
    units" and "(30.7, 1.7) sd 47".
  - r2-log1p's `hold` case was not untouched.
  - r2-density-large's hold-out does not test bd0.
  - r2-impact's hold-out is not a real one.
- Round 3 (T-2513, 2026-10-24; the densities), recorded in statistics.md's density entries and four new "(T-2513)"
  entries (T density, BINOM pmf, NEGBINOM pmf, F density):
  - `r3-m1-residual`: density M2 for min(a,b) <= 1 (exponent split EXP(hi)·fl(1 + lo), lnB = -LN(the other shape) at
    a = 1 or b = 1, Excel's measured GAMMALN); algdiv for max > 8 still open;
  - `r3-loader-const`: Loader's constant for a, b > 1 (extended E, stored k, n, nk, accurate stirlerr; 2-4 units left);
  - `r3-loader-bd0`: bd0 branches (M - k stored in the log form, cut 0.2, a +δ(fl(2kv)) series term; series open);
  - `r3-tdensity`: T.DIST(.,FALSE) is Loader's dt for df >= 3, closed forms at df 1 and 2;
  - `r3-binom-pmf`: BINOM k = 0 and k = n closed forms, one kernel with BETA.DIST, NEGBINOM on the shifted kernel;
  - `r3-fdensity`: F density negatives and its ratio switch (unsolved).
  - Engine tasks T-2585 (T density) and amended T-2645, T-2503, T-2507; research follow-up T-2586.
- Round 3 verifier overrides:
  - r3-tdensity's per-df constant is not r3-loader-const's stirlerr table: the exact Loader constant does as well or
    better on both hold-outs.
  - r3-loader-bd0's "log-form blocks 656 of 770" was a favourable subset (387 of 640 on a new hold-out), and its CR
    baseline is not comparable with a free-constant fit.
  - r3-binom-pmf's NEGBINOM final step fl(p·B) is not bit-exact.
  - r3-m1-residual's min = 1 rule extends to a < 1 with b = 1 on one shape, (0.37, 1); the engine's GAMMALN is off on
    24 of 66 arguments below 1.
  - r3-fdensity: "ln x is a stored Excel LN product" is not established.

## Verifier code is deliberately independent
Each `verify-*` directory holds a verifier's own parser, scorer and model re-implementation (`vlib.py`, `vparse.py`,
`vdata.py`, ...). They overlap in purpose by design: a verifier was required to write its own code instead of importing
the worker's or another verifier's, so that a shared bug cannot confirm itself. Treat them as frozen evidence for the
verdicts in RESULTS.md and statistics.md, not as a library: new work imports the worker libraries (`excelmath.py`,
r2-bratio's `bratio_excel.py`, r3-loader-bd0's `r3lib.py`, r4-f-small-df's `sdmodel.py`, r5-f-final's
`fdensity_excel.py`) and writes any new verifier from scratch. The repo's duplication ratchet scans these files too
and passes (decision 0101).

## Data (this directory)
- `cells.tsv`: 39,065 oracle-filled cells (excel-graph) whose formula is exactly one call with arguments the fit
  lab resolves: fn, case, cell, args (`|`-joined Python reprs), excel (repr). BETA.DIST 3,046, F.DIST 3,234,
  F.DIST.RT 1,712, T.DIST 19,955, T.DIST.RT 5,034, BINOM.DIST 3,288, NEGBINOM.DIST 1,111, inverses, legacy names.
  Rebuild with `cd tools && uv run python ../docs/research/t2464/extract.py`.
- `beta_exact.tsv`: the 3,052 BETA.DIST/BETADIST cells with A=0, B=1 and 0 < x < 1, with the exact value (mpmath,
  60 digits, correctly rounded), Excel's error in ulps and relative, and the branch (direct: x <= a/(a+b)).
  Baseline: reflected cdf 319/398 bit-exact; direct cdf 98/619 (median |rel| 4.5e-16, 34 cells visibly off);
  pdf about 337/2,035.

## Tools
- Python: `/home/user/rustsheet/bindings/python/.venv/bin/python` (has mpmath). Install more with
  `uv pip install --python /home/user/rustsheet/bindings/python/.venv <pkg>`.
- `excelmath.py`: `excel('EXP'|'LN'|'LOG10'|'GAMMALN'|'GAMMA', [doubles])` returns Excel's value of each, from the
  engine's bit-exact models (Excel's EXP model V6, LN = FLDLN2/FYL2X, GAMMALN/GAMMA from T-1941/T-2129), one
  batch per call (batch your inputs; one call costs ~0.1 s). Also `fl64`, `fl53`, `x87(op, a, b)` =
  fl53(fl64(a op b)) (x87 extended precision rounded to double after every stored op), `ulps`.
- gcc with `-mfpmath=387` and `long double` for exact x87 verifiers (playbook section 3.2).
- The fit lab (`tools/src/rustsheet_tools/fitlab/kernels/beta.py`) has Cephes incbet/incbcf/incbd, NR betacf,
  LibreOffice GetBetaDist and the T-1744 best combination (158/283 on the old grid) as Python code to reuse.
- Prior scripts: docs/research/t1759_beta_direct_branch.py (360 combinations), docs/research/t0969/.

## Rules
- Score every candidate on the same data: bit-exact count, within 4 ulp, and within 15 digits (|excel - model| <=
  half a unit in the 15th significant digit of excel), per branch (pdf, direct cdf, reflected cdf) and overall.
  Record negatives too (decision 0056).
- Positive controls before trusting a method (playbook section 4): check a method on synthetic data from a known
  kernel at the same points.
- Hold-out: report scores on a half of the data you did not tune on.
- Excel's xlsx number parser misreads about 1 input in 1E6; check misses at x +- 1 ulp before blaming the model.
- Work only in your own subdirectory `docs/research/t2464/<your-dir>/`: scripts, a `RESULTS.md` with what you tried
  and the counts, and any data you derive. Do not edit files outside it, do not run git commit/push, and do not
  touch the engine source. The coordinator merges findings.

## Evaluating Excel's elementary functions
`excelmath.excel` evaluates `=NAME(x)` on the engine's own kernels through the python bindings (`import rustsheet`;
`make py-bindings` builds bindings/python/.venv, run the scripts with its python). The ignored test harness
(`t2464_harness.rs`) that did this before is gone (T-2977); its `BRATIO` op is the engine's BETA.DIST, F.DIST and
F.DIST.RT cells now, which take bratio's port where it reaches (`bratio::kernel_tails`).
