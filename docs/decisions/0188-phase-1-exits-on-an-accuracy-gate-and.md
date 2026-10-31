# 0188 — Phase 1 exits on an accuracy gate and user-representative parity

- **Status:** accepted (2026-10-31, owner rulings, recorded by T-3346).
- **Amends:**
  - decision 0114 item 1 (the Phase 1 exit) and item 3 (which tasks are parked as Phase 1b);
  - decision 0118 item 3: the Phase 1 rate is counted on user-representative suites only (R1(b));
  - decision 0146: the probe rows and the ironcalc-calc rows of the milestone exits count only the cells R1(c) names;
  - decision 0143: its rule also applies to the milestone probe rows (R7);
  - decision 0052 item 1 and CLAUDE.md prime directive 1: a second exception, R3 (`MATH-OVER-EXCEL`), besides
    directive 5;
  - decision 0056: every numeric fit also follows the R4 guards.
- **Context:** the owner's 2026-10-31 assessment of a fresh eval at `main` 921ed6ad.
  - **Visibly-off concentration.** 97% of the visibly-off numbers sit in six distributions: BETA.DIST, F.DIST,
    BINOM.DIST, NEGBINOM.DIST, T.DIST and T.DIST.RT. Nearly all of those cells are fit-lab research sweeps in
    probes-numeric (BETA.DIST: 206,862 of 207,057 cells). On the BETA.DIST density, Excel is closer to 60-digit truth
    than the engine in 338 of 400 sampled cells (median relative error 1.3e-15 for Excel, 5.9e-15 for the engine). A
    correctly rounded kernel would still be visibly off from Excel on 50% of the off BETA.DIST cells and 93% of the off
    F.DIST cells: those cells measure how closely we reproduce Excel's rounding noise, not whether users get right
    numbers.
  - **Distance from Excel.** Of the 248,892 numeric cells that differ from Excel, 60.9% agree to at least 15 digits,
    28.6% to 14, 7.2% to 13, 2.8% to 12, and 0.5% to fewer than 12, mostly adversarial probes.
  - **User-representative suites.** Counting only gen-*, ironcalc-calc, corpus-bench, smoke and fixtures (calipers and
    poi were not fetched), 447 of 506 functions are at ≥ 99.9% within 15 digits; with every suite counted, 425 were
    published and 422 are at HEAD. Real world: calipers is 99.96% bit-exact; corpus-bench has a visibly-off cell in 1
    of 320 workbooks, poi in 3 of 115.
  - **Fresh stratified points.** 36 functions × 120 new stratified points, scored against Excel for the web and
    50-digit mpmath. Functions that port Excel's algorithm generalize: ERF, NORM.*, PMT, NPER, PRICE, YIELD, DURATION,
    Bessel, GAMMALN, WEIBULL, EXPON. These generalize badly (share within 15 digits, fresh points against the public
    cases): T.INV 30% vs 69%, T.INV.2T 14% vs 51%, POISSON.DIST 76% vs 96%, F.INV 63% vs 78%, BINOM.DIST 75% vs 90%,
    T.DIST 88% vs 98%. The cause is a statrs inverse-beta port with probe-tuned thresholds (inverse_beta.rs,
    t_dist.rs) and tuned form selectors.
  - **The engine is wrong where Excel is right** (users hit these):
    1. IRR returns `#NUM!` on conventional cash-flow series whose IRR is near 0 (1e-5 to 3e-4) or near −10%.
    2. RATE returns `#NUM!` on ordinary annuities with fv ≠ 0, e.g.
       `RATE(239,-14841.27102594,344293.4825124,-155496.6323522,0)` is 0.0431054691928255 in Excel.
    3. T.INV.2T with p near 1 loses up to 5 orders of magnitude to cancellation: `T.INV.2T(0.999999999989522,2)`
       gives 1.49e-8, true 1.48e-11. T.INV and T.INV.2T at typical inputs have a median error against the math of
       7.5e-15 and 1.5e-14, against Excel's 3e-16 and 6e-16.
    4. CHISQ.INV and CHISQ.INV.RT return `#NUM!` at df ≥ 2e7, mostly with p near 1. BETA.INV returns `#NUM!` at
       p ≈ 1 − 1e-8 with a small b, where the answer is 1.0.
    5. BINOM.DIST at n ≥ 6e5 and NEGBINOM.DIST with a large number of failures are off 1e-9 to 1e-8 where Excel is
       within about 1e-13.
    6. XIRR on two-flow series over long spans is off about 1e-8 to 1e-7 where Excel is close.
    7. F.DIST at x ≈ 1e-12 with d2 ≈ 7.8e6 is off 5e-4.
  - **Where Excel itself is far from the math** (the engine matches Excel bit for bit unless noted):
    - T.INV, F.INV, BETA.INV and CHISQ.INV.RT at tiny p (p < 1e-150; for CHISQ.INV.RT a tiny p is the right tail):
      Excel is grossly wrong, e.g. `T.INV(1.87E-297,3937)` is −156.96 in Excel and −40.26 in truth. The assessment
      reports the engine returns the true value there; that is evidence T-3358 owes as committed points.
    - NPER at rates 0 < rate < 1e-14: up to 4% off. GAMMA.INV at a huge shape, on its Wilson–Hilferty closed-form
      branch: about 2.7e-4 off.
    - BESSELJ: about 1e-7 typical, 1.8e-5 at most. XIRR: about 1e-8 (solver tolerance). RATE with 1–2 periods: about
      1e-2 at rates near 1e-8. WEIBULL.DIST pdf underflow; DURATION negative for very short bonds under basis 2 and 3.
  - **Code-audit risks.**
    - The fit lab has no hold-out split: fits are scored on the cells they are tuned on (including the gated gen-m9),
      there is no guard on accuracy against the math, and about 15 empirically fitted continuous constants or tables
      are in the engine.
    - Tuned cut-points: annuity.rs `DISCOUNTED_INTEREST_FROM`, cumulative.rs `GEOMETRIC_MIN_LEAD` and
      `LONG_GEOMETRIC_MIN_LEAD`, student_t.rs `LOG_GAMMA_BETA_MIN` (6.5), the complex/trig.rs bands.
    - Extrapolated fits: GAMMA.INV's Wilson–Hilferty `WILSON_HILFERTY_SPREAD` (fitted at 1E13, applied from 2E10);
      f_density.rs `STIRLING_ERROR` past df 56.
    - Others: GAMMALN's K2 dual coefficients, GEOMEAN's 15-digit snap, the IRR fallback.
  - So the Phase 1 rate of decision 0118, counted over every suite, is dominated by research sweeps whose remaining
    misses are within a few units of the 15th digit of two accurate answers, while real accuracy bugs that users hit
    (items 1–7) are not what the exit measures.
