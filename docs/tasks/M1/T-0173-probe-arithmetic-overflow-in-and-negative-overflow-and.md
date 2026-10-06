---
id: T-0173
title: "Probe arithmetic overflow in +, - and /, negative overflow and per-element array overflow"
milestone: M1
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for arithmetic overflow beyond the `*` case: what `+`, `-` and `/` give when the
result leaves the double range, what a negative overflow gives, and how an overflowing element
behaves inside an array operation. The rules are recorded in docs/excel-semantics/numbers.md
"Overflow in `*` is #NUM! where it happens", so its UNVERIFIED parts become VERIFIED or corrected.

## Context
M1 scope covers operators and Excel floating point. numbers.md (T-0136) verifies only `*`
(`CONCAT(1e307*100)` = #NUM!, `=1E+307*10-10^308` = 0) and lists as UNVERIFIED: `+`, `-`, `/`,
negative overflow (`-1e307*100`) and the per-element array form (`{1e307,1}*100`). The unit test
`overflowing_product_is_num_inside_formulas` pins our guess only. gen-m1 never reaches the double
range, so no oracle cell covers these. Literals of 1E+308 or more make Excel for the web reject the
workbook (numbers.md "Formula literals of 1E+308 or more are invalid"), so build large values by
computation (`9E+307`, `10^307*9`), never type them.

Probe case `eval/suites/probes/cases/arith-overflow.case.json` (excel-probe skill), one cell per
question, each with a `&""` and an `ISERROR`/`ERROR.TYPE` companion where it helps tell #NUM! from
#DIV/0! or a number:
- addition and subtraction: `=9E+307+9E+307`, `=9E+307-(-9E+307)`, `=-9E+307-9E+307`,
  `=SUM(9E+307,9E+307)`, `=9E+307+9E+307-9E+307` (does the intermediate overflow?);
- division: `=9E+307/0.1`, `=-9E+307/0.1`, `=1/1E-308`, `=1E-307/1E+300` (underflow, compare with
  numbers.md "Subnormal results become 0");
- negative product: `=-1E+307*100`, `=1E+307*-100`;
- arrays: `=INDEX({1E+307,1}*100,1)`, `=INDEX({1E+307,1}*100,2)`, `=ROWS({1E+307;1}*100)`,
  `=SUM(IFERROR({1E+307,1}*100,0))`;
- `^` overflow is out (T-0959).

Code that the findings concern: `arithmetic::times` and the `+`/`-`/`/` helpers in
crates/rustsheet-engine/src/arithmetic.rs.

## Acceptance criteria
- [ ] `eval/suites/probes/cases/arith-overflow.case.json` and its oracle sidecar (excel-graph) are committed.
- [ ] numbers.md: the overflow entry states the rule for `+`, `-`, `/`, negative overflow and arrays,
      each part marked VERIFIED with the probe cells as evidence (or corrected where Excel differs).
- [ ] Every probe cell the engine gets wrong is listed as a known failure in eval/baseline/probes.txt
      (new case, not a regression) and covered by a filed engine follow-up task naming the cells;
      or fixed here if the fix is a few lines in `arithmetic.rs` (then the unit test pins Excel's values).
- [ ] `make check` green; reviewers approve.

## Out of scope
- `^`/POWER overflow (T-0959) and function results (SUM of ranges, PRODUCT) beyond the cells above.
- Changing the subnormal flush (numbers.md "Subnormal results become 0" has its own evidence).

## Outcome
Probe `probes/arith-overflow` (18 questions, 4 cells each, excel-graph) shows `+`, `-`, `/`, negative
products and per-element array overflow all give `#NUM!` (also `SUM`), and that a divisor below the
smallest normal double is `#DIV/0!`. Fixed in `arithmetic.rs` (`plus`, `minus`, `divide`) with a
unit test. Known failures left: `SUM` overflow (T-0185) and the 14-digit text of 4.4942328371558E+307
(T-0186). numbers.md updated.
