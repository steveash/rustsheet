---
id: T-3175
title: "Re-probe T-3148's neighbouring-double cells with exact-piece literals: Excel cuts the probe's 17-digit p and x literals to 15 digits"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
An oracle-filled probe gives Excel's GAMMA.INV and CHISQ.INV.RT at the true neighbouring doubles of p = 0.999, and
Excel's GAMMA.DIST, CHISQ.DIST and CHISQ.DIST.RT at the true doubles q + k ulp beside each p = 0.999 quantile, so
T-3146 can fit Excel's landing point from cells that mean what they say.

## Context
- Gap triage 2026-10-29: probes-numeric/t3148-gamma-quantile-flat-band (T-3148, done) wrote its neighbouring doubles
  as 17-digit decimal literals. Excel cuts formula number literals to 15 significant digits (numbers.md; T-0143,
  T-0144; the 450-cell check in docs/research/numeric-reaudit-2026-10-05.md), and so does the engine. So A120 to A128
  (`0.99899999999999967`, `...78`, `...89`) all evaluate p = 0.998999999999999, A129 to A140 all evaluate p = 0.999,
  and the cdf series from A141 (`GAMMA.DIST(6.9077552789821359,1.0,1,TRUE)`, `...368`, `...35`, `...377`, ...) reads
  several k as one 15-digit x. Check: `GAMMA.INV(p,2,1)` is 9.233413476450405 for the first group and
  9.233413476451513 for the second, 1.108E-12 apart, which is 1E-15 of p over the density 9.04E-4.
- statistics.md's T-3148 bullet now carries this correction; T-3146 (the engine fit) is warned not to read A120 to A140
  as adjacent doubles.
- How: the exact-piece form the T-2586 probes use, a double written as `(a*2^-k+b*2^-m)` with a, b integers below
  2^27 (see probes-numeric/t2586-r5-f-final-hold A1: `=(116335370*2^-30+21204251*2^-56)`), so every operand is exact
  and the sum is the intended double. Keep T-3148's layout: the same shapes and p, the three doubles each side of
  0.999 for GAMMA.INV(p,2,1), GAMMA.INV(p,5,1) and CHISQ.INV.RT(p,5), and k in 0, +-1, +-2, +-4 ... +-128 for the
  four GAMMA.DIST and four CHISQ.DIST / CHISQ.DIST.RT series, plus a column echoing each argument (`=A1`) so the
  sidecar shows the double Excel used. Fill with excel-graph (`excel-probe` skill).

## Acceptance criteria
- [ ] A new case probes-numeric/t3175-gamma-quantile-flat-band-exact, every argument an exact-piece expression, with
      its oracle sidecar filled (excel-graph, no pending cells) and the echo column equal to the intended doubles.
- [ ] statistics.md's T-3148 bullet is rewritten from the new cells: whether Excel's quantile changes across adjacent
      p doubles, and the band edges of the cdf in true ulps of x, citing the new case's cells.
- [ ] T-3146's Context points at the new case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any engine change (T-3146 fits the rule; T-2638 and T-3147 own the cdf kernel).
- Removing or rewriting t3148's case: its cells stay valid evidence for the 15-digit arguments they really have.