- **Decision** (rulings R1–R7):
  - **R1. Phase 1 exit.** Phase 1 exits when all of these hold, in addition to the existing catalog and semantic
    criteria (decision 0114 item 1's catalog, coverage and filed-task rules, decision 0094's coverage, and the
    exit criteria of M1, M2 and M5–M11 as decision 0146 measures them, read through (c) below):
    - **(a)** Gate A (R2) holds.
    - **(b)** **Gate B:** every in-scope function has ≥ 99.9% of its attributed cells within 15 significant digits
      (decision 0118's measure: attribution, the 20-cell fallback, decisions 0134 and 0143 exceptions), counted on
      user-representative evidence only: the gen-* suites, ironcalc-calc, corpus-bench, calipers, poi, smoke and
      fixtures. A run that did not fetch calipers, poi or corpus-bench reports Gate B as not measured.
    - **(c)** Each milestone's probe row and ironcalc-calc row (decision 0146) counts only:
      - visibly-off type, error or text cells, and cells whose wrong number comes from a semantic rule;
      - numeric cells that fail Gate A's envelope: neither within 15 significant digits of Excel nor within ε of the
        math (R2).

      Other visibly-off numeric cells in probes and probes-numeric, the fit-lab research sweeps included, stay measured
      and reported every run, as **Phase 1b evidence**. Until a cell has a math reference (the Gate A spec says how
      one is attached), a visibly-off numeric cell counts as failing the envelope.
    - The existing semantic probe-row criteria and the ironcalc-calc allowance (decision 0146 item 3) are kept.
  - **R2. Gate A (accuracy, priority 1).**
    - Every numeric function in scope gets a committed, stratified reference table: about 40% typical inputs, 30%
      tails and domain edges, 20% extreme-but-valid, 10% log-uniform over the domain. Each point carries a 50-digit
      mpmath value and, where filled, Excel's value from the oracle.
    - `make check` fails if any point is neither within ε = 1e-9 relative of the math (with an absolute floor of
      1e-300 for values near 0) nor within 15 significant digits of Excel's value (decision 0083's rule).
    - It also fails where the engine returns an error and Excel a number, or the reverse.
    - Points where R3 says to follow the math are checked against the math only.
    - The table format, location, generator, oracle fill, the check and the per-function stratification are a spec
      (filed by this record), since the gate is architectural.
  - **R3. Excel versus math.**
    - Prime directive 1 still holds: match Excel, its inaccuracy included, except where Excel's relative error against
      50-digit math exceeds **G = 1e-4** on evidenced points. There the engine follows the math.
    - R3 never overrides directive 5: the math path honours the same resource, recursion and iteration caps as every
      other path, and may not panic or allocate without bound.
    - Each such case is a `MATH-OVER-EXCEL` entry in docs/excel-semantics/ with its evidence and a bounded region, and
      is listed here. A new case needs a decision record **carrying an owner ruling** (the owner sets directive 1;
      decision 0052's last consequence), with the eval evidence, the region and Excel's and the math's values; no
      agent, steward included, adds a case on its own.
    - An engine change that follows the math in a region lands only after the region's evidence points and bounds are
      committed (T-3358, T-3364, T-3367).
    - Initial cases (statistics.md and financial.md, "MATH-OVER-EXCEL" entries):
      1. **Tiny-p quantiles**, per function and parameter: T.INV(p, df) at p < 1e-150 (left tail); F.INV(p, d1, d2) at
         p < 1e-150 (left tail); BETA.INV(p, alpha, beta) at p < 1e-150 (left tail); CHISQ.INV.RT(p, df) at
         p < 1e-150 (the right tail). The parameter bounds (df; d1 and d2; alpha and beta) are not yet evidenced: the
         assessment's "df ≥ 1e6" does not hold for its own example, `T.INV(1.87E-297,3937)` (Excel −156.96, math
         −40.26, df 3937). Each function's exact region is fixed from T-3358's R3-marked points and amended into the
         entry; until then the case covers only the cited point.
      2. **NPER** at 0 < rate < 1e-14 (Excel up to 4% off), excluding rates where 1 + rate == 1, which are `#DIV/0!` in
         Excel (financial.md, "NPER divides by ln(1+rate)") and stay so. Negative tiny rates are not covered.
      3. **GAMMA.INV at a huge shape, the Wilson–Hilferty closed-form branch only**: where the gamma density at
         Excel's start x0 is below the smallest normal double, Excel returns x0, about 1.000266 times the true quantile
         (statistics.md "GAMMA.INV and CHISQ.INV at a huge shape"; probes/t3070-gamma-inv-band A110,
         probes/t2982-gamma-inv-huge-shape A7–A23). The same branch serves GAMMAINV, CHISQ.INV, CHISQ.INV.RT and CHIINV at
         shape df/2 (t3070 A304–A327, t2982 A92, A93), so they are in the case too. Its `#NUM!` and true-quantile
         branches still match Excel.
    - Excel's merely imprecise results are still matched: BESSELJ (about 1e-7), XIRR (about 1e-8), CHISQ.INV.RT
      (about 5e-9), and every other case below G.
    - The other gaps the assessment names (RATE with 1–2 periods, the WEIBULL.DIST pdf underflow, DURATION negative for
      very short bonds under basis 2 and 3) are not R3 cases: Excel's value stays the target until a decision record
      lists one with its evidence.
    - Excel's errors are still matched: R3 never applies to an error against a number.
    - A cell of an eval suite inside a listed region where the engine returns the math (within ε) stays an F line of
      eval/baseline and the CI gate is unchanged; Gate B and R1(c) count it as passing, the way decision 0134 leaves out
      a listed cell (the harness task below marks such cells).
  - **R4. Anti-overfitting guards** (process, binding on all numeric-fit work, on top of decision 0056):
    1. The fit lab has a mandatory hold-out split: a deterministic ~30% split by case or cell hash. Train and test
       counts are reported side by side. A candidate is accepted only if its test exact rate is within a stated margin
       of its train rate and it beats the correctly rounded candidate on test.
    2. After a model is frozen, a fresh log-uniform sweep over the full domain is filled and its result recorded in the
       task before merge.
    3. Every fitted constant, cut-point or table carries a `FITTED(T-XXXX, n_cells, range)` doc tag, and their count is
       ratcheted. A fitted piece is not applied outside the range it was measured on; outside it, the code falls back to
       accurate math.
    4. No fitting below the oracle's reproducibility floor (decision 0112; web against desktop up to 4 ulp,
       docs/research/oracle-calibration.md).
    5. The reviewer-excel-fidelity checklist requires the hold-out counts and the Gate A envelope result for any fit.
  - **R5. Phase 1b deferral.** Last-digit (bit-reproduction) work on the BETA.DIST and F.DIST densities, and on other
    dense-sweep cells already within Gate A, moves to Phase 1b. Todo tasks whose cells are only such cells carry
    `defer: phase-1b`. In-progress and blocked tasks keep their status and get a note that their remaining cells are
    Phase 1b under this record. Already-open, ready-to-merge PRs may still merge.
  - **R6.** IMTAN (ironcalc-calc/ENGINEERING/COMPLEXs.xlsx Sheet1!N6, T-2774) and IMSQRT (COMPLEXs.xlsx Sheet1!AL12 and
    AL26, T-2536) follow the Excel-for-the-web value under decision 0039 item 4. This record is the decision 0039 item 4
    requires to move a cell that passes against desktop evidence to the web value: the PRs of T-2774 and T-2536 may
    regress exactly these three cells. Their open question, the Mac 16.0300 cache, stays T-1075 items 16 and 17 (M14),
    so the three cells are lines of eval/milestones/exit-exceptions.tsv (M9) and eval/milestones/phase1-exceptions.tsv,
    added in this PR ahead of the fixes (they read as stale until T-2774 and T-2536 land; keep them). This answers
    T-3006.
  - **R7.** Decision 0143 (cells owned only by open M14 tasks leave the Phase 1 count) extends to the milestone probe
    rows: an M14-only probe cell, such as the 41 M9 FORECAST.ETS cells, never blocks a milestone exit.
