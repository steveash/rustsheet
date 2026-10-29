---
id: T-3119
title: "Probe dollar-outside-parentheses number text, bare <= and >= criteria against empty strings, and USDOLLAR"
milestone: M5
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence settles three behaviours that another engine's documents describe and RustSheet has
no probe for, and each finding is recorded in docs/excel-semantics/.

## Context
Found by T-0496 (docs/research/gridcraft-comparison-2026-10-29.md, "Leads without RustSheet
evidence"). gridcraft's own docs state these as observed in Excel; that is a lead, not evidence
(CLAUDE.md directive 1), so each needs a probe before any engine change.
- Number text with a currency sign outside the parentheses, or a percent sign after them.
  docs/excel-semantics/coercion.md "Parenthesised number text is negative" covers `($1)` and
  `(1%)`, not `$(1)`, `$ (1)`, `(1)%`, `$1%` or `$(1)%`.
- COUNTIF, SUMIF and COUNTIFS with a bare `"<="` or `">="` criterion over a range holding empty
  text (`=""` results), blanks, zero and text. docs/excel-semantics/criteria.md probes the bare
  `"="` and `"<>"` only.
- USDOLLAR: whether Excel evaluates it as a function (it is not on Microsoft's alphabetical list,
  so eval/catalog/excel-functions.tsv does not have it), and if so what it returns next to DOLLAR.

Write the probe formulas from this paraphrase with built-in pure functions only (excel-probe skill).
Suggested cells: `0+"$(1)"`, `0+"$ (1)"`, `0+"$(1,234)"`, `0+"(1)%"`, `0+"$1%"`, `0+"$(1)%"`,
`VALUE("$(1)")`, `ISNUMBER` of each as a text cell; `COUNTIF(A1:A6,"<=")`, `COUNTIF(A1:A6,">=")`,
`SUMIF(A1:A6,"<=",B1:B6)` over A1:A6 = blank, `=""`, 0, "a", 1, FALSE; `USDOLLAR(1234.567)`,
`USDOLLAR(-1234.567,1)`, `ISERROR(USDOLLAR(1))`.

## Acceptance criteria
- [ ] One probe case under eval/suites/probes/cases/ (tag M5) with every cell above, filled by
      excel-graph (or left pending with the task blocked on the oracle, excel-probe skill).
- [ ] docs/excel-semantics/coercion.md, criteria.md and text.md record each result as a rule with
      its probe cells; a mismatch with the engine is filed as a task in the owning lane.
- [ ] If Excel evaluates USDOLLAR, a task to add it to eval/catalog is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes: file them as tasks in the owning lanes.
