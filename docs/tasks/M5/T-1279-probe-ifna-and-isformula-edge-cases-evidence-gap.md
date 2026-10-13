---
id: T-1279
title: "Probe IFNA and ISFORMULA edge cases (evidence gap: 18 and 10 Excel cells)"
milestone: M5
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
IFNA and ISFORMULA each have at least 25 Excel-valued cells in eval, covering their scalar, array and reference edges, and any engine mismatch the probe finds is fixed (if local to the function) or filed as a task.

## Context
Gap triage 2026-10-09 (decision 0031): report.md "Excel evidence per function" lists IFNA with 18 Excel cells (prior 1) and ISFORMULA with 10 (prior 0.1), both M5 and below the 25-cell threshold. All current cells pass, but the evidence is mostly rich-error and argument-mode probes (probes/error-literals-new, rich-errors-args, error-stored-constants, info-logic-argument-modes; information.md "Information function argument modes").

Write one probe case `eval/suites/probes/cases/ifna-isformula-edges.case.json` (excel-probe skill). Suggested cells:
- IFNA: `IFNA(#N/A,1)`, `IFNA(NA(),"x")`, `IFNA(#DIV/0!,1)`, `IFNA(#VALUE!,1)` (other errors pass through), `IFNA(5,1)`, `IFNA("",1)`, `IFNA(#N/A,)` (empty fallback), `IFNA(#N/A,#DIV/0!)`, `IFNA(VLOOKUP(9,A1:B3,2,FALSE),"none")`, `IFNA(MATCH("z",A1:A3,0),0)`, `IFNA({1,#N/A,3},0)` (spills), `SUM(IFNA({1,#N/A,3},10))`, `IFNA(A1:A3,0)` over a range holding `#N/A`, `IFNA(#N/A,{1,2})`, `IFNA(Data!<blank>,1)`.
- ISFORMULA (reference arguments only: excel-graph refuses a workbook with `ISFORMULA(1)` or `ISFORMULA("A1")`, information.md): a formula cell, a constant number, a constant text, a blank cell, a formula returning `""`, a formula returning an error, a spill anchor and a spilled child (`=SEQUENCE(3)` in one cell, then ISFORMULA of the anchor and of the cell below), a cell on another sheet, `ISFORMULA(INDIRECT("A1"))`, `ISFORMULA(OFFSET(A1,0,0))`, `ISFORMULA(INDEX(A1:A3,2))`, `ISFORMULA(A1:A3)` (lifts, already known), and ISFORMULA of its own cell's neighbour holding a legacy array formula if the case format supports one.
Fill with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`. Drop any cell excel-graph refuses and note it.

Code: `fn_ifna` / `if_error_like` (crates/rustsheet-engine/src/functions/logical/mod.rs); `fn_isformula` (crates/rustsheet-engine/src/functions/information/mod.rs).

## Acceptance criteria
- [x] `probes/ifna-isformula-edges` is committed with its excel-graph oracle sidecar, and report.md's evidence table shows IFNA and ISFORMULA with 25 or more Excel cells each.
- [x] docs/excel-semantics/information.md (ISFORMULA) and logic.md (IFNA) record the results (VERIFIED, citing cells), including the spilled-child and empty-fallback answers.
- [x] Every failing probe cell is either fixed in this PR (only if local to the function) or filed as a task naming the cells; an IFNA-of-a-blank failure is attributed to the existing T-0291 instead of a new task. `make eval-baseline` shows no known failures outside the new case.
- [x] `make check` green; reviewers approve.

## Out of scope
- IFERROR (1632 Excel cells) and the IFERROR/IFNA blank-cell rule itself (T-0291).

## Outcome
Added probes/ifna-isformula-edges (40 cells, excel-graph). 39 pass; `ISFORMULA(INDEX(range,1))` (Calc!B12) fails with #VALUE! and is filed as T-1453 (known failure in the probes baseline). IFNA and ISFORMULA semantics recorded in logic.md and information.md. `ISFORMULA(1)`/`ISFORMULA("A1")` were not probed (excel-graph refuses them, already documented).
