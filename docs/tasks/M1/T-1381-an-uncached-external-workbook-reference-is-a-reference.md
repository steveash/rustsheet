---
id: T-1381
title: "An uncached external-workbook reference is a reference: ISREF, ROWS, unquoted [book]Sheet! prefix"
milestone: M1
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: [T-0124]
---

## Goal
A reference into another workbook that holds no cached data behaves as Excel's: it is a reference
of the written shape whose every cell reads `#BLOCKED!`, and the unquoted `[book]Sheet!ref` prefix
parses.

## Context
T-0124 made the value `#BLOCKED!` by special-casing the error of a reference on an unknown sheet
(`wrong_sheet_error`, a prefix holding `[`). The parser still builds `WrongReferenceKind` /
`WrongRangeKind`, so the reference-ness is lost. Known failures in
probes/external-ref-no-link (excel-graph, no externalLink part in the workbook):
- Calc!A4 `=ISREF('[other.xlsx]Sheet1'!A1)` → TRUE (we give FALSE)
- Calc!A10 `=ROWS('[other.xlsx]Sheet1'!A1:B3)` → 3 (we give `#BLOCKED!`)
- Calc!A7 `=SUM([other.xlsx]Sheet1!A1:B1)` → `#BLOCKED!` (we give `#ERROR!`: the lexer rejects the
  unquoted bracket prefix)
COUNT over it is 0 and ERROR.TYPE 11 (A9, A3 pass). Rule: docs/excel-semantics/references.md
"External reference with no cached data is `#BLOCKED!`". Likely shape: a dedicated node (or a
sheet-less reference) that ISREF/ROWS/COLUMNS/AREAS accept and that evaluates as an all-`#BLOCKED!`
area; probe COLUMNS, AREAS, INDEX, OFFSET on it first, and a 3-D prefix naming another workbook
(`'[b.xlsx]S1:S2'!A1`, `#BLOCKED!` today, UNVERIFIED) (excel-probe).

## Acceptance criteria
- [x] probes/external-ref-no-link A4, A7, A10 leave eval/baseline/probes.txt with no new F lines.
- [x] The extra shapes probed are recorded in docs/excel-semantics/references.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Cached values of `[n]Sheet!ref` (T-1382). Opening other workbooks (directive 5).

## Outcome
`reference_operators::external_area` gives ISREF, ROWS, COLUMNS, ROW, COLUMN and AREAS the written
shape of a reference on one sheet of another workbook (its cells still read `#BLOCKED!`), and the
lexer reads the unquoted `[book]Sheet!ref`. New probe probes/external-ref-no-link-shapes (20 cells,
excel-graph) recorded in references.md. Eval: probes fixed 3 (external-ref-no-link A4, A7, A10;
that case now passes 10/10), 8 new-coverage failures in the new case (INDEX, OFFSET, union,
intersection, COUNTA, spill), 0 regressions. Follow-up: T-1454.
