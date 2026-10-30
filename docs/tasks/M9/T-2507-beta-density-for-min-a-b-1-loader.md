---
id: T-2507
title: "Beta density for min(a,b) > 1: Loader's saddle point with Excel's bd0 (T-2464 E5)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
campaign: visible
depends_on: [T-2645]
spec: docs/specs/SPEC-0025-incomplete-beta-bratio-port.md
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

### Cases owned (T-2981)
Routed by T-2981 (eval of 2026-10-28 at `main` 9586958b): BETA.DIST(x,a,b,FALSE) cells with min(a,b) > 1, the Loader branch this task ships (the min(a,b) <= 1 cells of the same cases are T-3035's). 43,401 visibly-off cells in 22 cases, per case:
- probes-numeric/m9-beta-dist-a10: BETA.DIST density, min(a,b) > 1 5
- probes-numeric/m9-beta-dist-a100: BETA.DIST density, min(a,b) > 1 13
- probes-numeric/m9-beta-dist-a2: BETA.DIST density, min(a,b) > 1 4
- probes-numeric/m9-beta-dist-a2p5: BETA.DIST density, min(a,b) > 1 5
- probes-numeric/m9-beta-dist-a5: BETA.DIST density, min(a,b) > 1 4
- probes-numeric/m9-beta-dist-a50: BETA.DIST density, min(a,b) > 1 9
- probes-numeric/t2464-r2-density-large-p1: BETA.DIST density, min(a,b) > 1 2,889
- probes-numeric/t2464-r2-log1p-hix: BETA.DIST density, min(a,b) > 1 5,264
- probes-numeric/t2464-w2f-absweep: BETA.DIST density, min(a,b) > 1 1,345
- probes-numeric/t2464-w2f-xruns-2: BETA.DIST density, min(a,b) > 1 2,048
- probes-numeric/t2513-r3-binom-pmf-a: BETA.DIST density, min(a,b) > 1 1,945
- probes-numeric/t2513-r3-binom-pmf-hold: BETA.DIST density, min(a,b) > 1 1,608
- probes-numeric/t2513-r3-fdensity-a: BETA.DIST density, min(a,b) > 1 978
- probes-numeric/t2513-r3-loader-bd0-a: BETA.DIST density, min(a,b) > 1 7,749
- probes-numeric/t2513-r3-loader-bd0-b: BETA.DIST density, min(a,b) > 1 113
- probes-numeric/t2513-r3-loader-bd0-hold: BETA.DIST density, min(a,b) > 1 4,633
- probes-numeric/t2513-r3-loader-const-diverse1: BETA.DIST density, min(a,b) > 1 5,630
- probes-numeric/t2513-r3-loader-const-grid1: BETA.DIST density, min(a,b) > 1 1,275
- probes-numeric/t2513-r3-loader-const-sweep1: BETA.DIST density, min(a,b) > 1 5,918
- probes-numeric/t2513-verify-r3-binom-pmf-h: BETA.DIST density, min(a,b) > 1 73
- probes-numeric/t2513-verify-r3-loader-bd0-hold: BETA.DIST density, min(a,b) > 1 950
- probes-numeric/t2513-verify-r3-loader-const-hold: BETA.DIST density, min(a,b) > 1 943

## Acceptance criteria
- [ ] On the 32,264 a, b > 1 density cells of r2-density-large's data, the engine is within 15 digits on at least
      31,385 and bit-exact on at least 5,765 (M3 reaches 31,461 and 6,312).
- [ ] On t2513-r3-loader-const-hold1 it is within 15 digits on at least 3,172 of 3,296 and bit-exact on at least 952.
- [ ] The Outcome reports, per function and suite, the visible change on pre-T2464 cells against today's engine.
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

## Stage (decision 0114)
`campaign: visible`, not `accuracy` (re-staged 2026-10-27 by this task's second attempt). Decision 0114's accuracy
stage needs a high-precision reference to show Excel close to correctly rounded, and for min(a,b) > 1 it does not:
- Against a 40-digit mpmath reference of x^(a-1) (1-x)^(b-1) / B(a,b) (exact 1 - x), a correctly rounded kernel
  scores, exact / 4 ulp / 15 digits shown: hold1 251 / 1,738 / 2,147 of 3,296 (statistics.md's CR row, 2,435 by its
  half-unit rule), t2464-w2f-absweep 327 / 1,997 / 2,259 of 3,072, t2464-w2f-xruns-1 299 / 4,611 / 5,013 of 6,144
  (`docs/research/t2507/cr_score.py`), and, from the separate 60-digit run, the 928 pre-T2464 cells of
  `docs/research/t2464/beta_exact.tsv` 123 / 744 / 737. Excel's error on those 928 reaches
  235 ulp (`BETA.DIST(0.9,5,100,FALSE)`, m9-beta-dist-a5) and is 24-182 ulp at a dozen cells with shapes 10-100
  (m9-beta-dist-a10, -a50, -a100), from `beta_exact.tsv`'s err_ulp column.
- Loader's model (M3) reaches 3,176 of hold1 within 15 digits. So matching Excel's visible digits means
  reproducing its algorithm, which is the visible stage (decision 0114, decision 0056's time-box).

## Outcome
- Landed 2026-10-30 (owner-run session https://claude.ai/code/session_01Xbmds2YtTrK9a9T4mMitSB) under decision 0181,
  ported from `claude/hopeful-meitner-5oxf0g-t2507` (090e13ec1, 9d9fd0a0b) onto `main` ec865788:
  - `loader.rs` keeps only Excel's pieces (`excel_beta_density`, `excel_dbinom_exponent`, `excel_bd0`,
    `stirlerr_exact`); the R-style `stirlerr`, `bd0` and `dbinom_raw_dev` left with T-3034. Its logarithm of a
    double-double is `x87_log10::ln_double_double_pair`. `x87_register::Register` gains `from_double_double`,
    `to_double_double` and `neg`.
  - `beta.rs`: the density is T-2503's small-shape form, then the Loader density for a, b > 1, then today's forms.
- Against a `main` build over gen-m9, ironcalc-calc, probes and probes-numeric: visibly-off cells 82,100 -> 46,065
  (36,696 fixed, 661 new, all BETA.DIST densities in probes-numeric, docs/research/t2507/newly_visibly_off.tsv).
  Gated: ironcalc-calc 2 lines fixed, 1 regressed (BETADIST_BETAINV.xlsx G3, 1 ulp); no other suite moves.
- Criteria: t2464-r2-density-large-p1 is within 15 digits on all 13,824 cells (3,874 bit-exact; the criterion's
  32,264-cell r2 set is the research data, not one case); t2513-r3-loader-const-hold1 on 4,021 of 4,114 cells (1,785
  bit-exact) against 3,172 and 952 asked on its 3,296 a, b > 1 cells. The gated regression is decision 0181's.
- Follow-up T-3220 owns the 661 cells and G3 (T-2586's constant and bd0 series pieces).
