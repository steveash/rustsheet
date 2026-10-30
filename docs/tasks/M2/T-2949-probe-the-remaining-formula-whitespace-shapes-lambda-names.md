---
id: T-2949
title: "Probe the remaining formula whitespace shapes (LAMBDA names, LET names, group spacing, tab/NBSP in stored text)"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the whitespace shapes T-2786 left unprobed.

## Context
From the T-2781 review and SPEC-0023: `B1:B2` LF ` B2` (an intersection whose separator holds a line feed), whitespace around LAMBDA parameter names (`=LAMBDA( x , x)(1)`), the second LET name, `=( (A1) )`, `=- 5 %`, and leading TAB, LF or NBSP in a file's `<f>`. Probe with FORMULATEXT; refused shapes are recorded as Graph's refusals.

## Acceptance criteria
- [ ] Probe cases with excel-graph values; findings in syntax.md; implementation tasks filed for differences.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the findings.

## Outcome
Probed 25 accepted shapes as one-formula cases plus the refused shapes (TAB, NBSP, even-arity LET), which are in syntax.md and not kept as cases. Findings in syntax.md "Remaining whitespace shapes". Follow-up: T-3169 (implement). The baseline gained 25 case lines and 6 known failures from `make eval-baseline` (new probe coverage the engine does not match yet; T-3169 shrinks them).
