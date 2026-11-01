---
id: T-3555
title: "Gate A slice 3e: reference tables for ROUND, ROUNDUP and ROUNDDOWN, with the math read at the written decimal"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-3358]
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
- [x] One definition each for ROUND, ROUNDUP and ROUNDDOWN. Their unit tests reproduce the numbers.md values
      `ROUND(2.675,2)` = 2.68, `ROUND(1.005,2)` = 1.01, `ROUND(-2.5,0)` = -3, `ROUNDDOWN(123456789.123,-5)` =
      123400000 and `ROUND(1.5,-400)` = 0, and they treat `ROUNDUP(1.5,-400)` as a domain error.
- [x] The decimal-math rule is recorded (decision record or SPEC-0029 note) and cited in the definitions.
- [x] `eval/accuracy/tables/{ROUND,ROUNDUP,ROUNDDOWN}/s1.{case,math,oracle}.json` are generated, filled and committed.
      `rustsheet-eval accuracy` reports `missing tables` 3 lower than before the change. The Outcome names every
      failing point with its owner task, in `known-failures.tsv` under T-3379's rules, and lists every point where
      Excel is more than G = 1e-4 off the math. The expected number of such points is 0; any point listed is
      evidence against the rule.
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed (no sidecar changed; see the Outcome). `make eval` shows no
      cell of `probes/round-family` or `probes/roundup-wrapped-scaling` leaving the R1(c) envelope.
- [x] `make check` green; reviewers approve.

## Out of scope
- MROUND, TRUNC, INT, CEILING and FLOOR (M5; T-3551, filed with T-3550 in #2656, classifies them).
- Engine changes for failing points (file them as tasks with the cells).
- Arguments computed by a formula, which can carry a 16th digit (numbers.md's stage-1 tie). Eval suites own that
  case, not the tables.

## Outcome
- Definitions: `tools/src/rustsheet_tools/accuracy/defs/rounding.py`. The math is the argument's 15-digit decimal,
  rounded at `num_digits` (decision 0223, cited in the module docstring). `literal.decimal15` is the one 15-digit cut,
  and `round15` now uses it too. Unit tests are in `tools/tests/test_accuracy_rounding.py`, with the numbers.md values
  the acceptance lists. `ROUNDUP(1.5,-400)` is a math domain error.
- Tables: `eval/accuracy/tables/{ROUND,ROUNDUP,ROUNDDOWN}/s1`, 120 points each (48/18/18/24/12 by stratum), filled
  from excel-graph on 2026-11-01. `rustsheet-eval accuracy` on the head merged with main: 2362 points in 13 tables, 0 failing,
  0 table defects. Missing tables went down by 3: 310 to 307 before the merge with main, 305 after it (main had 310, not
  323, when this task started). No known-failures lines: every new point is
  in the envelope. ROUNDUP's 5 domain-error points are all `#NUM!` in Excel.
- Excel is more than G = 1e-4 off the math (relative, with no floor) at 5 points. Decision 0223 lists them: ROUND
  A73, ROUNDDOWN A81 and A84, and ROUNDUP A74 and A76. All five are the subnormal flush of T-2756, not the decimal
  rule. The engine matches Excel to the bit there, and the 1e-300 floor keeps each in the envelope. A first count
  in this Outcome said 0 because it applied that floor; the Excel-fidelity review caught it.
- ROUND/s1 A61, `ROUND(4.91499999999999E-06,8)`: Excel gives 4.9100000000000004E-06, 1 ulp above the double
  nearest 4.91E-06, which the engine returns. It is in the envelope and below the reproducibility floor. I recorded
  it in numbers.md as an observation against "return the nearest double".
- The draws keep `num_digits` within ±400. The first ROUNDUP draw went to ±65,535 and hit a table defect:
  `ROUNDUP(6.4421516092447E+265,-43376)` is the value in Excel, not `#NUM!`. That contradicts the wrap rule, so I
  recorded it in numbers.md ("ROUNDUP of a huge value at a huge negative num_digits", UNVERIFIED) and filed T-3627 to
  probe it. Then I replaced the batch before commit. The committed ROUNDUP s1 Calc!A111 (the same value at -320, `#NUM!`)
  narrows the observation, and T-3627 cites it.
- A `num_digits` beyond ±400, or a 16th-digit tie, has no math value (`NoValueError`). So `rustsheet-accuracy math`
  never gives an eval cell a math value from a rule that does not cover it.
- `rustsheet-accuracy math --suite probes-numeric` changed no committed sidecar: no probes-numeric cell is a
  one-call ROUND, ROUNDUP or ROUNDDOWN with literal arguments. It also wrote a sidecar for
  `t3567-r5-fdf1-hold` (F.DIST, not this task's), which I left out. probes/round-family (53) and
  probes/roundup-wrapped-scaling (480) still pass in full (eval/baseline/probes.txt `C` lines), so no cell leaves the
  R1(c) envelope.
