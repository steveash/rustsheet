---
id: T-1659
title: "Lotus-style dotted ranges: SUM(A1.B2) and ROWS(A1..B2) read A1:B2"
milestone: M2
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A dotted A1 range (`A1.B2`, `A1..B2`, `B2.A1`) in a formula is the range `A1:B2`, as in Excel, and FORMULATEXT shows it with `:`.

## Context
Found by T-1652 (docs/excel-semantics/references.md "R1C1-shaped sheet and defined names"). Excel for the web: `SUM(A1.B2)` is 7 and FORMULATEXT `=SUM(A1:B2)`; `ROWS(A1..B2)` is 2; `SUM(B2.A1)` is `=SUM(A1:B2)` (probes/lotus-dot-range). A defined name shaped like that is shadowed by the range: with `R1.C1` defined, `=R1.C1*1` is #SPILL! and `=C1.C1*1` is 0 (probes/r1c1-name-r1-dot-c1, -r1-dotdot-c1, -c1-c1-dot). Probe first: sheet-qualified forms (`Data!A1.B2`), `$` cells, whole rows or columns, a name as the second part, and whether the shape after the second cell matters (sheet quoting accepts a trailing `.anything`). Code: the A1 lexer (expressions/lexer).

## Acceptance criteria
- [x] probes/lotus-dot-range and the three r1c1-name dotted cases pass; the probed forms are recorded in the semantics entry.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`A1.B2`, `A1..B2`, `B2.A1`, `a1.b2` and `R1.C1` in an A1 formula are the range (`A1:B2`, `C1:R1`),
printed with `:`, over a defined name spelled so; a LET or LAMBDA name spelled so stays the
variable within its call (`LET(R1.C1,5,R1.C1*2)` is 10). The glued form lexes as one identifier
and the parser turns it into the range unless an enclosing LET/LAMBDA binds it
(`dotted_range_operand`, `bind_dotted_name`); forms with `$` or whitespace are read by the lexer
(`consume_dotted_corner`). Probed 16 more forms (probes/lotus-dot-range-*, 11 cases filled; 5
forms make Graph reject the file and have no case), recorded in references.md.
Eval: probes known failures 2282 -> 2274 (9 cells fixed: lotus-dot-range D1-D6 and the three
r1c1-name dotted A1 cells; 11 new cases, 34 of 35 cells pass). The one new failure,
lotus-dot-range-func-second E2 (ours lowercases the unknown call `A1.INDEX`), is T-1719.
