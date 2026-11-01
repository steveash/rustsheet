---
id: T-2646
title: "VALUE reads a mixed fraction, a sign before a space and a dash date with a one-digit year (probes/eng-inumber-date-boundary, 4 cells)"
milestone: M9
lane: functions-text
status: done
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
VALUE (and every reader built on `value_text_number`) reads `"- 5"`, `"3 1/2"` and `"1-1-1"` as Excel does.

## Context
T-3151 landed the mixed fraction (`mixed_fraction` in cast.rs; A22 and B22 pass): what is left is `"- 5"`, `"1-1-1"`
and probing the mixed-fraction edges below, plus an improper fraction (`"1 3/2"`) and many-digit parts.
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

## Outcome
`VALUE("- 5")`, mixed fractions with a sign, `$` or `%`, and dash/slash dates with a one-digit year now read as Excel does
(`parse_number`, `mixed_fraction`, `parse_separated`). Edge probe probes/value-sign-space-mixed-fraction-dash-date
(35 cells, excel-graph) passes. Eval delta: probes/eng-inumber-date-boundary S!B21 and S!B23 left the baseline (2 known
failures fewer), 35 new passing cells, no regressions. Quirks recorded in docs/excel-semantics/text.md.
