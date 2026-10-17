---
id: T-1781
title: "Bound TEXTSPLIT's padded result by the shared array limit (a 32,767-char text pads ~2^28 cells)"
milestone: M9
lane: functions-text
status: done
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXTSPLIT refuses a result above the shared array limit before it pads its rows, so a single untrusted text cannot build hundreds of millions of `#N/A` cells (CLAUDE.md directive 5).

## Context
Found while doing T-1771. `fn_textsplit` (crates/rustsheet-engine/src/functions/text/textsplit.rs) splits the text into rows and columns, takes the widest row (`max_cols`) and pads every row with `pad_with` to it without calling `limits::check_array_size`. The text is at most 32,767 characters, but a text whose first part holds the column delimiters and whose second part holds the row delimiters gives about 16,384 x 16,384 = 2^28 padded cells (several GB). Reproduced with a scratch unit test on the T-1771 branch: `=ROWS(TEXTSPLIT(REPT(",",1500)&REPT(";",1500),",",";"))` builds 1501 x 1501 = 2,253,001 cells and gives 1501 (COLUMNS gives 1501), where `&` and ROUND give `#ERROR!` above 1,048,576 cells. Check `check_array_size(rows_of_cols.len(), max_cols)` before padding. The bound is the existing DEVIATION (docs/excel-semantics/arrays.md "Array results above 1,048,576 cells"); add TEXTSPLIT to it. Ask the oracle (excel-probe) what Excel gives for the 1501 x 1501 case and record it there.

## Acceptance criteria
- [x] TEXTSPLIT returns `check_array_cells`'s error (the cell-count half of `check_array_size`) before padding any row when rows x widest row exceeds the limit.
- [x] A unit test shows the 1501 x 1501 case above is `#ERROR!` and a case under the limit is unchanged.
- [x] arrays.md's DEVIATION entry lists TEXTSPLIT. 4 new deliberate known failures (DEVIATION, probe textsplit-padded-size A1-A4), 0 regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- Moving TEXTSPLIT onto the SPEC-0011 driver.

## Outcome
`fn_textsplit` calls `limits::check_array_cells(rows, max_cols)` before padding any row; results at or under 2^20 cells are unchanged (1024 x 1024 still fits). Tests in `test/text_functions/textsplit_bound.rs` (the 1501 x 1501 and 1025 x 1025 cases give `#ERROR!`; failed before the fix). arrays.md DEVIATION lists TEXTSPLIT. Oracle probe `textsplit-padded-size` (excel-graph): Excel builds the array, ROWS and COLUMNS of the 1501 x 1501 split are 1501 and the 1025 x 1025 case gives 1025, so the refusal stays a directive-5 DEVIATION. A 16,000 + 16,000 delimiter case (about 2^28 cells) made the graph backend fail at createSession (retries exhausted), so it is not probed. 4 new deliberate known failures (DEVIATION, probe textsplit-padded-size A1-A4), 0 regressions. Review round: `check_array_size` answers `#VALUE!` past the grid, which changed a 16,385-column split under 2^20 cells; Excel builds it (probe textsplit-wide A1 = 16385), so TEXTSPLIT uses the cell-count-only `limits::check_array_cells`.
