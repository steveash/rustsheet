---
id: T-1640
title: "Names bound to 3-D ranges evaluate like the 3-D reference, not #REF! (gen-m8a 3d-names, 3d-misuse)"
milestone: M8
lane: engine
status: done
priority: P1
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
A defined name bound to a 3-D range (`span3` = `=Jan:Mar!$A$1:$B$2`) behaves like the 3-D reference written inline: aggregates (SUM, COUNT, STDEV.S, ...) and LAMBDA arguments accept it, and misuse gives Excel's errors.

## Context
gen-m8a (T-1606) case 8982982d63e058ed (3d-names, 108/280) and the name probes of 3d-misuse b221d72cc501675a: every use of `span3` and `span_calc` is `#REF!` in the engine (COUNT gives 0). Excel: `COUNT(span3)` 10, `ROWS(span3)`, `INDEX(span3,1,1)`, `span3+100` and a bare `=span3` are `#VALUE!`; ISREF(span3) is FALSE; and IFERROR(span3,x) is `#VALUE!`. Leave `span_all` alone: its span is quoted on one side only (`=Jan:'My Data'!$A$1`), Excel reads it as `#NAME?`, and the engine already matches. docs/excel-semantics/references.md "3-D references in defined names".

## Acceptance criteria
- [x] The 3-D name probes of gen-m8a 3d-names and 3d-misuse pass, with no regression elsewhere.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083: wrong values here are visible to anyone using defined names or tables (gen-m8a 3d-names and 3d-misuse), so this task is in the campaign. Its acceptance criteria are already the visible-parity stage.

## Outcome
A name whose body is a multi-sheet 3-D span now counts as possibly multi-area, so aggregates and
LAMBDA arguments read it sheet by sheet like the inline span; as a value it is `#VALUE!` (not the
inline `#REF!`), and IFERROR passes that `#VALUE!` through instead of taking its fallback. Eval:
gen-m8a (not gated) 3372 → 3577 of 3577 cells (+205); no gate suite changes. Unit test
`a_name_bound_to_a_span_reads_like_the_span`.
