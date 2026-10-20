---
id: T-2185
title: "Probe relative defined names past the grid edge and through INDIRECT"
milestone: M8
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what Excel gives for a relative defined name pushed past the last row or column, and for `INDIRECT` of a relative name's text, recorded in docs/excel-semantics/references.md.

## Context
T-1639 made a relative name (`Jan!A1`, stored relative to A1) read at the same offset from the
using cell (`NameReference::at`, crates/rustsheet-engine/src/defined_names.rs). Two edges are
unprobed (references.md "Relative references in defined names"): `rel_down` = `Jan!A2` used in
row 1048576 (and `Jan!B1` used in column XFD), where Excel is believed to wrap to row 1 / column A
while the engine reads cells past the grid as empty; and `INDIRECT("rel_same")`, which the engine
reads from the INDIRECT's cell. Write a probe case (excel-probe skill) with both, plus a
formula-bodied name over a relative range (`=SUM(Jan!A1:A2)`) used on the last row, and a
mixed-anchor name used above its anchor (`inv` = `Jan!$B$5:$B1` from row 1: the engine reads
B1:B5, ROWS 5, as an inline reference is).

## Acceptance criteria
- [ ] A probe case under eval/suites/probes with oracle values for the edge cells above.
- [ ] references.md updated with the findings; an engine follow-up task filed for any mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a follow-up task does that).
