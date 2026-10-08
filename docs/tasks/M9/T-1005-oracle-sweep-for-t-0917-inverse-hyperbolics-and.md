---
id: T-1005
title: "Oracle sweep for T-0917: inverse hyperbolics and SINH/TANH switchover thresholds"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
T-0917 can fit Excel's ASINH/ACOSH/ATANH/ACOTH/COTH/CSCH and SINH/TANH formulation against an oracle-filled sweep instead of the few values cached in Excel-authored workbooks.

## Context
Split from T-0942 (one sweep per M9 fitting task). Build a spec case `eval/suites/probes-numeric/cases/m9-hyperbolic.case.json` (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming the inputs, as in `m9-fin-power.case.json` from T-0942) and fill it with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric` (excel-probe skill).

Grid: ASINH/ACOSH/ATANH/ACOTH on log-spaced arguments (1e-10..1e10, and near 1 for ACOSH/ATANH/ACOTH: 1±1e-k), COTH/CSCH on 1e-10..710, and SINH/TANH on a fine grid around their switchover thresholds (bisect where results stop matching libm; T-0914 notes the fitted formulation).

## Acceptance criteria
- [ ] `m9-hyperbolic.case.json` and its `.oracle.json` sidecar are committed; no pending cells.
- [ ] Quirks visible in the sweep (for example where the engine already differs, `rustsheet-eval show probes-numeric/m9-hyperbolic.case.json`) are recorded in docs/excel-semantics/numbers.md with the case as evidence.
- [ ] T-0917 links the case and its `depends_on` no longer lists T-0942.
- [ ] Fixtures hold only formulas and Excel values; no RUSTSHEET_ORACLE_* values in logs or commits.
- [ ] If the oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or implementing the formulation (T-0917 does that).

## Outcome
Added `probes-numeric/m9-hyperbolic` (268 cells) with its excel-graph oracle sidecar (no pending cells). 110 of 268 cells differ from the engine (66 ulp, 31 close, 11 mismatch, 2 SINH(+-710) `#NUM!`); quirks recorded in numbers.md. T-0917 already depends on T-1005 and no longer on T-0942. Eval baseline unchanged (probe suite carries its own known failures; see make check).
