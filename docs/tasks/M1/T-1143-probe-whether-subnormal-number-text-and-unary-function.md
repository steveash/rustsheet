---
id: T-1143
title: "Probe whether subnormal number text and unary-function tails flush (companion cells)"
milestone: M1
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel-graph companion cells (`=(x)=0`) settle two unverified subnormal questions:
- whether subnormal number text flushes: `("1e-308"+0)=0`, `("1e-308"-0)=0`, `(VALUE("1e-309"))=0`;
- whether these functions flush or keep a subnormal result: CSCH(710), NORM.S.DIST(-37.6..-38,TRUE/FALSE), PHI(-37.7), NORM.DIST(19,0,0.5,FALSE) and ERFC/ERFC.PRECISE(26.7).

## Context
T-1027 made `parse_number_text` flush subnormal number text (marked UNVERIFIED in numbers.md). Decision 0042 now scores an excel-graph 0 against a kept subnormal as a pass, but only when the case has a FALSE `(x)=0` companion. The probes-numeric cells for the functions above have no companion, so they still score as mismatches. Write the probe with the excel-probe skill. Then update numbers.md, and file engine fix tasks where Excel differs.

## Acceptance criteria
- [ ] A probe case with oracle values covers the cells above. numbers.md records each result, and fix tasks are filed where the engine differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Outcome
Added probes/subnormal-text-and-tail-companions (15 formulas with `(x)=0` companions, excel-graph 2026-10-08).  Eval delta: probes known failures 379 -> 400 (21 new coverage, 0 regressions). Subnormal number text flushes to a true 0 (VERIFIED, numbers.md). CSCH(710), NORM.S.DIST (TRUE tail and FALSE at -37.75/-38), PHI(-37.7), NORM.DIST(19,0,0.5,FALSE) and ERFC/ERFC.PRECISE(26.7) are a true 0; `NORM.S.DIST(-37.6,FALSE)` is a kept normal. The engine keeps the subnormal in the function cases: fix owned by T-1180.
