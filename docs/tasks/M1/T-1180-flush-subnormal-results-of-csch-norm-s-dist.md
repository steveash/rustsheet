---
id: T-1180
title: "Flush subnormal results of CSCH, NORM.S.DIST, PHI, NORM.DIST, ERFC and ERFC.PRECISE to 0"
milestone: M1
lane: functions-math
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
CSCH, NORM.S.DIST, PHI, NORM.DIST and ERFC/ERFC.PRECISE return a true 0 where Excel does, instead of a kept subnormal.

## Context
probes/subnormal-text-and-tail-companions (T-1143, excel-graph 2026-10-08) has FALSE-or-TRUE `(x)=0` companions. Excel gives a true 0 (companion TRUE) for CSCH(710), NORM.S.DIST(-37.6/-37.75/-38,TRUE), NORM.S.DIST(-37.75/-38,FALSE), PHI(-37.7), NORM.DIST(19,0,0.5,FALSE), ERFC(26.7) and ERFC.PRECISE(26.7). `NORM.S.DIST(-37.6,FALSE)` is 4.0441448093482273e-308, a normal number that is kept. The engine keeps a subnormal in the first group (B5:B8, B10:B15 fail). See numbers.md "Subnormal results become 0". Excel's mechanism (a flush of a subnormal result versus an overflow or cutoff inside the algorithm) is not determined by these cells, so match the probed arguments first; probe the boundary (CSCH(709.3), CSCH(709.6), CSCH(-710), ERFC between 26.4 and 26.6, NORM.S.DIST(k,FALSE) for k from -37.61 to -37.65) with the excel-probe skill before generalizing. Prefer `arithmetic::flush_subnormal` over a new helper if a flush matches.

## Acceptance criteria
- [ ] The B5:B8 and B10:B15 companions and A5:A8, A10:A15 of probes/subnormal-text-and-tail-companions pass, and A9 (`NORM.S.DIST(-37.6,FALSE)`, a kept normal value 4.0441448093482273e-308 where the engine is one ulp off, 4.044144809348227e-308) passes; `make eval-baseline` records the fixes.
- [ ] `make check` green; reviewers approve.

## Out of scope
None.
