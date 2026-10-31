---
id: T-3405
title: "Probe what Excel does with an xlsx whose style or font indices dangle"
milestone: M12
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Know what Microsoft 365 Excel does when it opens an xlsx whose indices dangle, so the T-2913 DEVIATION
in `docs/excel-semantics/xlsx-storage.md` ("A workbook whose indices dangle is refused outright") is
backed by evidence rather than UNVERIFIED.

## Context
T-2913 made `Model::from_workbook` refuse a decoded workbook whose cell style, font/fill/border id,
shared-formula index, spill range, row/column record or merged range dangles (directive 5: these
indices are read unchecked and panicked later, T-2882). Excel's own behavior on such files was not
probed. If Excel repairs them ("We found a problem with some content...") rather than refusing, the
xlsx importer could repair the same way before the model is built, and the refusal would remain only
as the no-panic backstop for decoded bytes.
Code: `crates/rustsheet-engine/src/workbook_check.rs`, `crates/rustsheet-xlsx` import of styles.

## Acceptance criteria
- [ ] Probe files (via the `excel-probe` skill, on excel-graph): a cell with `s=` past `<cellXfs>`, and a
  `<xf fontId=>` past `<fonts>`; record whether Excel opens, repairs (and to what) or refuses each.
- [ ] The xlsx-storage.md entry cites the evidence and its status is updated; if Excel repairs, a
  follow-up task is filed to repair in the importer (refusal vs repair noted).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
