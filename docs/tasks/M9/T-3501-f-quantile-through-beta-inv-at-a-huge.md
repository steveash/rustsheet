---
id: T-3501
title: "F quantile through BETA.INV at a huge df: F.INV.RT(0.5,2,1.2E8) is 4E-9 off the math and 1E-8 off Excel, outside Gate A (gen-m9 f34f66fb7f81e673 A209)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3516]
campaign: visible
---

## Goal
F.INV, F.INV.RT and FINV at a df of 1E5 or more follow Excel's mapping from its BETA.INV at every df, odd and
fractional ones included, so gen-m9 f34f66fb7f81e673 Calc!A209 is within 15 digits of Excel, and no point of T-3516's
sweep is outside Gate A's envelope.

## Context
- Gap triage 2026-10-31 (`make eval` on origin/main). Needs an oracle sweep: T-3516 (the cached t1538 evidence has
  round dfs only, and there the engine already matches).
- Cell list:
  - gen-m9/f34f66fb7f81e673.case.json (1): Calc!A209
- `F.INV.RT(Data!E7,2,(Data!$C9+Data!E2))` = `F.INV.RT(0.5,2,123456789.1229)`; the engine truncates the df to
  123456789 (so does Excel: the df argument is truncated). Values: Excel 0.6931471803615475, engine
  0.6931471872147749, 50-digit math 0.69314718445161477 (mpmath `d2/2*expm1(2*ln 2/d2)`). Excel is -5.9E-9 relative
  off the math, the engine +4.0E-9: neither within 15 digits of Excel nor within 1E-9 of the math, so the cell fails
  Gate A (decision 0188 R2) as well as the visible campaign.
- At the round df 1E8 the same call matches: `F.INV.RT(0.5,2,1E8)` is Excel 0.6931471826011306, engine
  0.6931471826011304 (probes-numeric/t1538-f-inv-huge-df A10). The engine's path is `fisher.rs` `f_quantile` ->
  `f_quantile_through_beta` (from `F_QUANTILE_STATRS_DF_MAX` = 1E5): F.INV.RT(p,2,d2) = 1/F.INV(p,d2,2), with
  y = `beta_lower_crossing(d2/2, 1, p)`. At shape b = 1 the crossing is near 1 and y/(1 - y) keeps only 1 - y's place on
  the 2^-53 grid (step about 7E-9 relative in the result here), so no y-based value can be within 1E-9 of the math
  unless it lands on the nearest grid point. statistics.md "BETA.INV at a huge shape is AS 109's Newton search
  (T-1538)" already records this cell as an exception: y = BETA.INV(0.5,61728394.5,1) is one double from the nearest
  root, which lies almost halfway (0.50/0.50 ulp) between two doubles, "a tie Excel's own rounding decides,
  UNVERIFIED (T-2555)". The engine picks the other side of the tie. A scratch evaluation of y = p^(1/a) and its grid
  neighbours through the engine's mapping gives 0.69314718105, 0.69314718790 and 0.69314717420, none of them Excel's
  value bit for bit, so the mapping's rounding (statistics.md: "not pinned" to about 10 ulps) is open too. Half-integer
  shapes (odd dfs) are the untested axis: at the even df 1E8 the engine matches.
- What the task decides: (a) a rule for Excel's side of such ties and its mapping, from T-3516's points, or, if none
  reproduces them, (b) the R4.3 fallback, accurate math, which is inside Gate A's envelope but stays visibly off Excel
  here; record which in statistics.md (the exception's UNVERIFIED status) and drop T-2555 from that line if this task
  settles it.
- statistics.md "F quantile at a huge df goes through Excel's BETA.INV (T-1538)": which of the reciprocal paths
  Excel takes is "not pinned", and the mapping's rounding is "not pinned" to within about 10 ulps.
- Decision 0188 R4.3: a piece measured at round dfs and applied between them is an extrapolation; if no model fits
  T-3516's sweep, the fallback outside the measured range is accurate math (the `s`-form 1 - y from
  `-expm1(ln p / a)` at b = 1 gives 0.6931471844516146 here, inside the envelope).
- Related: T-3398 (BETA.INV and F quantiles on the ported CDF, in flight), T-3363 (Excel's t, F and chi-squared
  inverse), T-3364 (extrapolated fits fall back to accurate math), T-2555 (BETA.INV early Newton stop at a huge
  shape). None lists this cell.

## Visible-parity stage
`visible`: Excel is itself 6E-9 off the math here, so the target is Excel's value (directive 1; R3's 1E-4 bar is not
reached). Target: Calc!A209 within half a unit of Excel's 15th significant digit, and every T-3516 point inside
Gate A's envelope.

## Acceptance criteria
- [ ] gen-m9/f34f66fb7f81e673 Calc!A209 is within 15 significant digits of Excel; its line leaves
      eval/baseline/gen-m9.txt if it becomes bit-exact, and no new known-failure line appears in any gated suite.
- [ ] Every point of T-3516's two cases is inside Gate A's envelope (within 15 digits of Excel, or within 1E-9
      relative of 50-digit math); the Outcome lists the counts and any point outside.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188): hold-out counts for the chosen model and the
      correctly rounded candidate side by side, trained on `t3516-f-inv-huge-df-sweep` only and tested on
      `t3516-f-inv-huge-df-holdout` (never tuned on), the margin stated before scoring; a fresh log-uniform sweep filled
      after the model is frozen, with its result in the Outcome; the Gate A envelope result on that fresh sweep; each
      fitted constant, cut-point or band tagged `FITTED(T-3501, n_cells, range)` and applied only inside its range,
      with accurate math outside it; no step fits differences under the oracle's reproducibility floor (4 ulp, web
      against desktop); no keep-matched branch (no band defined by the cells that already match, decision 0056).
- [ ] statistics.md "F quantile at a huge df goes through Excel's BETA.INV (T-1538)" records the rule found (or the
      fallback) with the T-3516 cells that pin it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- F quantiles below a df of 1E5 (T-3398, T-2500).
- The tiny-p quantiles (decision 0188 R3 case 1).
