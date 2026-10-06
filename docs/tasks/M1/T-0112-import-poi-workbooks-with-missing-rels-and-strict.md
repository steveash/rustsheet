---
id: T-0112
title: "Import POI workbooks: missing workbook rels (49609) and Strict OOXML (r:id)"
milestone: M1
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Import `poi/49609.xlsx`, `poi/SampleSS.strict.xlsx` and `poi/SimpleStrict.xlsx`, or reject them with a documented, intentional error.

## Context
Found by T-0003. 49609 fails with "missing workbook rels"; the Strict files fail with `Missing "r:id" XML attribute` (Strict OOXML uses a different relationships namespace). See `F poi/... !InvalidCase|!EngineError` in `eval/baseline/poi.txt`.

## Acceptance criteria
- [x] The three cases import without error, or a decision record justifies refusing a given file.
- [x] Their baseline `!` entries are removed via `make eval-baseline`.
- [x] `make check` green; reviewers approve.

## Out of scope
.xls (BIFF) files.

## Outcome
- Excel for the web opens poi/49609.xlsx (backslash, lower-case zip entry names; oracle probe
  2026-10-06). `PartNames` (rustsheet-xlsx `import::package`, public for the eval snapshot reader)
  finds parts case-insensitively with `\` read as `/`; the importer and `rustsheet-eval snapshot`
  share it. 49609 now imports (no formulas, not Excel-authored: Pending, not scored).
- `XmlNode::relationship_id` reads `r:id` in the Strict OOXML namespace too, so SimpleStrict,
  SampleSS.strict, sample.strict and 57914 import; their cached cells pass.
- Eval: poi known failures 635 -> 630 (five `!` entries gone), +4 newly checked cells all passing,
  0 regressions. Strict `t="d"` date cells are still dropped: T-0406. Strict files declare
  `dateCompatibility="0"`, which the importer ignores; probe filed as T-0407.
