---
id: T-2924
title: "Fit Excel's betaln for 1 < min(a, b) < 8 < max(a, b): a stored double -3 to +2 ulps off the accurate ln B (probes/t2829-ftest-ln-beta-grid)"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-2829]
campaign: visible
---

## Goal
A form for Excel's ln B(a, b) with both shapes above 1 and one above 8 (F.TEST's and BETA.DIST's bpser) that
reproduces the integer-ulp map of probes/t2829-ftest-ln-beta-grid on at least 90 of its 99 cells, or the forms tried
recorded in docs/excel-semantics/statistics.md.

## Context
T-2829 read Excel's L = ln B off 99 F.TEST cells deep in bpser (x near 1E-12, z unstored), a from 2.5 to 8 by b from
8.5 to 19.5: `python docs/research/t2829/ln_beta_map.py` (output in `ln_beta_map.out`). Every cell's L is a stored double
an integer k ulps from fl(accurate ln B), k from -3 (a = 6, b = 11.5, gen-m9/b6593d7dd13f2dac A306) to +2; fractions of
L + ln a, L + ln b or L + lnGamma(a + b) are random, so the stored quantity is L itself. Scores of the forms tried (cells
reproduced of 99): accurate 45 (what `f_test_tail::neg_ln_beta` ships above 8), TOMS betaln with Excel's LN 49, the TOMS
reduction with w = prod a/(a + b) 55, gamln(a) + algdiv(a, b) unreduced 40, Excel GAMMALN sums in each order 15-29.
|k| <= 1 at a = 2.5, 4, 4.5 and 5, up to 2 at a = 3, 3.5, 6, 7 and 8, and 3 only at (6, 11.5); for integer a (common
a0 = 1 after the reduction) it is not constant in b, so the error is not algdiv(a0, b) alone. The same L is open for BETA.DIST (statistics.md, "algdiv
OPEN": Excel's algdiv about 1 unit from TOMS's), so a fit should be checked on BETA.DIST's cells too.
Ideas: the reduction product in x87 or in another order; lnw from a product of quotients h/(1 + h) with LN of each
factor; Stirling's series for lnGamma(a + b) and lnGamma(b) with Excel's own constants; a probe at a = 1.5 .. 2 to pin the
unreduced pieces (k at a = 2.5 is -1 to +1).

## Acceptance criteria
- [ ] A candidate scored on all 99 grid cells and on BETA.DIST's bpser cells with a shape above 8 (`uv run rustsheet-fitlab`
      or the script above); the best form and its score in statistics.md.
- [ ] If a form reproduces at least 90 grid cells without losing a bit-exact F.TEST or BETA.DIST cell, a follow-up task to
      port it to `f_test_tail.rs` / `bratio`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Porting the form (its own task), the shapes of 1/2 and 1 (the remainder model), min(a, b) < 1.
