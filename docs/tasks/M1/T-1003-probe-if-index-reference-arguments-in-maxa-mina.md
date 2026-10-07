---
id: T-1003
title: "Probe IF/INDEX reference arguments in MAXA, MINA, AVERAGEA, PRODUCT, LARGE, RANK"
milestone: M1
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Record what Excel does for `IF(...)` and `INDEX(...)` arguments of MAXA, MINA, AVERAGEA, PRODUCT, LARGE and RANK.

## Context
Follow-up of T-0196, which routed those functions through `evaluate_aggregate_arg`, so IF/INDEX arguments now arrive as references (`=MAXA(IF(TRUE,A1))` with A1 = text "5" went from 5 to 0). references.md "IF and INDEX return references to aggregates" has evidence for SUM, AVERAGE, MIN, MAX and COUNT only. Probe MAXA, AVERAGEA, PRODUCT, LARGE and RANK over IF(TRUE,textcell) and INDEX of a text or boolean cell; if Excel differs, file the engine fix.

## Acceptance criteria
- [ ] Probe case and excel-graph sidecar committed; results in references.md (status VERIFIED or the corrected rule).
- [ ] New-case engine mismatches are baselined; no engine change.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (file them separately).

## Outcome
Probe `probes/if-index-ref-args-aggregates` (54 cells, excel-graph): IF and INDEX arguments behave exactly like the direct reference in all six functions, so T-0196's routing is right; references.md is VERIFIED. 48/54 cells pass; the 6 RANK mismatches (`#N/A` expected, `#NUM!` actual, direct references included) are baselined and filed as T-1055. No engine change.
