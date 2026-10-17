---
id: T-1781
title: "Bound TEXTSPLIT's padded result by the shared array limit (a 32,767-char text pads ~2^28 cells)"
milestone: M9
lane: functions-text
status: todo
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
- [ ] TEXTSPLIT returns `check_array_size`'s error before padding any row when rows x widest row exceeds the limit.
- [ ] A unit test shows the 1501 x 1501 case above is `#ERROR!` and a case under the limit is unchanged.
- [ ] arrays.md's DEVIATION entry lists TEXTSPLIT. No eval cell changes (`make eval`: 0 fixes, 0 regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Moving TEXTSPLIT onto the SPEC-0011 driver.

