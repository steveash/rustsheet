---
id: T-2003
title: "Probe and match overflow in IMSIN IMCOS IMSINH IMCOSH and IMPRODUCT with a blank"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IMSIN, IMCOS, IMSINH, IMCOSH and IMPRODUCT-with-a-blank-cell overflow give what Excel gives, not `inf` text.

## Context
Found by T-1827 review: `IMCOSH("1000")`, `IMSIN("1+1000i")` print inf text; `IMPRODUCT("1E200","1E200",<blank>)` gives 0 via the blank shortcut. Also probe IMSUB overflow, `IMSUM("9E307","9E307","-9E307")` and an overflow followed by a later error argument. Probe first (excel-probe skill).

## Acceptance criteria
- [x] A probe pins each form and ours matches.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `eng-complex-overflow` (21 cells, excel-graph). IMSIN/IMCOS/IMSINH/IMCOSH overflow is `#NUM!`; IMSUB sets an overflowing part to 0; IMSUM/IMPRODUCT overflow is `#VALUE!` immediately (beats later bad args, cancelling terms and blanks). All fixed in complex.rs and recorded in engineering.md. Eval: probes +1 case, 21/21 pass.
