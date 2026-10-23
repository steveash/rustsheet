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

## Results (2026-10-23)
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

Engine tasks: T-2501 to T-2511 (plan in r2-impact section 4, plus r2-tdist). Research follow-ups: T-2512 and T-2513.

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

## Rebuilding the harness
`excelmath.py` runs the ignored test `functions::statistical::t2464_harness::t2464_kernels`
(crates/rustsheet-engine/src/functions/statistical/t2464_harness.rs). Build it with
`cargo test --release -p rustsheet_engine --lib t2464_kernels --no-run`, then copy the newest
`target/release/deps/rustsheet_engine-*` executable to `docs/research/t2464/.harness-bin` (git-ignored), or point
`T2464_HARNESS` at it.
