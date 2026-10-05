---
id: T-0234
title: "Probe SUMIF/AVERAGEIF resized sum_range past the sheet edge and single-cell ranges in *IFS"
milestone: M2
lane: functions-criteria
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish with oracle evidence (excel-probe skill) what Excel does in three cases T-0214 left unevidenced, and make the engine match.

## Context
T-0214 resizes SUMIF/AVERAGEIF sum_range to the criteria range's shape. Reviewer findings:
- When the resized range runs past XFD1048576, `resize_to` clamps to the sheet edge; Excel's behavior (clamp vs `#REF!`) is unevidenced.
- `apply_ifs` now evaluates the first range by reference, so `SUMIFS(A1,B1:B1,...)` / MINIFS etc. with a single-cell sum range compute a value instead of `#VALUE!`; no oracle cell covers it.
- Criteria ranges are still evaluated in context, so a single-cell criteria range (`=SUMIF(A1,">0")`, `SUMIFS(A1:A1,B1,">0")`) returns `#VALUE!`; Excel likely treats it as a 1x1 range.
  Note the split: the array/range-criteria SUMIF path (`node_to_range`) already takes criteria ranges by reference, so `=SUMIF(A1,{">0"},B1)` treats A1 as 1x1 while the scalar path errors; make both match the oracle.
Code: `crates/rustsheet-engine/src/functions/statistical/if_ifs.rs`; rule in docs/excel-semantics/criteria.md.

## Acceptance criteria
- [ ] A probe case (probes suite) covers: SUMIF/AVERAGEIF with sum_range at XFD1, A1048576 and XFD1048576 and a 2x2 criteria range; single-cell sum range in SUMIFS/MINIFS; single-cell criteria range in SUMIF/SUMIFS.
- [ ] Engine matches the oracle values; the hand-written grid-edge assertion in test_fn_sumifs.rs is replaced or confirmed.
- [ ] docs/excel-semantics/criteria.md entries become VERIFIED.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Shape-mismatch rules (T-0209, T-0214).
