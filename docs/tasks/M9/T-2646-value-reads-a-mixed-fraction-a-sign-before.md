---
id: T-2646
title: "VALUE reads a mixed fraction, a sign before a space and a dash date with a one-digit year (probes/eng-inumber-date-boundary, 4 cells)"
milestone: M9
lane: functions-text
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
VALUE (and every reader built on `value_text_number`) reads `"- 5"`, `"3 1/2"` and `"1-1-1"` as Excel does.

## Context
Found in T-2541's boundary probe probes/eng-inumber-date-boundary (excel-graph, 2026-10-25): `VALUE("- 5")` is -5 (B21),
`VALUE("3 1/2")` 3.5 (B22; so `IMABS("3 1/2")` is 3.5, A22), `VALUE("1-1-1")` 36892, 2001-01-01 (B23); ours `#VALUE!` for
all three. Record each in docs/excel-semantics/text.md (or coercion.md where number coercion shares the rule) and probe
the edges first (excel-probe skill): whether `"- 5"` also reads in `="- 5"+0`, `"3 1/2"` against `"-3 1/2"`, `"3  1/2"`,
`"3 1/0"`, and `"1-1-1"` against `"1/1/1"`, `"13-1-1"`. Code: `value_text_number` in functions/text/common.rs and
the number-coercion grammar it calls.

## Acceptance criteria
- [ ] probes/eng-inumber-date-boundary S!A22, B21, B22, B23 pass and leave eval/baseline/probes.txt (`make eval-baseline`), no new known-failure lines.
- [ ] A probe of the edges above is oracle-filled and passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IM* reading of other text (T-2541, T-2641).
