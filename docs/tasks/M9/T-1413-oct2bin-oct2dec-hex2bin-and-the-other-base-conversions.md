---
id: T-1413
title: "OCT2BIN, OCT2DEC, HEX2BIN and the other base conversions read a blank cell as 0 (#NUM! today)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
OCT2BIN, OCT2DEC, OCT2HEX, HEX2BIN, HEX2DEC, HEX2OCT (and the BIN2* functions, per the probe) read a blank cell as 0, as
Excel does, instead of `#NUM!`.

## Context
T-1289's gen-m9 suite (excel-graph sidecars, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/d244411dd90f7fbd Calc!A15 `=(VLOOKUP(Data!C4,Data!D11:E12,2)&UPPER(OCT2BIN(Data!E5,2)))` with E5 blank: Excel
  "-0.000100" (the VLOOKUP's -0.0001 and "00"), engine `#NUM!`.
Engine today: `OCT2BIN(<blank>,2)`, `HEX2BIN(<blank>)` and `OCT2DEC(<blank>)` are `#NUM!`; `BIN2DEC(<blank>)` is 0 and
`DEC2BIN(<blank>)` "0" already. Typed empty text is a separate question: probe with the excel-probe skill each X2Y function of a
blank cell and of `""`, with and without places, before implementing (docs/excel-semantics/engineering.md has the
DECIMAL rule that all-blank text is 0, which may or may not carry over). Code:
crates/rustsheet-engine/src/functions/engineering/number_basis.rs; find the shared digit reader and fix it once.

## Acceptance criteria
- [ ] gen-m9/d244411dd90f7fbd A15 passes in `./target/release/rustsheet-eval show gen-m9/<case>.case.json`, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] The blank/empty-text probe cells pass; engineering.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
