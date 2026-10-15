---
id: T-1411
title: "ASC lifts over a multi-cell range (#VALUE! today)"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
ASC lifts over a multi-cell range (one result per cell, a spill), as the other M5 text functions do.

## Context
T-1289's gen-m9 suite (excel-graph sidecars, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/65faaeb48419087d Calc!A5 `=ASC(INDEX(Data!A6:D8,0,3))` (column C6:C8): Excel `#SPILL!` (an array result blocked by
  the next Calc cells), engine `#VALUE!` (`ROWS(ASC(Data!A6:A8))` is `#VALUE!`; `ROWS(LEN(INDEX(Data!A6:D8,0,3)))` is 3).
docs/excel-semantics/text.md "Text function argument modes": the M5 text functions lift scalar arguments over a range or
array; ASC was only probed on one cell. DBCS also gives `#VALUE!` for a range in the engine: probe `ROWS(DBCS(A1:A3))` and
`ROWS(ASC({"a";"b"}))` with the excel-probe skill and fix DBCS here too if Excel lifts it. Reuse the text lift helper
(functions/text/text_arg.rs). Code: `fn_asc` (crates/rustsheet-engine/src/functions/text/char_code.rs).

## Acceptance criteria
- [ ] gen-m9/65faaeb48419087d A5 passes in `./target/release/rustsheet-eval show gen-m9/<case>.case.json`, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] The ASC/DBCS lift probe cells pass; text.md says ASC (and DBCS, per the probe) lifts.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
