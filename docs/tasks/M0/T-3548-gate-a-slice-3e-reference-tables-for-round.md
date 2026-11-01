---
id: T-3548
title: "Gate A slice 3e: reference tables for ROUND, ROUNDUP and ROUNDDOWN, with the math read at the written decimal"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
ROUND, ROUNDUP and ROUNDDOWN, three of the ten M2 functions Gate A counts as missing, each get a committed,
oracle-filled `s1` table with a cited mpmath definition, and `make check` runs them. The "math" of a decimal rounding
function is defined so that Excel's documented decimal behavior is not counted as Excel being far from the math.

## Context
- Gate A cannot hold while a catalog function with `scope: in` and `returns: number` has no table and no `exempt.tsv`
  line (SPEC-0029 "Gate A holds"). `rustsheet-eval accuracy` on main as of 8ee20308 prints `missing tables 323`.
  Ten of them are M2 functions: ABS, AVERAGE, MAX, MIN, ROUND, ROUNDDOWN, ROUNDUP, SUM, SUMIF, SUMIFS. M2 is the current
  milestone. T-3358 and its slice tasks T-3529 and T-3535 (filed on T-3358's branch, open PR #2648) cover only decision 0188's 18 functions, so nothing
  owns these. T-3549 covers SUM and AVERAGE, and T-3550 (open PR #2656) exempts ABS, MIN, MAX, SUMIF and SUMIFS.
- The design question this task settles: SPEC-0029 ("Excel behavior to match") evaluates the math "at the double
  nearest the written decimal". For ROUND that turns the evidenced decimal rule into a false "Excel is far from the
  math". `ROUND(2.675,2)` is 2.68 in Excel, but the exact double 2.67499999999999982236431605997495353221893310546875
  rounds to 2.67, a relative gap of 3.7e-3, more than decision 0188 R3's G = 1e-4. docs/excel-semantics/numbers.md,
  "ROUND family rounds in two decimal stages" (VERIFIED, probes/round-family A1–A53, probes/precision-and-rounding
  A6–A8), says Excel first cuts the binary value to 15 significant digits and then rounds that decimal. A table literal
  has at most 15 significant digits (`accuracy/literal.py`), so that cut gives back the written decimal. So the
  definition should round the 15-digit decimal of the argument (half away from zero for ROUND, away from zero for
  ROUNDUP, toward zero for ROUNDDOWN). Record this as a decision record (preferred, since it changes what "the math" means for these functions), and cite it from the definition's docstring. Do not use an R3 mark: Excel is not inaccurate here.
- Quirks the points will meet, all within the envelope of a correct engine:
  - ROUNDUP adds the unit in binary (numbers.md "ROUNDUP adds the unit in binary"). For example,
    `ROUNDUP(123.456,2)` is 123.46000000000001, about 1 ulp from the decimal math, so it is well inside ε.
  - Huge `num_digits` wrap (numbers.md "ROUND family: huge num_digits wrap", "ROUND family with huge num_digits"):
    `ROUND(1.5,400)` is 1.5, `ROUND(1.5,-400)` is 0, and `ROUNDUP(1.5,-400)` is `#NUM!`. The domain and extreme
    ranges of `num_digits` cite these entries. Where the math overflows, the point is a domain error and Excel's error
    is the target.
  - The tie at the 16th digit cannot occur: a 15-digit literal has no 16th digit. Say so in the docstring rather
    than drawing it.
- Tools: write a `Definition` under `tools/src/rustsheet_tools/accuracy/defs/` (one family file, e.g. `rounding.py`),
  then run `cd tools && uv run rustsheet-accuracy generate ROUND` (and the same for ROUNDUP and ROUNDDOWN),
  `uv run rustsheet-oracle fill -b excel-graph --suite accuracy` and `target/release/rustsheet-eval accuracy`. Two
  numeric parameters give 120 points per function (SPEC-0029 "Counts"). After that, rerun
  `uv run rustsheet-accuracy math --suite probes-numeric` (T-3380) and commit the sidecars.
- Engine code: `crates/rustsheet-engine/src/decimal_round.rs` (`round_decimal`, T-0106, T-0268).

## Acceptance criteria
- [ ] One definition each for ROUND, ROUNDUP and ROUNDDOWN. Their unit tests reproduce the numbers.md values
      `ROUND(2.675,2)` = 2.68, `ROUND(1.005,2)` = 1.01, `ROUND(-2.5,0)` = -3, `ROUNDDOWN(123456789.123,-5)` =
      123400000 and `ROUND(1.5,-400)` = 0, and they treat `ROUNDUP(1.5,-400)` as a domain error.
- [ ] The decimal-math rule is recorded (decision record or SPEC-0029 note) and cited in the definitions.
- [ ] `eval/accuracy/tables/{ROUND,ROUNDUP,ROUNDDOWN}/s1.{case,math,oracle}.json` are generated, filled and committed.
      `rustsheet-eval accuracy` reports `missing tables` 3 lower than before the change. The Outcome names every
      failing point with its owner task, in `known-failures.tsv` under T-3379's rules, and lists every point where
      Excel is more than G = 1e-4 off the math. The expected number of such points is 0; any point listed is
      evidence against the rule.
- [ ] The probes-numeric math sidecars are regenerated with the new definitions and committed. `make eval` shows no
      cell of `probes/round-family` or `probes/roundup-wrapped-scaling` leaving the R1(c) envelope.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MROUND, TRUNC, INT, CEILING and FLOOR (M5; T-3551, filed with T-3550 in #2656, classifies them).
- Engine changes for failing points (file them as tasks with the cells).
- Arguments computed by a formula, which can carry a 16th digit (numbers.md's stage-1 tie). Eval suites own that
  case, not the tables.