- **Consequences:**
  - CLAUDE.md directive 1, decision 0052, docs/milestones.md (Phase 1) and docs/excel-semantics/README.md (the
    `MATH-OVER-EXCEL` status) change in this PR.
  - Tasks filed by T-3346 (its Outcome has the table): eight P0 `accuracy` bug tasks (IRR and RATE first); the Gate A
    spec and its implementation slices; the Gate B and R1(c) report measures with R7; the fit-lab hold-out split; the
    `FITTED(...)` tag and ratchet; the reviewer and skill text; the generalization fixes (Excel's t, F and chi-squared
    inverse; accurate math outside the measured range of extrapolated fits; re-validating tuned selectors; POISSON.DIST
    and BINOM.DIST); NPER below a rate of 1e-14 (R3). Existing tasks were retagged (the probe-row and text owners to
    `semantics`), deferred (R5) or given notes, as T-3346's Outcome lists.
  - Until the harness tasks land, the report's Phase 1 line and exit rows read as before; nothing counts as met that
    did not before. No gate, baseline or ratchet changes in this PR, and the CI gate still forbids any bit regression.
  - The fit-lab sweeps keep running and their counts stay in the report: they are Phase 1b's measure.
  - R4.5 and R3's acceptance rule are in reviewer-excel-fidelity (checks 6 and 7), the work-loop, steward and
    gap-triager text, and docs/process/quality-gates.md (T-3362). Until T-3356, T-3357, T-3358, T-3360 and T-3361 land,
    that text asks for a manual hold-out split, a probes-numeric fresh sweep and mpmath values attached in the task.
