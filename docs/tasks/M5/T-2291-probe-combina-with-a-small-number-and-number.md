---
id: T-2291
title: "Probe COMBINA with a small number and number_chosen past 2^31 (the n+k-1 bound is extrapolated there)"
milestone: M5
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's COMBINA for a small `number` with `number_chosen` at or past 2^31 is known from oracle cells, so the n+k-1 bound of T-2162 is confirmed or corrected there.

## Context
- T-2162 made COMBINA `#NUM!` when `n+k-1` >= 2^31-1 (`COMBIN_MAX_NUMBER`, functions/math_and_trigonometry/mathematical.rs).
  The probe behind it (probes/t2199-text-length-and-combin-cuts) only has cells with `n` near 2^31. For a small `n` and a
  huge `k` the rule is extrapolated: `COMBINA(1,1E10)` is now `#NUM!` here (it used to be 1). Found by the T-2162
  Excel-fidelity review.
- Probe with the `excel-probe` skill: `COMBINA(1,2^31-1)`, `COMBINA(1,2^31)`, `COMBINA(2,2^31-2)`, `COMBINA(2,2^31-1)`,
  `COMBINA(1,1E10)`, `COMBINA(2,1E300)` and `COMBIN(2^31-2,2^31-2)`.

## Acceptance criteria
- [ ] Those cells are in a probes case, oracle-filled (excel-graph).
- [ ] numbers.md "COMBINA is COMBIN(n+k-1, k)" no longer says extrapolated; a fix task is filed if ours differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing COMBIN or COMBINA (the fix task's).

## Outcome
Probe case `probes/t2291-combina-small-n-huge-k` oracle-filled (excel-graph): all six COMBINA cells are `#NUM!` and
`COMBIN(2^31-2,2^31-2)` is 1, so the n+k-1 bound of T-2162 is confirmed for small `n`; no fix task needed.
numbers.md updated. Eval delta: none (probe cases only).
Follow-up: T-2517 (probe just under the cut).
