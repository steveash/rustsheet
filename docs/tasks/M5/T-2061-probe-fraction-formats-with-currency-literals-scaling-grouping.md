---
id: T-2061
title: "Probe fraction formats with currency, literals, % and , scaling, grouping and #/0 numerators"
milestone: M5
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-2029]
---

## Goal
We have Excel evidence (excel-graph) for fixed-denominator fraction formats combined with the
format features that gen-m10 never mixes with them. Each finding is recorded in
docs/excel-semantics/number-formats.md "Fraction formats", and any mismatch with the engine becomes
its own implementation task.

## Context
T-2029 implemented `# ?/2`, `# ?/4`, `# ?/10`, `# ??/16` and `?/100` from gen-m10 cells, which use no
currency, literals, scaling or grouping. formatter/fraction.rs already makes these choices, none of
them probed:
- the minus leads a currency symbol and leading literals (`$# ?/4` on -0.5 gives "-$ 2/4", `"x"# ?/4`);
- `%` and a trailing `,` scale the value before the fraction is taken (`# ?/4%`, `# ?/4,`);
- `#` and `0` numerator placeholders pad as for integers (`# #/4`, `# 00/16` on 1.5);
- a tiny negative with no integer part prints "-0/100" (`?/100` on -0.001);
- grouping (`#,##0 ?/4` on 1234.5) is not applied.
- the `#`-numerator rules (blank fraction prints nothing, separator dropped when a side is empty)
  also apply under a fixed denominator (`# #/4` on 1 gives "1", on 0.5 "2/4"; T-2030);
- `0` placeholders in a denominator pad with zeros (`?/00` on 0.5 gives "1/20"; T-2030).
Use the `excel-probe` skill. Write one probe case with TEXT() cells for each bullet: positive,
negative, zero and a carry value.

## Acceptance criteria
- [ ] A probes case covers every bullet above and has oracle values (excel-graph).
- [ ] number-formats.md "Fraction formats" states the rule each bullet follows, citing cells.
- [ ] Each mismatch with the engine is filed as a task (`make new-task`) or fixed here, if size S.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Placeholder denominators (T-2030), numerator overflow (T-1740), escaped literals and `_x` between
  the parts (T-0982).
