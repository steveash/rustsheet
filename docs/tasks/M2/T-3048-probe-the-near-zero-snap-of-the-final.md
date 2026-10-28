---
id: T-3048
title: "Probe the near-zero snap of the final addition in SUMIF, SUMIFS, the AVERAGE family and SUMPRODUCT"
milestone: M2
lane: research
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for whether SUMIF, SUMIFS, AVERAGE, AVERAGEA, AVERAGEIF, AVERAGEIFS and SUMPRODUCT snap a near-zero
total to 0 the way SUM, SUBTOTAL(9/109), AGGREGATE(9) and PERCENTOF do, with the boundary located for each one that
does, so the engine follow-up can be filed with exact cells.

## Context
- docs/excel-semantics/numbers.md "Near-zero snapping", "SUM forms": SUM snaps its final addition wherever it sits
  (`=SUM(A1:A3)*1` over 0.1, 0.2, -0.3 is 0; a trailing 0 addend stops it), and so do SUBTOTAL(9/109), AGGREGATE(9)
  and PERCENTOF (probes/near-zero-sum-forms, near-zero-aggregate-forms, percentof-snap-overflow-union).
- No probe covers the other summing functions. The engine snaps only in `fn_sum` / `SumParts`
  (functions/math_and_trigonometry/sum.rs), `subtotal_sum`, the AGGREGATE dispatch and PERCENTOF, so today
  `SUMIF(A1:A3,"<>x")`, `SUMIFS(A1:A3,B1:B3,"y")`, `AVERAGE(A1:A3)` and `SUMPRODUCT(A1:A3,{1;1;1})` over 0.1, 0.2,
  -0.3 return a residue (5.55e-17 or a third of it). If Excel snaps any of them, every such cell is visibly off.
  SPEC-0009 (docs/specs/SPEC-0009-aggregate-argument-walker.md) keeps the last-addition tracking as SUM's own, so the
  answer decides whether that state moves into the shared walker.
- Opus: the snap is a numeric boundary (8 ULP of the smaller addend, T-0110, T-2714), and AVERAGE may snap its sum,
  its quotient or neither, which needs boundary pairs to tell apart.

## Acceptance criteria
- [ ] A new excel-graph probe case `probes/t3048-snap-conditional-sums` (milestone M2, decision 0043) reads back, for
      each of SUMIF (with and without sum_range), SUMIFS, AVERAGE, AVERAGEA, AVERAGEIF, AVERAGEIFS and SUMPRODUCT
      (one array, two arrays, and an `--(cond)` mask): the 0.1/0.2/-0.3 cancel over a range, the same with a trailing
      0 addend, the same with a skipped (criteria-rejected) row last, `*1` of the call, and a boundary pair at k = 7
      and k = 8 for addends near 1. SUM over the same ranges is the control in every row.
- [ ] numbers.md "Near-zero snapping" records, per function, whether it snaps, on which addition (the last kept
      addend, or the last row visited), and for AVERAGE-family calls whether the sum or the quotient is snapped,
      citing the cells.
- [ ] Every function Excel snaps that the engine does not has its cells as known failures in eval/baseline/probes.txt
      and one filed follow-up task (lane functions-criteria for the *IF(S) functions, functions-stat for the AVERAGE
      family, functions-math for SUMPRODUCT) naming those cells; `make exit-owners MILESTONE=M2` reports 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (the follow-up tasks own them).
- Root `+`/`-` with array operands (T-3047) or under pickers (T-3049).
- DSUM, DAVERAGE and SUMX2PY2-style sums (file a probe task in their milestone if this one finds snapping widespread).
