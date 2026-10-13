---
id: T-1281
title: "Probe DOLLAR, FIXED and UNICHAR edge cases (evidence gap: 2-3 Excel cells each)"
milestone: M5
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
DOLLAR, FIXED and UNICHAR each have at least 25 Excel-valued cells in eval, covering rounding, negative and out-of-range decimals and code-point bounds, and any engine mismatch the probe finds is fixed (if local to the function) or filed as a task.

## Context
Gap triage 2026-10-09 (decision 0031): report.md "Excel evidence per function" lists DOLLAR with 2 Excel cells (prior 0.2), FIXED with 3 (prior 0.2) and UNICHAR with 2 (prior 0.05), all M5. The only evidence is probes/text-argument-modes (`DOLLAR(A6)`, `FIXED(A2)`, `FIXED(A6)`, `UNICHAR(A3)`; text.md "Text function argument modes") and the availability probes. The code makes several unprobed choices: `fn_dollar` and `fn_fixed` take `floor` of decimals (so -1.5 becomes -2, not -1), put the sign or parentheses on the unrounded value, and accept any decimals size; `fn_unichar` takes `floor` and gives `#VALUE!` for 0, negatives and any value `char::from_u32` rejects (surrogates, above 0x10FFFF).

Write one probe case `eval/suites/probes/cases/dollar-fixed-unichar-edges.case.json` (excel-probe skill). Suggested cells:
- DOLLAR: `DOLLAR(1234.567)`, `(1234.567,0)`, `(1234.567,-2)`, `(-1234.567)`, `(-0.004)` (rounds to zero: "$0.00" or "($0.00)"?), `(0.125,2)`, `(2.675,2)` (binary-tie rounding), `(1E+15)`, `(1234.5,1.9)`, `(1234.5,-1.5)`, `(1,127)`, `(1,128)`, `(1,-128)`, `DOLLAR("12")`, `DOLLAR(TRUE)`.
- FIXED: `FIXED(1234.567)`, `(1234.567,1,TRUE)`, `(1234.567,-2)`, `(-1234.567,-2,TRUE)`, `(0.125,2)`, `(2.675,2)`, `(1E+15,2)`, `(1234.5,1.9)`, `(1234.5,-1.5)`, `(1,127)`, `(1,128)`, `(1,-400)`, `FIXED(1234,0,"x")` (text no_commas), `FIXED(1234,0,2)`.
- UNICHAR: `UNICHAR(65)`, `(65.9)`, `(0)`, `(-1)`, `(1)`, `(160)`, `(55296)` and `(57343)` (surrogates), `(65535)`, `(65536)`, `(128512)`, `(1114111)`, `(1114112)`, `LEN(UNICHAR(128512))`, `UNICHAR("66")`, `UNICHAR({65,66})` (lift?).
Fill with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`.

Code: `fn_dollar`, `fn_fixed` and `format_abs` (crates/rustsheet-engine/src/functions/text/string_format.rs); `fn_unichar` (crates/rustsheet-engine/src/functions/text/char_code.rs).

## Acceptance criteria
- [ ] `probes/dollar-fixed-unichar-edges` is committed with its excel-graph oracle sidecar, and report.md's evidence table shows DOLLAR, FIXED and UNICHAR with 25 or more Excel cells each.
- [ ] docs/excel-semantics/text.md records the results (VERIFIED, citing cells): how fractional and negative decimals are truncated, the decimals bound, the sign of a value that rounds to zero, and the UNICHAR error kinds per range.
- [ ] Every failing probe cell is either fixed in this PR (only if local to the function) or filed as a task naming the cells; a FIXED rounds-to-zero sign failure is attributed to T-1257 instead of a new task. `make eval-baseline` shows no known failures outside the new case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- TEXT and number-format rendering (number-formats.md), BAHTTEXT (T-1214).
- Any last-digit rounding work beyond filing it (decimal rounding is shared with ROUND; file a task if a tie case differs).

## Outcome
Added `probes/dollar-fixed-unichar-edges` (76 excel-graph cells). Fixed: DOLLAR/FIXED `decimals` truncates toward
zero (was floor) and is `#VALUE!` from 128; UNICHAR lifts over arrays (static analysis said not-implemented), gives
`#N/A` for surrogates and Unicode noncharacters. Recorded in docs/excel-semantics/text.md. The FIXED rounds-to-zero
sign cells (A49, A50) remain known failures, attributed to T-1257 (evidence added there). Eval: probes known
failures 2853 -> 2855 (the 2 FIXED cells; 74/76 cells pass, no regressions).
