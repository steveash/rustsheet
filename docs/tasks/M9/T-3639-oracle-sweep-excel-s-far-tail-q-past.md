---
id: T-3639
title: "Oracle sweep: Excel's far-tail Q past x = 600 at shapes 1.5 to 20.5 and above shape 11 at x 30 to 600, with a hold-out, for T-3640"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
defer: phase-1b
---

## Goal
The fit task T-3640 has dense excel-graph evidence of Q(a, x) (CHISQ.DIST.RT, POISSON's cdf) where T-3269's far-tail
form is not applied today (shapes above `FAR_TAIL_MAX_SHAPE` = 11 at 30 < x <= 600, and every shape past
`FAR_TAIL_MAX_X` = 600), enough for a train set and a separate hold-out (decision 0188 R4.1), so no fit is attempted
blind.

## Context
- Gap triage 2026-11-01: T-3640 owns 37 cells of probes-numeric/t3269-fresh-sweep in those two regions, 7 to 955 ulp off
  Excel. Ours is the correctly rounded Q at most of them (within 2 ulp of 50-digit math at 30 of the 37); Excel is 7 to
  955 ulp off the math, so matching Excel is a fit of its formulation.
- Cached evidence: docs/research/t3269/holdout-split.tsv has 175 points above shape 11 at 30 < x <= 600, but only 12
  nonzero points past x = 600 (shapes 1.5 to 19.5, x 616 to 724), 7 of which are the t3269-fresh-sweep cells T-3640
  owns. `FAR_TAIL_MAX_X` is UNVERIFIED in `incomplete_gamma/excel_tail.rs` ("no cell past it says where Excel leaves
  the form"). Q stays a normal double up to about x = 700 + a ln x, so the band past 600 is short and must be dense.
- Deferred with T-3640 (decision 0188 R5): the sweep serves only Phase 1b work. The steward removes both keys together.
- Model: docs/research/t3269/fresh_sweep_2.py (case layout, seeded generator, `hypotheses` labels, 6-significant-digit x
  so 2x is exact in 15 digits; integer shapes alternate POISSON(a - 1, x, TRUE) and CHISQ.DIST.RT(2x, 2a)).

## Acceptance criteria
- [ ] A new probes-numeric case `eval/suites/probes-numeric/cases/t3639-far-tail-switch-sweep.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming a and x), built by a
      seeded generator `docs/research/t3639/sweep.py` and filled with
      `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill). Grid
      (about 400 cells): a at the multiples of 1/2 from 11.5 to 20.5, x log-uniform on (30, 600] (200 cells, 40 of
      them at x 30 to 45 next to the cut); a at the multiples of 1/2 from 1.5 to 20.5, x log-uniform on (600, 760]
      while Q stays normal (200 cells, including the last normal x per shape within 1%).
- [ ] A second case `t3639-far-tail-switch-sweep-hold.case.json` (200 cells, the same grid rule, a different seed and
      no shared point) is filled in the same PR for T-3640's hold-out; its description says it is never used for
      fitting.
- [ ] Both `.oracle.json` sidecars are committed with no pending cells, and their math sidecars are generated with
      `rustsheet-accuracy math --suite probes-numeric` (T-3380) and committed.
- [ ] statistics.md "Excel's far upper tail (T-3269)" cites both cases in its Evidence, with counts against today's
      engine (bit-exact, within 4 ulp, within 15 digits) and of Excel's cells correctly rounded, per region.
- [ ] probes-numeric is ungated, so the new cases add no eval/baseline/ line; `make check` green; reviewers approve.

## Out of scope
- Any engine change (T-3640). The subnormal end of the tail (T-3638). Shape 6 and x/a = 10 (T-3469).
