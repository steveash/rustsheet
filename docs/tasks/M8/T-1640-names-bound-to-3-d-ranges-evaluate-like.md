---
id: T-1640
title: "Names bound to 3-D ranges evaluate like the 3-D reference, not #REF! (gen-m8a 3d-names, 3d-misuse)"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A defined name bound to a 3-D range (`span3` = `=Jan:Mar!$A$1:$B$2`) behaves like the 3-D reference written inline: aggregates (SUM, COUNT, STDEV.S, ...) and LAMBDA arguments accept it, and misuse gives Excel's errors.

## Context
gen-m8a (T-1606) case 8982982d63e058ed (3d-names, 108/280) and the name probes of 3d-misuse b221d72cc501675a: every use of `span3`, `span_all` and `span_calc` is `#REF!` in the engine (COUNT gives 0). Excel: `COUNT(span3)` 10, `ROWS(span3)`, `INDEX(span3,1,1)`, `span3+100` and a bare `=span3` are `#VALUE!`; ISREF(span3) is FALSE; and IFERROR(span3,x) is `#VALUE!` (not x), though IFERROR(span_all,x) (a one-cell 3-D name) is x. docs/excel-semantics/references.md "3-D references in defined names".

## Acceptance criteria
- [ ] The 3-D name probes of gen-m8a 3d-names and 3d-misuse pass, with no regression elsewhere.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
