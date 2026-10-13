---
id: T-1464
title: "Probe UNICHAR with a range, boolean and empty-cell argument"
milestone: M5
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
UNICHAR's behavior for a multi-cell range, a boolean and an empty-cell argument is evidenced by an Excel-valued probe, and any engine mismatch is fixed or filed.

## Context
T-1281 made `fn_unichar` (functions/text/char_code.rs) lift over arrays via `text_arg` and `broadcast_text`; only the array-literal shape `UNICHAR({65,66})` is oracle-backed (probes/dollar-fixed-unichar-edges A45/B45). A range argument, `UNICHAR(TRUE)` (engine: U+0001) and an empty reference (engine: `#VALUE!`) are unprobed. Add cells to a new probe case, fill with `rustsheet-oracle fill -b excel-graph --suite probes`, and record the result in docs/excel-semantics/text.md#unichar-code-point-bounds.

## Acceptance criteria
- [ ] A probe case has oracle cells for UNICHAR over a range, TRUE and an empty cell, recorded as VERIFIED in text.md.
- [ ] Every failing cell is fixed (if local) or filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
