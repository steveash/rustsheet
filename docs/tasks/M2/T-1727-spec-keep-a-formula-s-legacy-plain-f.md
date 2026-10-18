---
id: T-1727
title: "Spec: keep a formula's legacy (plain <f>) reading past the @ rewrite"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A spec in docs/specs/ for how the engine remembers that a formula is legacy (a plain `<f>`, or `Input::LegacyFormula`) where Excel's legacy result differs from the dynamic one for the same text, so a legacy `=N(IF(1,{2;3}))` can be `#VALUE!` while the dynamic one spills 2, 3.

## Context
Today a legacy formula is stored as its dynamic twin with the `@`s `add_implicit_intersection`
adds (legacy_formula.rs, xlsx import); nothing else records that it was legacy, and the xlsx export
re-derives the plain `<f>` vs `cm="1"` choice from the text (`legacy_reading_intersects`,
rustsheet-xlsx export/worksheets.rs). That is enough while every legacy difference shows as an
`@`. It is not for N and T over an array-valued IF, CHOOSE, INDEX or IFS: Excel adds no `@`
(FORMULATEXT `=N(IF(1,{2;3}))`), yet the legacy cell is `#VALUE!`, also nested
(`=ABS(N(IF(1,{2;3})))`, `=N(IF(1,{2;3}))+0`, `=N(IF(1,{2}))`), while `=SUM(N(IF(1,{2;3})))` is 5 and
`=@IF(1,N(IF(1,{2;3})))` 2. Evidence: probes/legacy-n-t-reference-args, probes/legacy-array-result
A10/G10/I10, probes/n-array-lift D4 (excel-graph, 2026-10-16, T-1713); docs/excel-semantics/
references.md "Legacy N and T over a reference-returning call". The same marker also decides the
export: a dynamic `=N(IF(1,{2;3}))` saved as a plain `<f>` today would read back legacy in Excel.

The spec should weigh at least: a per-cell legacy flag in storage (what Excel keeps: `cm` on the
cell), versus a hidden AST node with an R1C1 storage token that FORMULATEXT and the export do not
print; how copy/move, fill, the API's `Input::LegacyFormula`, the xlsx import/export and the Python
binding carry it; where evaluation reads it (an N/T call whose array-valued argument came from a
reference-class call, outside an array context or `@`); memory cost per cell; the T-1717 general
`@` rewrite it must coexist with.

## Acceptance criteria
- [ ] docs/specs/SPEC-XXXX (from the TEMPLATE) with the chosen design, the alternatives and the implementation tasks it cuts (T-1713 resumes on it).
- [ ] Reviewers (architecture, performance) approve.

## Out of scope
- Implementing it.
