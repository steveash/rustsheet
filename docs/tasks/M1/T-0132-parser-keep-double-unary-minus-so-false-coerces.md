---
id: T-0132
title: "Parser: keep double unary minus so --FALSE coerces to 0"
milestone: M1
lane: engine
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`--x` coerces `x` to a number like Excel (`--FALSE` = 0, `--""` = `#VALUE!`), instead of returning
`x` unchanged.

## Context
gen-m1 (T-0109 triage, oracle excel-graph 2026-10-05): 3 failing cells.
- `gen-m1/6462bf910ef9f28f` A49 `--Data!C4` (C4 = FALSE): Excel 0, engine FALSE.
- `gen-m1/6462bf910ef9f28f` A84 `--Data!$A11` (A11 = ""): Excel `#VALUE!`, engine "".
- `gen-m1/57fe331cb7611dbf` A167 `--("">1.5)`: Excel 1, engine TRUE.
Passing neighbours that must stay green: `--100` (6462… A17), `--Data!D7` (A144) = 100,
`--'Q''1'!E9%` (57fe… A210) = `#VALUE!`.
Semantics: docs/excel-semantics/coercion.md "Double unary minus coerces to a number".

Root cause: `expressions/parser/mod.rs` `parse_power` folds a run of `+`/`-` prefixes into one sign,
so an even number of minus signs emits no `UnaryKind` node at all and nothing coerces. Keep the
coercion, either with nested `UnaryKind::Minus` nodes or with one minus node per pair. Check
`stringify.rs` (`UnaryKind` → `-(...)` when the operand is itself unary): `=--A1` must
round-trip as `--A1`, not `-(-A1)`. Also check the array-literal path in `parse_array_row`, which
matches `UnaryKind { Minus, NumberKind }`.

## Acceptance criteria
- [x] The 3 cells above disappear from eval/baseline/gen-m1.txt with no new F lines in any suite.
- [x] Unit test: `=--A1` round-trips through stringify unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
- Unary plus (`=+FALSE`): no gen-m1 failure.

## Outcome
Done (2026-10-05). Ported upstream `9b1ae3fd` (`Ported-From: ironcalc@9b1ae3fd`): `parse_power`
emits one `UnaryKind::Minus` per minus sign (still bounded by `MAX_FORMULA_DEPTH` via `grow`), unary
minus and percent evaluate through `apply_number_unary` (element-wise over ranges/arrays),
`stringify` prints `--x` without parentheses and parenthesises `-(A1=B1)` / `-(A1&B1)`, and
`move_formula` does the same. Upstream's unit tests (`test_unary_operators.rs`) came with it; its
Excel-authored `TEXT/MINUS_MINUS.xlsx` passes 72 of 73 cells but calc_tests workbooks must pass
whole, so it is added by T-0146 with the remaining cell (C12 `--"12:00"`). The upstream cast-to-time-of-day hunk is left to T-0146,
which reuses T-0910's `parse_clock_text`; the matching upstream assertions (`--"12:00"`, `"12:00"+1`,
`--"6 PM"`) were dropped from the ported test until then.
Eval: gen-m1 261 → 258 (the 3 cells), gen-m2 780 → 779, probes 136 → 134, 0 regressions. The nesting test now asserts a 10,000-minus run is a
too-deep parse error (not a stack overflow) and a run under the cap parses.
