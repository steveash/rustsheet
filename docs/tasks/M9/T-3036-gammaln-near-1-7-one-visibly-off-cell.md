---
id: T-3036
title: "GAMMALN near 1.7: one visibly-off cell in the bpser sweep (probes-numeric t2464-r2-bpser-small-gl A3188)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
campaign: accuracy
depends_on: []
---

## Goal
`GAMMALN((56921738*2^-25+29850023*2^-51))` (x about 1.6964) agrees with Excel to 15 significant digits: Excel
-0.09655332490423914, ours -0.09655332490423907 (about 5 ulp, just past half a unit of the 15th digit). Or the
Outcome shows that no measured coefficient set for the (1.5, 2] rational reaches it without a gated loss, and the
cell goes to a `defer: phase-1b` task with T-2122.

## Context
- Filed by T-2981 (the T-0087 sweep of 2026-10-28 gave this cell to T-2981; eval of 2026-10-28 at `main` 9586958b).
  No open, non-deferred task owns GAMMALN's (1.5, 2] form: T-2123 (done) shipped T-1941's models, T-2122 (deferred,
  phase-1b) cracks Excel's own rational coefficients.
- The rule: docs/excel-semantics/statistics.md "GAMMALN is Excel's own x87 routine (T-1941, T-2128)", (1.5, 2]:
  fl(fl(x - 1) * fl(fl(x - 2) * fl(N/D))) with a (9,9) rational whose coefficient doubles are not known to the bit; the
  engine has the research's best set (`functions/statistical/gamma_kernels.rs`). That entry says the misses are "a few
  ulps, never visible at 15 digits". This cell is the exception: GAMMALN is small here (it crosses 0 at 2), so a
  5-ulp miss is visible. The statistics.md entry needs that correction whatever the outcome.
- Candidate sets: T-1941's research (`docs/research/t1941/`, branch `agent/research/T-1941-crack-gammaln-gamma-ln`,
  HANDOFF2 §3) scored more than one (1.5, 2] coefficient set; probe t2513-r3-m1-residual-gl has Excel's GAMMALN at 569
  shape arguments to score them on.

### Cases owned (T-2981)
- probes-numeric/t2464-r2-bpser-small-gl: GAMMALN 1 (Calc!A3188)

## Acceptance criteria
- [x] Calc!A3188 of probes-numeric/t2464-r2-bpser-small-gl is within 15 digits in `make eval`, with no gated loss
      (`make eval-check`) and no new visibly-off GAMMALN cell, or the Outcome scores the candidate sets and files the
      deferral.
- [x] statistics.md's GAMMALN entry corrects "never visible at 15 digits" with this cell.
- [x] `make check` green; reviewers approve.

## Out of scope
- Excel's exact coefficients (T-2122).

## Visible-parity stage
Target (decision 0114): the one cell above within 15 digits, or explained and deferred.

## Outcome
- Deferred, as the second branch of the first criterion allows. The cell is Excel's error, not ours: at x = 1.6964000000000001 the 40-digit
  lgamma (mpmath) is -0.0965533249042390811. The engine's -0.09655332490423907 is 1 ulp from it, and Excel's -0.09655332490423914 is 4.2 ulp
  from it. The gap is 5 ulp, 7e-17 against the 5e-17 half unit of the 15th digit.
- The four measured (1.5, 2] coefficient sets, scored at this cell with the stored-op chain `fl(fl(x-1)*fl(fl(x-2)*fl(N/D)))`:
  `xl_chain_d3ls` (the engine's) ...907, `xl_chain_pcD0` ...910, `xl_chain_80` ...910 and `xl_chain_L3b` ...911. None reaches ...914 (each
  stays within 15 digits of the math, not of Excel), so no swap can fix the cell without Excel's own coefficients. I did not score them on the
  rest of the eval, because the cell rules all four out. The cell goes to T-2122 (`defer: phase-1b`), in a "Cell added by T-3036" section.
- statistics.md's GAMMALN entry: "never visible at 15 digits" is corrected with this cell and the scores.
- No engine change; eval delta none. `make check` green on 6e09a09f; security and design approve.
