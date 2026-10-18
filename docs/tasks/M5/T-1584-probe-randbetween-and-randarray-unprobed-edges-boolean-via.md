---
id: T-1584
title: "Probe RANDBETWEEN and RANDARRAY unprobed edges (boolean via reference, error order, huge span, one omitted bound)"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's behavior for the RANDBETWEEN and RANDARRAY edges the engine implements without oracle evidence is recorded, and the engine matches it.

## Context
T-1318 left these unprobed (docs/excel-semantics/numbers.md "RANDBETWEEN and RANDARRAY argument checks"): a boolean through a reference (`RANDBETWEEN(H1,2)`, H1 TRUE); error order `RANDARRAY(0,1,1.5,2.5,TRUE)`; `RANDBETWEEN(-1E308,1E308)` and `RANDARRAY(1,1,-1E308,1E308,TRUE)`; `RANDBETWEEN(,5)` and `RANDBETWEEN(5,)`. Add the cells to probes/volatile-invariant (excel-probe skill); code in functions/math_and_trigonometry/random.rs.

## Acceptance criteria
- [ ] The cells are in the case with oracle values, the numbers.md entry states the verified rule, and the engine passes them (baseline updated).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probed on excel-graph: boolean via reference `#VALUE!`, RANDARRAY error order, omitted `RANDBETWEEN` bound is `#N/A` (engine read 0), overflowing span `#NUM!` for RANDBETWEEN and a finite draw for RANDARRAY. New case `probes/randbetween-huge-span` (the 1E308 literal made excel-graph reject volatile-invariant). Engine fixed in `random.rs`; documented in `docs/excel-semantics/numbers.md`. Eval delta: no known failures added or removed (new cases pass; baseline ratcheted for the new coverage).
