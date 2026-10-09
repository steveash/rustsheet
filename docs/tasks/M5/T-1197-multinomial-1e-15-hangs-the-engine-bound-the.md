---
id: T-1197
title: "MULTINOMIAL(1E+15) hangs the engine: bound the loop (gen-m5 timeout)"
milestone: M5
lane: functions-math
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
MULTINOMIAL of a huge argument (e.g. 1E+15) terminates within the resource limits and matches Excel's result.

## Context
T-1192's gen-m5 case 21b9ae9affd6b38c times out (60s): `=MULTINOMIAL(1E+15)` alone (Calc!A25 there) hangs the engine, a loop proportional to the argument. Directive 5 requires bounded work on untrusted input. Probe what Excel returns for large arguments (likely `#NUM!`), then bound the loop; any deviation gets a DEVIATION entry. Code: crates/rustsheet-engine/src/functions/math_and_trigonometry/multinomial.rs. Related: T-1190 (inexact values).

## Acceptance criteria
- [ ] `MULTINOMIAL(1E+15)` returns Excel's result in well under a second (probe evidence cited).
- [ ] gen-m5/21b9ae9affd6b38c runs to completion.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
