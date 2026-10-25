---
id: T-2507
title: "Beta density for min(a,b) > 1: Loader's saddle point with Excel's bd0 (T-2464 E5)"
milestone: M9
lane: functions-stat
status: blocked
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-2645]
---

## Goal
BETA.DIST(x,a,b,FALSE) with both shapes > 1 follows Excel's Loader (2000) saddle-point structure,
(a+b-1)·dbinom_raw(a-1, a+b-2, x, fl(1-x)), with Excel's bd0. The same function is the brcomp prefactor of bfrac
(T-2508).

## Context
- T-2464 research, step E5 of docs/research/t2464/r2-impact/RESULTS.md section 4.
- The rule is docs/excel-semantics/statistics.md "Beta density for min(a,b) > 1 is Loader's saddle point (T-2464,
  T-2513)":
  - q = fl(1 - x), k = a - 1, n = fl(a+b) - 2 and nk = fl(n - k), all stored (R's form; the single-ulp jumps of up to
    ±20 units are ε = n - k - nk), and M = fl(n·x) and fl(n·q);
  - bd0's log form when |k - M| >= 0.2·(k + M): t = fl(k·LN(fl(k/M))), then t + (M - k), M - k stored (it matters
    beyond ratio 2); each bd0 probably returned as a double (M4);
  - below that cut a series with v = fl(d/(k + M)) stored and a +δ(fl(2kv)) term (r3-loader-bd0's `ph` knob); its
    arithmetic is not identified;
  - E = LN(n + 1) + stirlerr terms - lf/2 - bd0_p - bd0_q, held extended into Excel's EXP (T-2645's EXP of an extended
    argument), never stored; lf = ln 2π + LN(k) + LN(nk) - LN(n) with Excel's LN; stirlerr accurate (a table plus a
    Stirling series), not GAMMALN-based.
- Models (T-2513 round 3, verifier-confirmed): `docs/research/t2464/r3-loader-const/m1_model.py` M3 (the structure
  above) and M4 (bd0 stored); r3-loader-bd0's `r3lib.py` for bd0's knobs. Scores, exact / 4 ulp / 15 digits:
  - r2-density-large's 32,264 cells: M3 6,312/25,807/31,461, r2's LDe 5,765/25,254/31,385, CR 3,573/26,024/29,280;
  - hold-out t2513-r3-loader-const-hold1 (3,296, log form): M3 952/2,990/3,176, M4 960/2,996/3,172, LDe 685/2,675/3,029,
    CR 251/1,738/2,435;
  - verifier hold-out t2513-verify-r3-loader-const-hold (1,600): M3 133/1,035/1,351, CR 66/533/909.
- Still PARTIAL: the constant carries an unexplained residual of 2-4 units sd (-10 to -14 at n 70-105), and the series
  branch 3-41 units of noise, so bit-exactness stays near 20-30%. Both are T-2586's.
- BINOM.DIST's interior pmf shares this dbinom_raw (and NEGBINOM.DIST's pmf the one shifted to (r-1, f+r-1)):
  statistics.md "BINOM.DIST pmf ..." and "NEGBINOM.DIST pmf ..." (T-2513). Expose it so T-2146 can call it.
- The status is PARTIAL:
  - The per-shape constant is unexplained (round 2: -8 to +8 units; round 3 cut the residual to 2-4 units sd), so the
    models lose within-4-ulp cells to CR on r2's data (M3 25,807 against 26,024).
  - Round 2's hold-out probe did not test bd0 (verifier); round 3's do.
  - On pre-T2464 cells LDe is no better than CR: it fixes 149 and breaks 17, against 148 and 4 (r2-impact
    `pdf_large.out`).
  - The engine's powf × tgamma is closer to Excel's last bits on w2f's max < 8 runs: 1,222 bit-exact, against LDe's
    719.
- So this task is a visible-parity step, not a bit-exactness one. Ship the structure if it wins on visible cells with
  no gated loss. Otherwise record the comparison and leave the density to T-2586.
- r2-impact's expected change: -148 visibly-off pre-T2464 cells (it breaks 25 there) and -5,007 in the w2f probes.
- Code: `incomplete_beta::beta_density` (and statrs above a + b = 171.6). Exact stirlerr can come from Excel-independent
  code (a table plus a Stirling series); it must be accurate to well under an ulp of the result.

## Acceptance criteria
- [ ] On the 32,264 a, b > 1 density cells of r2-density-large's data, the engine is within 15 digits on at least
      31,385 and bit-exact on at least 5,765 (M3 reaches 31,461 and 6,312).
- [x] On t2513-r3-loader-const-hold1 it is within 15 digits on at least 3,172 of 3,296 and bit-exact on at least 952.
- [x] The Outcome reports, per function and suite, the visible change on pre-T2464 cells against today's engine.
- [ ] No gated known-failure line is added.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pinning the per-shape constant or the series branch (T-2586).
- The T density (T-2585), which reuses this task's bd0 and stirlerr.

