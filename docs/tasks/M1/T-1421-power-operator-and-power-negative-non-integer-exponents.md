---
id: T-1421
title: "Power operator and POWER: negative non-integer exponents (gen-m1 3^-0.5, 0.5^-0.5; split from T-0221)"
milestone: M1
lane: engine
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`^` and POWER return Excel's double when the exponent is negative and not an integer, so the
last two gen-m1 failing cells pass and gen-m1 reaches 100% (an M1 exit criterion).

## Context
These are the last 2 failing gen-m1 cells (eval/baseline/gen-m1.txt, 2026-10-12). Both are
NumericUlp, and the only operation in each is `^` (`make eval-show CASE=gen-m1/6462bf910ef9f28f.case.json`,
oracle excel-graph):
- `gen-m1/6462bf910ef9f28f` Calc!A41 `=3^-0.5`: Excel 0.5773502691896258, engine 0.5773502691896257.
- `gen-m1/6462bf910ef9f28f` Calc!A207 `=0.5^-0.5`: Excel 1.414213562373095, engine 1.4142135623730951
  (the correctly rounded value).

Split from T-0221 (M2, opus) on 2026-10-12. T-0221 owns all non-integer exponents and says to
"take those first". Being M2, though, it is ordered after every M1 engine task in `next-task`
(scripts/tasks.py sorts by milestone first), so the last M1 gen-m1 blocker would wait behind P3
polish work. T-1302 (done) implemented integer exponents (right-to-left square-and-multiply).
This task covers only exponents that are negative and not integers.

Lead (a hypothesis from T-0221, not verified): for a negative exponent, Excel's value is
`1/(x^|y|)`, where `x^|y|` is the engine's existing positive-exponent result. That gives
`1/(3^0.5)` = 0.5773502691896258, `1/(0.5^0.5)` = 1.414213562373095 and, for gen-m2
`1b6ceed7489e0800` Calc!A62 `=(2^(-0.5))`, `1/(2^0.5)` = 0.7071067811865475. All three match Excel,
and correctly rounded `powf` misses all three. One more data point to test the lead against:
gen-m9/b5352ebdb9c5f1a1 Calc!A16 (27^-2.5) is Excel 0.0002639918926335737, engine 0.00026399189263357376.
Positive non-integer exponents (for example gen-m2 `100^0.3`) are not explained by this lead
and stay in T-0221.

Before you implement, confirm the rule with an excel-probe sweep (excel-probe skill). Cover
negative non-integer exponents over several bases (below 1, above 1, near 1, very large and very
small), exponents like -0.5, -1.5, -2.5, -1/3 and -0.3, and negative bases with an odd-root
exponent (`(-8)^(-1/3)`). The sweep goes in a new probes case (milestone M2, the milestone of
T-0221's fit, by decision 0043: never M1). If the sweep refutes `1/(x^|y|)`, record what it does
show in numbers.md, fix only what the evidence supports, and hand the rest back to T-0221 in the
Outcome note.

Code: `arithmetic::power` (POWER and `^`). docs/excel-semantics/numbers.md has "POWER and `^`"
and "Integer exponents". Keep the special cases there unchanged: `0^-1` is #DIV/0!, and an odd
root of a negative base is real.

## Acceptance criteria
- [x] A probe sweep (a new probes case, milestone M2, oracle excel-graph) shows the rule for
      negative non-integer exponents. The numbers.md "POWER and `^`" entry states the rule with that
      evidence and lists `Tasks: T-1421`.
- [x] `gen-m1/6462bf910ef9f28f` Calc!A41 and Calc!A207 disappear from eval/baseline/gen-m1.txt, so the file
      has no F lines. gen-m2 `1b6ceed7489e0800` Calc!A62 disappears too if the rule covers it.
      No suite gets a new F line.
- [x] Unit tests in `arithmetic` pin `3^-0.5`, `0.5^-0.5` and `2^-0.5` to Excel's doubles, and pin the
      unchanged special cases `0^-1` (#DIV/0!) and `(-8)^(-1/3)` (-0.5, or whatever the probe shows).
- [x] `make check` green; reviewers approve.

## Out of scope
- Positive non-integer exponents (`100^0.3`, `3.14159^3.14159`): T-0221.
- Integer exponents: T-1302 (done).
- Desktop confirmation of the fit: T-1075 item 2 (M14).

## Outcome
- **Done by T-1319.** Its sweep (probes-numeric/power-non-integer-sweep, excel-graph) refutes the plain `1/(x^|y|)` lead with libm. The rule is `1/exp(|y|·ln x)` for a negative non-integer `y`, with `1/sqrt(x)` at `y` = -0.5. numbers.md "Non-integer exponents: exp(y·ln x)" lists T-1421.
- **Probe location:** the sweep is in the ungated probes-numeric suite rather than a new `probes` case. It is outside the frozen M1 list either way (decision 0043).
- **Cells:** gen-m1/6462bf910ef9f28f A41 and A207 and gen-m2/1b6ceed7489e0800 A62 pass, so eval/baseline/gen-m1.txt has no F lines.
- **Tests:** unit tests `non_integer_exponents_use_exp_of_y_ln_x` and `special_cases_keep_the_general_path` pin `3^-0.5`, `0.5^-0.5`, `2^-0.5`, `0^-1` (#DIV/0!) and `(-8)^(1/3)` (-1.9999999999999998, POWER.xlsx).
- **Not met:** `(-8)^(-1/3)` was probed and is -0.5 in Excel, but we give -0.5000000000000001. It is not pinned and is handed to T-0221 (numbers.md "Non-integer exponents").
