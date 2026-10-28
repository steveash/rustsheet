---
id: T-2645
title: "Excel log1p and EXP of an extended argument as engine primitives; BINOM.DIST(0,n,p) on them (T-2464 E0)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
The engine has the two elementary pieces that Excel's incomplete-beta and beta-density code build on, and
BINOM.DIST(0,n,p) uses them; BINOM.DIST(n,n,p,FALSE) takes its own closed form:
- Excel's log1p, for ln(1 + a) with -0.375 <= a < 0;
- Excel's EXP of an extended (64-bit, `x87_register::Register`) argument.

## Context
- T-2464 research, step E0 of docs/research/t2464/r2-impact/RESULTS.md section 4.
- docs/excel-semantics/statistics.md "Excel's log1p is a 14-term atanh series in alnrel's variable (T-2464)" gives
  the log1p:
  - an x87-stored atanh series in r = a/(a + 2), 14 forward terms with c_k = RN(1/(2k+1)), then 2·(r + s);
  - for x > 0.375 (a < -0.375) it is LN(fl(1 - x)).
  - Model: `docs/research/t2464/r2-log1p/excel_log1p.py` (`log1p_series`, `ln1m`). Pinned l per p:
    `r2-log1p/{pilot,runs,grid,hold,tiny}.pins.tsv`.
- BINOM.DIST(0,n,p,·) = EXP(fl(n·l)), with l from the log1p above for p <= 0.375 and LN(fl(1 - p)) above (w5, r2-log1p).
  It is BINOM's own shortcut, not the kernel's b0 = 1 closed form. It is #NUM! from n = 2^31.
- BINOM.DIST(n,n,p,FALSE) = EXP(fl(n·LN(p))), Loader's (2000) other end branch (T-2513, statistics.md "BINOM.DIST pmf:
  closed forms at k = 0 and k = n"): 172 of 172 cells.tsv cells (CR 70) and 320 of 320 on
  t2513-verify-r3-binom-pmf-h. The pmf at k = 0 is the same rule as above (173 of 173 pmf cells; verifier 318 of 320).
  The k = n rule is measured on the pmf only.
- EXP of an extended argument is needed by the M1 density (T-2503) and by bpser's z64 prefactor (T-2505).
  - The mpmath model of Excel's EXP V6 that accepts an extended input is `docs/research/t2464/w1-density/lib.py`
    (`exp_v6`).
  - The engine has `x87_exp::excel_exp` (double argument only), `x87_log10::excel_ln`, `arithmetic::extended_*` and
    `x87_register::Register`.
- r2-impact (`binom_k0.py` → `binom_k0.out`):
  - The rule is bit-exact on 345 of 345 BINOM.DIST(0,n,p) cells in docs/research/t2464/cells.tsv.
  - It fixes all 68 visibly-off pre-T2464 cells (cdf and pmf) and breaks none.
- Code: `functions/statistical/binom.rs` and `discrete.rs` (today a double-double pmf walk), `x87_exp.rs`.
- Code must not read docs/ (decision 0066). Copy the pinned rows the tests need into a fixture under the engine crate.

## Acceptance criteria
- [x] Pin BINOM.DIST(0,n,p)'s `#NUM!` limit in (2^30, 2^31] with a probe (statistics.md "BINOM.DIST(0,n,p) is #NUM!
      somewhere in (2^30, 2^31]") and match it.
- [x] `excel_log1p` matches r2-log1p's pinned l on at least 15,764 of the 15,791 pinned rows (pilot, runs, grid,
      hold), and every miss is exactly 1 ulp off (verify-r2-log1p/vscore.out).
- [x] For a in [-0.375, 0) it is bit-identical to `excel_log1p.log1p_series` on a committed fixture of at least 5,000
      points.
- [x] EXP of an extended argument matches w1's mpmath V6 model on at least 5,000 random extended inputs.
- [x] On every double input, EXP of an extended argument is bit-identical to `excel_exp`.
- [x] BINOM.DIST(0,n,p,TRUE|FALSE) and BINOMDIST(0,…) are EXP(fl(n·l)). That is 345 of 345 bit-exact on the cells.tsv
      rows, and the 68 visibly-off pre-T2464 cells are fixed, with no new known-failure lines.
- [x] BINOM.DIST(n,n,p,FALSE) and BINOMDIST(n,n,p,FALSE) are EXP(fl(n·LN(p))): 172 of 172 bit-exact on the cells.tsv
      rows, with no new known-failure lines.
- [x] statistics.md's log1p and BINOM pmf entries gain an "Implemented in" line.
- [x] `make check` green; reviewers approve.

