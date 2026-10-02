---
id: T-0112
title: "Import POI workbooks: missing workbook rels (49609) and Strict OOXML (r:id)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
depends_on: []
---

## Goal
Import `poi/49609.xlsx`, `poi/SampleSS.strict.xlsx` and `poi/SimpleStrict.xlsx`, or reject them with a documented, intentional error.

## Context
Found by T-0003. 49609 fails with "missing workbook rels"; the Strict files fail with `Missing "r:id" XML attribute` (Strict OOXML uses a different relationships namespace). See `F poi/... !InvalidCase|!EngineError` in `eval/baseline/poi.txt`.

## Acceptance criteria
- [ ] The three cases import without error, or a decision record justifies refusing a given file.
- [ ] Their baseline `!` entries are removed via `make eval-baseline`.
- [ ] `make check` green; reviewers approve.

## Out of scope
.xls (BIFF) files.
