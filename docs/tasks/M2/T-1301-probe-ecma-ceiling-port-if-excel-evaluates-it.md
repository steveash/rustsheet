---
id: T-1301
title: "Probe ECMA.CEILING, port if Excel evaluates it as CEILING (ironcalc 8a82396a)"
milestone: M2
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`ECMA.CEILING` parses, round-trips and evaluates exactly as excel-graph does, or is shown to be unsupported there.

## Context
Upstream IronCalc 8a82396a (2026-09-23, first listed 2026-10-10) added `ECMA.CEILING`, evaluated
like the two-argument `CEILING`. The function is not in `eval/catalog/excel-functions.tsv`, and
upstream is not evidence. Probe it with the excel-probe skill first: positive and negative numbers
and significances, a zero significance, mixed signs (CEILING is #NUM! there), the one-argument form,
text and boolean arguments, and how FORMULATEXT prints it (any `_xlfn.` prefix on save). Port only
what excel-graph confirms (`Ported-From: ironcalc@8a82396afbababfdac296c70b02ac9bdb456db98` if code
is taken), reusing our CEILING implementation rather than copying it. Record the rule in
docs/excel-semantics/.

## Acceptance criteria
- [ ] A probe case covering the forms above is oracle-filled.
- [ ] The engine matches every cell (or `ECMA.CEILING` is left unknown if Excel rejects it); findings recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- CEILING.MATH, CEILING.PRECISE, ISO.CEILING and the FLOOR family.