## Visible-parity stage
Decision 0083: this task is done when every BETA.DIST(.,FALSE) cell with min(a,b) > 1 that it owns is within half a unit
of the 15th significant digit of Excel's value, or the cells still off are named in the Outcome and handed to T-2586.
r2-impact counts 261 visibly off pre-T2464 (224 at max >= 8, 37 at max < 8). Bit-level criteria move to a follow-up
task without the campaign key.

## Blocked because
Blocked because: the model cannot meet the no-new-visibly-off and no-gated-regression rules. On the branch
`claude/hopeful-meitner-5oxf0g-t2507` the structure is implemented and wired into BETA.DIST's pdf for min(a,b) > 1.
It fixes 36,410 visibly-off probes-numeric cells. It also makes 677 cells visibly off, and Excel does not agree
with them (`docs/research/t2507/newly_visibly_off.tsv`). It adds one gated known failure: `BETADIST_BETAINV.xlsx`
G3, 1 ulp. Both come from the model's residuals (the shape constant and the series branch, T-2586). No regime split
by branch pair or by n avoids both (every branch-pair and n bin with a visible gain has visible losses, and the one
bin without them holds G3). As the Context section says ("Otherwise record the comparison and leave the density to
T-2586"), the change waits until T-2586 pins the residual, or until the owner accepts the losses with a decision
record. Unblock when either holds.

## Outcome
Not merged (see "Blocked because"); the comparison is recorded here and in statistics.md.

- **Code (branch only):**
  - `loader.rs`: `excel_beta_density`, `excel_dbinom_exponent` (exposed for T-2146), `excel_bd0` (stored x87 ops,
    the 0.2 cut, M - k stored, the series with v stored and the +δ(fl(2kv)) term) and `stirlerr_exact` (a
    double-double recurrence plus a 12-term Stirling series, about 1e-25 absolute).
  - `x87_log10.rs`: `ln_double_double`, factored out of FYL2X's kernel.
  - `x87_register.rs`: `from_double_double`, `to_double_double`, `neg`, `is_negative`, `same_value`.
  - `beta.rs`: BETA.DIST pdf with min(a,b) > 1 takes the model; other shapes and the F density are unchanged.
  - Every cell of a 300-cell hold1 sample matches an mpmath re-implementation of the rule.
- **Acceptance numbers** (exact / 4 ulp / 15 digits):
  - hold1: 967 / 3,050 / 3,193 of 3,296, against main's 105 / 931 / 1,460 and M3's 952 / 2,990 / 3,176.
  - r2-density-large: w1's probe1 sidecar is not in the repository, so 29,088 of the 32,264 cells are scored:
    5,904 / 23,850 / 28,779, against main's 2,808 / 16,050 / 20,480. The 5,765 bit-exact bar is met. The 31,385
    fifteen-digit bar needs 2,606 of the 3,176 missing cells.
  - Every probes-numeric BETA.DIST(.,FALSE) cell with literal a, b > 1 (101,556): 22,109 / 83,103 / 94,285,
    against 6,783 / 43,908 / 60,316.
- **Main-vs-HEAD comparison, every suite** (origin/main 77ea05010 against the branch; `probes/union-walk-repeated-in-lambda`
  timed out on main and is excluded):
  - fixed (now bit-exact): 20,728;
  - lost (bit-exact before): 4,948, all numeric. By ulp: 1: 2,034, 2: 750, 3: 1,010, 4: 707, 5: 345, 6-24: 102.
    Every cell is listed with its ulp in `docs/research/t2507/lost_cells.tsv`. One of them is gated (G3).
  - newly visibly off: 677, median 9 ulp, max 84 (`newly_visibly_off.tsv`). Most are large shapes
    ((100.5,157.5) 63, (60.5,67.5) 49, (40.25,89.75) 45, (25,25) 37), but small ones occur too ((2,10) 33).
  - visibly fixed: 36,410; changed but still failing: 79,538; type or error changes: 0.
  - gated: ironcalc-calc fixes `BETADIST_BETAINV.xlsx` G43 and G45 and regresses G3. gen-m9, probes and the other
    gated suites are unchanged.
- **Pre-T2464 cells, BETA.DIST only** (visibly fixed / newly visibly off):
  - probes-numeric: m9-beta-dist-a2 1 / 0, -a2p5 2 / 0, -a5 2 / 1, -a10 2 / 1, -a50 4 / 0, -a100 4 / 1;
    t0969-beta-density-b 132 / 0, t0969-beta-prefactor 27 / 8, t0969-beta-switch 41 / 1. In all, 215 / 12.
  - ironcalc-calc: 0 / 0 (bit-exact +2 / -1).
  - gen-m9 and probes: no change.
- **Findings** (statistics.md):
  - ln 2π must be the correctly rounded 1.8378770664093456. One ulp lower costs 174 hold1 exact cells.
  - Each of r3's bd0 knobs helps on the full probe set.
  - G3 is a 1-ulp residual of the constant at (2,3).
