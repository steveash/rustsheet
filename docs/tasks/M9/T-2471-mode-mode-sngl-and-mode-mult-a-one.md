---
id: T-2471
title: "MODE, MODE.SNGL and MODE.MULT: a one-cell range reference to an empty cell is #VALUE! (gen-m9, 36 cells)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
MODE, MODE.SNGL and MODE.MULT return `#VALUE!` when an argument is a one-cell range reference (`Data!C11:C11`) to an
empty cell, as they already do for a plain blank single-cell reference.

## Context
Gap triage of the T-2256 gen-m9 sweep. statistics.md "MODE: a one-cell range reference to an empty cell is #VALUE!
(T-2256)"; "Statistical function argument modes" has the plain-reference form (probes/mode-typed-arguments A23). 34 cells
are top-level, two nest MODE in another call. The engine treats `X:X` as a range (blank skipped).
Code: crates/rustsheet-engine/src/functions/statistical/mode_functions.rs (the argument reader that already special-cases
a blank single-cell reference).

Owned gen-m9 cells (T-2256 triage, excel-graph, 2026-10-23): 36 known-failure lines in eval/baseline/gen-m9.txt, 36 visibly off.
Examples:
- gen-m9/09b47629f54b9817 Calc!A188 `=MODE.SNGL(Data!B1:B1)`: Excel #VALUE!, ours #N/A
- gen-m9/5cebac1f02bf30f2 Calc!A183 `=MODE.MULT(Data!A10:A10)`: Excel #VALUE!, ours #N/A
- gen-m9/775d053de498b8c7 Calc!A352 `=MODE(Data!C11:C11)`: Excel #VALUE!, ours #N/A
- gen-m9/7980ffb6095a340d Calc!A62 `=MODE.MULT(Data!B11:B11)`: Excel #VALUE!, ours #N/A
- gen-m9/9735c5ab72fa5ecf Calc!A317 `=MODE.SNGL(Data!C11:C11)`: Excel #VALUE!, ours #N/A
Cells:
- gen-m9/097af1ca33638670.case.json (1): Calc!A134
- gen-m9/09b47629f54b9817.case.json (1): Calc!A188
- gen-m9/139684e68fb2fc3b.case.json (2): Calc!A40, A374
- gen-m9/2138b909d8f810ea.case.json (1): Calc!A177
- gen-m9/329b177b5d5165ad.case.json (1): Calc!A172
- gen-m9/334f3dc9ca937f10.case.json (1): Calc!A216
- gen-m9/43c6227e0ff368c7.case.json (2): Calc!A9, A133
- gen-m9/5389dd4d5b5739d2.case.json (1): Calc!A353
- gen-m9/583ca2baea0bb30a.case.json (1): Calc!A45
- gen-m9/5cebac1f02bf30f2.case.json (2): Calc!A145, A183
- gen-m9/607e85cfee83a689.case.json (1): Calc!A171
- gen-m9/775d053de498b8c7.case.json (3): Calc!A97, A104, A352
- gen-m9/7980ffb6095a340d.case.json (1): Calc!A62
- gen-m9/7f7ca726c3f68554.case.json (1): Calc!A122
- gen-m9/80586b859b73c87d.case.json (1): Calc!A205
- gen-m9/873f30d2bf76f8db.case.json (1): Calc!A71
- gen-m9/91e167cd8d7eb652.case.json (1): Calc!A342
- gen-m9/92d4fa29e5510f1e.case.json (1): Calc!A14
- gen-m9/9735c5ab72fa5ecf.case.json (2): Calc!A165, A317
- gen-m9/a2262ea466fcc142.case.json (1): Calc!A163
- gen-m9/a4090567758520df.case.json (1): Calc!A185
- gen-m9/a5342a7e24fd803e.case.json (1): Calc!A64
- gen-m9/a63d1c7d6f96c56d.case.json (1): Calc!A381
- gen-m9/bbab6562e88b70e8.case.json (3): Calc!A57, A159, A176
- gen-m9/bcb1105c61a2649d.case.json (1): Calc!A341
- gen-m9/c7664e494e418395.case.json (1): Calc!A251
- gen-m9/d910f000b60e2eac.case.json (1): Calc!A351
- gen-m9/eccd03df42e40006.case.json (1): Calc!A198

T-2436 probe (excel-graph, 2026-10-24; probes/t2436/<case>, each gen-m9 cell's sub-expressions on its Data sheet): `MODE.SNGL(Data!B4:B10,Data!D10:D10,..)` and `MODE.MULT(Data!D10:D10,Data!D1:D6)` with D10 empty are `#VALUE!` (ours `#N/A`).
Owned gen-m9 cells (T-2436 probe, excel-graph, 2026-10-24): 2 known-failure lines in eval/baseline/gen-m9.txt, 2 formulas.
Examples:
- gen-m9/984cba67bbe6972b Calc!A277 `=AVERAGEA(MODE.SNGL(Data!B4:B10,Data!D10:D10,(1e-15^3.14159)),"ABC")`: Excel #VALUE!, ours #N/A; inner `MODE.SNGL(Data!B4:B10,Data!D10:D10,(1e-15^3.14159))` Excel #VALUE!, ours #N/A
- gen-m9/ecc87f80077959e0 Calc!A187 `=CONFIDENCE.T(RADIANS(STDEV.S(Data!E5:E12,Data!A1:A1,{0.02,0.027})),ACCRINTM(MODE.MULT(Data!D10:D10,Data!D1:D6),39448,(-0.5),4317,3),8)`: Excel #VALUE!, ours #N/A; inner `MODE.MULT(Data!D10:D10,Data!D1:D6)` Excel #VALUE!, ours #N/A
Cells:
- gen-m9/984cba67bbe6972b.case.json (1): Calc!A277
- gen-m9/ecc87f80077959e0.case.json (1): Calc!A187

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2436): these 2 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Visible parity (gen-m9, T-2256): these 36 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] Unit tests: `MODE(Z1:Z1)`, `MODE.SNGL(1,1,Z1:Z1)` and `MODE.MULT(Z1:Z1,{5})` with Z1 empty are `#VALUE!`;
      `MODE(A1:A1,A1:A2)` with numbers is unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
- MODE over multi-cell ranges with blanks (unchanged: blanks skipped).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2256): every cell it owns has the wrong type, error or text (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns
has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity
criterion under Acceptance criteria states it.

## Outcome
MODE, MODE.SNGL and MODE.MULT: a one-cell range to an empty cell is `#VALUE!`. Eval delta: 39 gen-m9 known-failure
lines fixed (all 38 owned plus probes/t2475-stat-subexpressions P!A96 and one more nested cell), 18 probes lines fixed, 0 regressions.
