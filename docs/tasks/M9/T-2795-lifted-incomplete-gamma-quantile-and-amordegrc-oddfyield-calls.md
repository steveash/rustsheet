---
id: T-2795
title: "Lifted incomplete-gamma calls charged the cap's bound: #ERROR! where Excel spills (gen-m9, 4 cells; ODDFYIELD and AMORDEGRC split to T-2815)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
A lifted CHIDIST, CHISQ.INV.RT, GAMMADIST or POISSON (ODDFYIELD and AMORDEGRC: T-2815) over an array of a few hundred elements (a
PIVOTBY or FILTER result) returns its array, as Excel does (`#SPILL!` in these cells' layout), instead of `#ERROR!`
from the per-call lift budget, while each lifted call keeps a provable work bound (directive 5).

## Context
Split from T-2433, which charged HYPGEOM.DIST by its counts (`ValueWork::HypergeometricSum`, SPEC-0011 "Security rule
for kernel work") and fixed its cells. The rest are charged the bound of their kernel's cap per element
(lift/kinds/statistical.rs `INCOMPLETE_GAMMA_WEIGHT` 2^18, 512 elements; lift/kinds/financial.rs `AMORDEGRC_WEIGHT`
2^18; ODDFYIELD's `CouponWalks` charges a text date as the longest schedule), so a PIVOTBY of about 560 elements
passes the 2^27 cap.

Notes from T-2433 (data, check before relying on them):
- Incomplete gamma (statistical/incomplete_gamma.rs): the series (x < a + 1) has a provable bound from (a, x): once
  a + n >= 2x each term at most halves and no term exceeds the sum, so it ends within max(0, ceil(2x - a - 1)) + 55
  terms (at most a + 57); the small-shape Q series (a < 0.5, x < 1) within about 56. The continued fraction (x >= a + 1)
  has no closed-form bound: its measured terms grow with sqrt(a) (8,839 at a = 1E9). Either enforce a term budget
  computed from the shape (with a sweep test showing it is never reached where the 2^19 cap is not), or charge actual
  work after each element (a new `KernelWork` variant: a spec amendment to SPEC-0011 first). An inversion
  (CHISQ.INV.RT, GAMMA.INV) runs up to about 2,250 cdfs, so its bound is the cdf count times the per-cdf bound.
- An element with an error argument is refused before any cdf, so it can be charged 1 (as `ByValue` does): check
  whether that alone fixes ee2497a5e683c5d9 A199 (its beta is a SUMIFS over mismatched ranges).
- ODDFYIELD over `FILTER(..)-..`: the FILTER's texts are charged as the longest schedule (`CouponWalks`); a text
  that is not a date is `#VALUE!` before any walk.

Owned gen-m9 cells (from T-2433's triage list; Excel `#SPILL!` unless noted, ours `#ERROR!`):
- gen-m9/3e98b26cf6713e77 Calc!A204 `=CHISQ.INV.RT(_xlfn.PIVOTBY(Data!A1:D12,Data!B1:B12,Data!C1:E12,LAMBDA(acc,(-0.5))),10)`
- gen-m9/589f42e67f4b4b8d Calc!A121 `=KURT(POISSON((Data!D12*Data!B12),10000000001,_xlfn.PIVOTBY(..)),Data!E8:E12)`: Excel `#VALUE!`
- gen-m9/c2974c061384fdd9 Calc!A56 `=CHIDIST(_xlfn.PIVOTBY(Data!A1:D12,Data!B1:E12,Data!C1:E12,LAMBDA(acc,INT(acc))),5)`
- gen-m9/ee2497a5e683c5d9 Calc!A199 `=GAMMADIST(_xlfn.PIVOTBY(..),..,SUMIFS(..),..)`
- gen-m9/ba4ddb817a8e700e Calc!A323 `=ODDFYIELD(45535,45661,..,38502,(FILTER(Data!B1:C12,..)-(Data!E6/3.14159)),82,110,4)`
- gen-m9/e8d55ec393460bb7 Calc!A10 `=AMORDEGRC(56947,_xlfn.PIVOTBY(..),39763,716,2,PMT(100,271,417889,43,0),1.9)`

## Acceptance criteria
- [x] Visible parity (gen-m9): the 4 incomplete-gamma known-failure lines above (A56, A204, A199, A121) disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Each changed charge is a bound of its element's work (proved, or enforced by the kernel's own budget), measured
      with crates/rustsheet/benches/lift.rs at its new element cap, and the DEVIATION entry (statistics.md
      "Incomplete gamma: per-call cap and lifting") says how an element is charged (financial.md: T-2815).
- [x] `make check` green; reviewers approve.

## Out of scope
- ODDFYIELD and AMORDEGRC (ba4ddb817a8e700e A323, e8d55ec393460bb7 A10): split to T-2815.
- HYPGEOM.DIST (T-2433). The functions' values (their accuracy tasks).

## Visible-parity stage
Decision 0083: every cell this task owns has the wrong type or error, so the task is in the campaign.

## Outcome
The incomplete-gamma entries (CHISQ.*, CHIDIST, CHIINV, GAMMA.DIST, GAMMADIST, GAMMA.INV, GAMMAINV, POISSON,
POISSON.DIST) are charged by value (`ValueWork::IncompleteGamma`) instead of the flat 2^18 an element:
- A cdf of shape at most a takes at most `incomplete_gamma::cdf_terms_bound(a)` terms. The power series' a + 58 is
  proved, and the small-shape Q series adds 20. The continued fraction now has a budget of 256 + 32 * cbrt(a) terms,
  enforced by the kernel. A one-off sweep of shapes 1E-3 to 1.6E15 needed at most about 10 * cbrt(a) terms (110 below
  a = 10), and a unit test keeps the 2x headroom for shapes up to 1E10. No value changes.
- An element is charged in proportion to that bound at the measured 4.7 ns a term and 49 ns a cdf, at most 2^18.
- An element with an error argument, or an inversion whose probability is outside (0, 1), a logical or a blank, is
  charged 1, because every kernel reads its arguments first. A text shape or probability is charged 2^18.
- An inversion is charged 2,250 cdfs within its 2^19-term budget, so in practice it keeps the full weight.
- `CHIDIST(x,5)` costs 150 units, so a lift of it runs up to 894,784 elements. The new bench case
  "CHIDIST, df 5 at the continued fraction's slowest x" covers it.
Eval delta: gen-m9 -4 known failures, 0 regressions: c2974c061384fdd9 A56, 3e98b26cf6713e77 A204,
ee2497a5e683c5d9 A199 and 589f42e67f4b4b8d A121 now match Excel.
Follow-up: T-2815 (ODDFYIELD's off-calendar date and AMORDEGRC's text elements, 2 cells).
