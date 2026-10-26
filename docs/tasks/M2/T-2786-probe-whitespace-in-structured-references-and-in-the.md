---
id: T-2786
title: "Probe whitespace in structured references and in the formula text Excel saves"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the two whitespace questions SPEC-0023 leaves open: whitespace inside a structured
reference, and whether the `<f>` text Excel saves keeps typed whitespace.

## Context
SPEC-0023 "Unknown". Probe runs with no operand beside them (`=NOW( )`, `=IF(1, ,2)`, `=SUM(1, )`), re-probe
the refused `=SUM (1)` and record Graph's error text, and probe typed `=SUM(T[ Col])`, `=SUM(T[[#This Row], [Col]])`, `=SUM(T[ [Col] ])` and
similar on a real table, with FORMULATEXT. For the saved text, enter `=1 + 2`, `=SUM( 1 )` and a line-feed formula
through Graph and read the `<f>` of the workbook Excel saves (the oracle already downloads it; record the text
in docs/excel-semantics/xlsx-storage.md, as SPEC-0007 did for parentheses).

## Acceptance criteria
- [ ] Probe cases with excel-graph oracle values; findings in docs/excel-semantics/ (syntax.md, xlsx-storage.md).
- [ ] An implementation task filed with `make new-task` for each answer that differs from what SPEC-0023 step 1
      prints, and SPEC-0023's "Unknown" updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the findings.

## Outcome
Probe `probes/t2786-whitespace-structured-refs-runs` (excel-graph) answers: runs beside a missing argument are kept
(`=NOW( )`, `=IF(1, ,2)`, `=SUM(1, )`); `=SUM (1)` and `=SUM( )` are refused (400 InvalidArgument); structured references
refuse a space in a bare specifier and normalize `T[ [Col] ]` to `T[ Col ]`; Excel saves typed whitespace in `<f>`, a
line feed as raw CR LF bytes (read back as LF). Findings in syntax.md and xlsx-storage.md; SPEC-0023 "Unknown" answered. Filed T-2799 (missing
arguments), T-2801 (structured-reference whitespace), T-2800 (this-row shapes, which Graph's calculated-column
fill masked). Eval delta: none (probe cases have no expectations).
