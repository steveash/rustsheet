---
id: T-1816
title: "DEC2BIN and DELTA take an error in their second argument over text in the first"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
DEC2BIN and DELTA give the error of their second argument when the first is uncoercible text, as
Excel does: `INDEX(DEC2BIN({"x";9},NA()),1)` and `INDEX(DELTA({"x";1},NA()),1)` are `#N/A`.

## Context
Found by T-1383 (probes/eng-later-argument-lift M1-M8, excel-graph 2026-10-17). Inside one element
BESSELJ, BITLSHIFT, CONVERT, COMPLEX, IMPOWER and ERF report the first argument's error (`#VALUE!`,
IMPOWER `#NUM!`), but DEC2BIN and DELTA report the second argument's `#N/A` (M2, M8). Ours reads the
first argument first and gives `#VALUE!` for both; the scalar calls `DEC2BIN("x",NA())` and
`DELTA("x",NA())` presumably differ the same way (probe them). The kernels are
`dec2bin_of` (functions/engineering/number_basis.rs) and `compare_with_default`
(functions/engineering/misc.rs); GESTEP shares the latter, so probe GESTEP and the other base
conversions (DEC2HEX, DEC2OCT, BIN2*, HEX2*, OCT2*) before changing their order. Record the rule in
docs/excel-semantics/engineering.md "Engineering function argument modes".

## Acceptance criteria
- [ ] probes/eng-later-argument-lift M2 and M8 leave eval/baseline/probes.txt, with no new F lines.
- [ ] A probe case pins the order for GESTEP and the other conversions, and each kernel follows it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The lifting itself (T-1383).

## Outcome
New probe probes/eng-first-vs-second-error shows all eleven functions (nine base conversions, DELTA, GESTEP) read the second argument first, scalar and lifted. The kernels now coerce `places`/`number2` before the first argument. probes/eng-later-argument-lift M2 and M8 pass; rule recorded in docs/excel-semantics/engineering.md.
