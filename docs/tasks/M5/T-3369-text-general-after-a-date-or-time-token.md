---
id: T-3369
title: "TEXT: General after a date or time token prints Excel's value, not #VALUE! (probes/text-format-general-company A5)"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3368]
campaign: semantics
---

## Goal
TEXT prints what Excel prints for `General` after a date or time token instead of `#VALUE!`, so
probes/text-format-general-company A5 (`TEXT(5,"d General")` = "5 0.5") and the T-3368 probe cells pass.

## Context
- Gap triage 2026-10-31 (`make eval` on 454f580b4): probes/text-format-general-company Sheet1!A5,
  `=TEXT(5,"d General")`: Excel "5 0.5" (excel-graph, 2026-10-31, T-1614), ours `#VALUE!` (TypeMismatch). It is
  the one visibly-off cell no open task owned; it holds the M10 probe row (docs/milestones.md) open.
- Ours: `Parser::parse_part` (crates/rustsheet-engine/src/formatter/parser.rs, the `general_count` check) returns an
  error for `General` in a date section (`is_date`).
- Semantics: docs/excel-semantics/number-formats.md, "General" bullet (oddity, unimplemented). The value General
  prints is not explained by the one cell, so T-3368 (sonnet) probes it first; implement the rule its cells show.
  Excel is right even if the rule looks arbitrary: match the cells, do not tidy it.

Cells:
- probes/text-format-general-company.case.json (1): Sheet1!A5
- the T-3368 probe case's failing cells (named in T-3368's Outcome).

## Visible-parity stage
`semantics`: the cell is a wrong type (`#VALUE!` against text). Target: probes/text-format-general-company A5 and
every T-3368 probe cell give Excel's text.

## Acceptance criteria
- [ ] probes/text-format-general-company Sheet1!A5 passes: its known-failure line disappears from
      eval/baseline/probes.txt, and so do the T-3368 probe case's lines, with no new known-failure lines in any
      baseline (`make eval-baseline`, 0 regressions).
- [ ] The other 15 cells of probes/text-format-general-company still pass.
- [ ] docs/excel-semantics/number-formats.md's "General" bullet names where the rule is implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `General` in the text (4th) section.
- Any T-3368 cell whose rule stays unexplained after a time-boxed fit (decision 0056): name it in a follow-up task.