## Out of scope
- The positive-argument alnrel that algdiv and bcorr use. It is not this series (r2-log1p `algdiv_try.py`); see T-2512.
- Wiring the log1p into the density (T-2503) or into bgrat (T-2509).

## Visible-parity stage
Decision 0083: this task is done when every cell it owns is within half a unit of the 15th significant digit of
Excel's value. It owns the BINOM.DIST(0,n,p) cells, which are 68 visibly off before it. Bit-level criteria still unmet
then move to a follow-up task without the campaign key.

## Outcome
- Probe `probes-numeric/t2645-binom-trials-limit` (46 cells, excel-graph): the limit is 2^31 - 1 trials, not 2^31.
  BINOM.DIST and BINOMDIST are `#NUM!` from trunc(n) = 2147483647 for every k, p (0 and 1 too) and cumulative flag,
  and so is BINOM.DIST.RANGE (probed only at number_s = 0, p = 1E-12); 2^31 - 2 evaluates; NEGBINOM.DIST has no such limit. Recorded in statistics.md "BINOM.DIST is
  #NUM! from 2^31 - 1 trials" and implemented (`binom.rs` `TRIALS_LIMIT`).
- `x87_log1p.rs`: `excel_log1p` (the stored 14-term atanh series) and `excel_ln_one_minus` (LN(fl(1 - x)) above
  0.375). It gives 15,764 of the 15,791 pinned rows, every miss 1 ulp off (ignored test `matches_every_pinned_row`,
  run with `RUSTSHEET_T2464_LOG1P=docs/research/t2464/r2-log1p`). It equals `excel_log1p.ln1m` on all 5,211 rows
  of `test/fixtures/log1p_kernel.fixture`, 5,065 of them with x <= 0.375; the 27 Excel misses are in the fixture.
- `x87_exp.rs`: `excel_exp_extended(Register)`. `excel_exp` is now that function applied to the loaded double, so the
  two agree on every double by construction. A test also checks full-width (64-bit) registers of doubles. It matches
  w1's mpmath `exp_v6` on 5,200 random extended arguments (`test/fixtures/exp_extended.fixture`). `Register::parts`
  is new (and a test-only `from_parts`).
- BINOM.DIST/BINOMDIST at k = 0 (pmf and cdf) = EXP(n·l), and the pmf at k = n = EXP(n·LN(p)). `make eval` against
  unmodified origin/main fixed 385 cells and added no failures: BINOM.DIST 373, BINOMDIST 4, BINOM.DIST.RANGE 2,
  plus 6 composite cells. That is 340 in t2145-binomial-sweep (1,124 -> 1,464 of 3,260), 32 in the new probe, 3 in
  ironcalc BINOM.xlsx and 8 in gen-m9. No k = 0 or k = n cell with 0 < p < 1 still fails, so all 345 k = 0 and all
  k = n pmf rows of cells.tsv are bit-exact. One standing failure moved but still fails: gen-m9/caa1eed48b1883fe
  A384, `BINOMDIST(-0.0001,9,0.999999,..)`. Excel gives `#NUM!` because it tests k < 0 before truncating; that cell
  belongs to T-2435.
- The ungated probes-numeric delta, from the fidelity reviewer's full comparison against main after the research PR
  landed its probes: 17,009 cells fixed, 0 newly failing. By case: t2464-r2-log1p-grid 5,424, -hold 5,384,
  -runs 5,306, -pilot 195, t2513-verify-r3-binom-pmf-h 360, t2145-binomial-sweep 340. 27 cells changed value and
  still fail; they are the documented 1-ulp log1p misses.
- Merged main's T-2502 (#1749) into `binom.rs`. Its k = n - 1 cdf and its NEGBINOM r = 1 / f = 0 closed forms
  (`closed_form_tails`, `k >= 1`) run after this task's k = 0 and k = n pmf branches. Both tasks' tests are kept.
- statistics.md (after merging the research PR #1741): the log1p and BINOM pmf entries now have "Implemented in"
  lines. The bracket entry "BINOM.DIST(0,n,p) is #NUM! somewhere in (2^30, 2^31]" is merged into "BINOM.DIST is
  #NUM! from 2^31 - 1 trials (T-2464, T-2645)" and superseded by it.
- Renumbered from T-2633 (main took that id), with the probe renamed to t2645-binom-trials-limit. After merging main
  (T-2119's BETA.DIST/BETA.INV change does not touch this code), `make check` passes locally: fmt, clippy, tests,
  eval-check with 0 regressions, evidence floor, python, hygiene and typos.
