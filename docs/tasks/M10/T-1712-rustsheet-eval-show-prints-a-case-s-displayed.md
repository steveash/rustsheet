---
id: T-1712
title: "rustsheet-eval show prints a case's displayed-text verdicts"
milestone: M10
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`rustsheet-eval show <case>` (and `make eval-show`) prints a case's displayed-text verdicts beside its value verdicts, so a text failure can be inspected per case.

## Context
T-1610 added `CaseResult.text` (crates/rustsheet-eval/src/text_expect.rs, decision 0068); only report.md's "Displayed text" section shows it. Raised by the T-1610 excel-fidelity review. Example: probes/displayed-text-formats S!A17, A21, B4.

## Acceptance criteria
- [ ] `show` prints each text cell that fails (Excel text, ours) and a text pass count for the case; a unit test covers the output.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
