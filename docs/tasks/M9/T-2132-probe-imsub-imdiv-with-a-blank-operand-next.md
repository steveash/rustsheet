---
id: T-2132
title: "Probe IMSUB/IMDIV with a blank operand next to a j number, and IMSUM with mixed empty-argument forms"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IMSUB, IMDIV and IMSUM give Excel's result for a blank operand next to a `j` number and for mixed empty-argument forms.

## Context
T-1828 found that a suffixless real number takes the other operand's suffix (engineering.md). `complex_of` in
functions/engineering/complex.rs still returns `Suffix::I` for a blank with `ZeroIfBlank`, so `IMSUB("3+4j",Q1)` is
`#NUM!` in ours; unprobed. Also unprobed: `IMSUM(,Q1)` (ours "0") and `IMSUM(,"")` (ours `#NUM!`), where `IMSUM(,)` is `#N/A`.

## Acceptance criteria
- [ ] One probe case covers `IMSUB("3+4j",Q1)`, `IMSUB(Q1,"3+4j")`, `IMSUB("3+4j",)`, IMDIV with a blank first and second
  argument, `IMSUM(,Q1)` and `IMSUM(,"")`; answers recorded in engineering.md and matched (`Suffix::Real` for a blank if Excel agrees).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probe `probes/eng-imsub-imdiv-blank-j` (25 cells, excel-graph 2026-10-20): a blank takes the other operand's suffix
(`complex_of` now returns `Suffix::Real`), an empty IMSUB/IMDIV argument is `#N/A`, and an empty first IMSUM/IMPRODUCT
argument is `#N/A` (a later one is skipped). All recorded in engineering.md. Eval: probes baseline +25 cells passing.
