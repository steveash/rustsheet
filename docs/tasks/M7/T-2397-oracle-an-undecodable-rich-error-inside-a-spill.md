---
id: T-2397
title: "Oracle: an undecodable rich error inside a spill is scored as an expected blank; leave it pending instead"
milestone: M7
lane: harness
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
A cell Excel fills with a rich-value error the harness cannot decode is never scored, inside a spill as well, and gen-m7's 64 such cells (36 from T-2359, 28 from the T-2254 triage) stop counting as known failures.

## Context
Re-split from T-2359. `rustsheet-oracle fill` drops `#UNDECODED!` cells from the sidecar (`_drop_undecoded` in
tools/src/rustsheet_tools/oracle/cli.py) so they stay pending. But the runner reads a spilled cell that has no sidecar entry as
an expected blank, so the dropped cell is scored as empty. probes/groupby-lifted-args (excel-graph, 2026-10-22, T-2359) shows
it: `GROUPBY(Data!A1:A4,Data!B1:B4,LAMBDA(w,REGEXEXTRACT("a","[0-9]",-1)))` spills `#VALUE!` (ERROR.TYPE 3 through INDEX, K9),
but excel-graph returns the spilled cells as rich errors. Some were reported UNDECODED and some were not reported at all (B33:B36),
and every one ended up as an expected blank until the probe read them through INDEX.

Owned cells, 36 lines in eval/baseline/gen-m7.txt: gen-m7/18b80d3fd292d63a.case.json C951:E962,
`=_xlfn.GROUPBY(Data!A1:B12,Data!B1:D12,LAMBDA(w,(REGEXEXTRACT(Data!A5,"[0-9]",(-1))/-Data!D10)))`. Excel <empty>, ours `#VALUE!`;
Excel's real value is `#VALUE!` (the probe), so ours is right.

Fix: record undecoded cells explicitly in the sidecar (for example the `#UNDECODED!` marker, which the runner never scores)
instead of dropping them. Also find why some rich errors in a spill are not reported at all. Then re-fill the case.

More cells (T-2254 triage, excel-graph, 2026-10-22), 28 lines, all holes excel-graph left inside a spill: cells of a spill have no sidecar
entry while their neighbours do, so the runner scores them as expected blanks. In each the unreported cells are where the value is an
error coming from REGEXEXTRACT (rich errors, as in the GROUPBY case above); ours gives `#VALUE!` there, which is likely right:
`EXPAND(EXPAND({1},2,3),3,Data!E5,<REGEXEXTRACT(..,2)+AVERAGE(..)>)` reports the inner 2x3 (1 and plain `#N/A` pads) but none of the
3x7 result's other cells, the outer pad being the REGEXEXTRACT error (0624b684889e9228 A612); `REGEXEXTRACT(..,"x*",GROUPBY(..),1)` has 11
holes among reported "" and `#VALUE!` cells (21b4eef18424ce94 A1504; its 26 "" cells are T-2360's); `REGEXEXTRACT(..,"^.",TOROW(Data!C10:D11),0)`
reports A136 `#VALUE!` and C136 "?" but not B136, D136 (9bb61f6ae34c7e20). These are the "not reported at all" kind: re-fill these three
cases once the fix lands and read the holes through INDEX/ERROR.TYPE probe cells if the fill still leaves them out.
Owned: 28 known-failure lines in eval/baseline/gen-m7.txt, 3 formulas.
- gen-m7/0624b684889e9228.case.json A612 `=EXPAND(EXPAND({1},2,3),3,Data!E5,(REGEXEXTRACT(Data!D11,"a|b",2)+AVERAGE(Data!C7:C10,Data!A6:A9)))`: D612 Excel <empty>, ours #VALUE!; cells (15): D612:G612, D613:G613, A614:G614
- gen-m7/21b4eef18424ce94.case.json A1504 `=REGEXEXTRACT((("#N/A"&"Hello World")&(Data!B5&"#N/A")),"x*",_xlfn.GROUPBY(Data!B1:B12,Data!B1:E12,LAMBDA(acc,acc),3),1)`: C1504 Excel <empty>, ours #VALUE!; cells (11): C1504, C1506, A1507:B1507, A1508:B1508, D1508, D1512, C1513, B1514, D1514
- gen-m7/9bb61f6ae34c7e20.case.json A136 `=REGEXEXTRACT(("?b"&("?b"&Data!D12)),"^.",TOROW(Data!C10:D11),0)`: B136 Excel <empty>, ours #VALUE!; cells (2): B136, D136

## Acceptance criteria
- [ ] A spilled undecoded rich error is pending (not scored) after a fill; a tools or harness test covers it.
- [ ] The 64 lines listed under Context leave eval/baseline/gen-m7.txt with 0 regressions (or each that Excel really fills differently from ours, once read, moves to a task of its own).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Decoding the rich error itself (excel-graph gives no error type).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.
