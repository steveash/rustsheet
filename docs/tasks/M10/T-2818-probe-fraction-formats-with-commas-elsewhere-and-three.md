---
id: T-2818
title: "Probe fraction formats with commas elsewhere and three or more trailing commas"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel's result for fraction formats with commas in other positions, three or more trailing commas, `%` mixed with `,,`, and the displayed text (not TEXT) of the #VALUE! cases is probed and implemented.

## Context
T-2072 implemented `# ?/4,` and `# ?/4,,` (formatter/fraction.rs) from probes/fraction-format-features rows 5-6. The parser counts every comma after a digit placeholder that is not between digits in `part.comma`, so `0, ?/4` and `# ?,/4` now take the same path unverified; three or more commas are treated as one. The single-comma #VALUE! returns None (255 `#` displayed), the over-255 case displays #VALUE!; displayed text is unprobed.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes covers the positions above with Excel oracle values; the code matches every cell, `make eval-baseline` records it.
- [ ] docs/excel-semantics/number-formats.md drops the UNVERIFIED notes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
