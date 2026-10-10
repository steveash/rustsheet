---
id: T-1302
title: "Power operator and POWER: integer exponents by right-to-left square-and-multiply (6 gen-m1 cells)"
milestone: M1
lane: engine
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
`x^n` and `POWER(x,n)` with an integer exponent `n` give Excel's double. This is the verified
integer-exponent half of T-0221, split out because it clears 6 of the 8 cells that block the M1
exit (gen-m1 at 100%).

## Context
The rule is already fitted and VERIFIED. See docs/excel-semantics/numbers.md, "Integer exponents:
right-to-left square-and-multiply" (T-0942, probes-numeric/m9-fin-power, 154/154 cells per form).
For an integer `n`, Excel computes binary exponentiation in doubles and scans `n`'s bits from the
low end: `r=1; b=x; while n: if n&1: r*=b; b*=b; n>>=1`. `x^-n` is `1/` that result. This task
implements a rule that is already fitted. It does no fitting of its own.

Code: `power` in `crates/rustsheet-engine/src/arithmetic.rs`, which serves both `^` and POWER.
Today it calls `x.powf(y)` for every exponent. Search for an existing integer-power helper first
(`rg powi`). Use no `powi`: it does not specify the multiplication order.

gen-m1 cells this fixes (`make eval-show CASE=...`; the planner checked each by hand on 2026-10-10
against the rule above):
- `gen-m1/6462bf910ef9f28f` A33 `=(-Data!E7)^3`, A48 `=+Data!$D9^3`, A86 `=-Data!$C$3^3`
  (base ±1E-15, expected ±1.0000000000000001e-45). A57 `=Data!E7^3^2` (expected
  1.0000000000000002e-90).
- `gen-m1/57fe331cb7611dbf` A54 `=((('Q''1'!A$7^Data!B10)^("0.5"^TRUE))^10)`: the final `^10` of
  0.7071067811865476 gives 0.031250000000000035. A104 `=('Q''1'!B9^'Q''1'!C1)`, which is 3.25^100,
  gives 1.54289402744788e+51.

probes-numeric/m9-fin-power has 616 cells; 301 of them fail today, all in `^n`, `^-n` and `POWER`
forms. gen-m2 cells such as f0d1bacec5de8146 A196 (`100^100` = 1.0000000000000005e+200) should
also clear.

So that no judgment call is needed, apply the rule only under these conditions:
- `y` is finite, `y.fract() == 0.0`, and `|y| <= 2^53`. That is at most 54 loop iterations,
  which keeps the work bounded (directive 5).
- The square-and-multiply result `r` is finite and non-zero. For a negative `n`, `1/r` must also
  be finite and non-zero.

Otherwise keep today's path unchanged, including every special case and error. Those cases are
`0^0`, `0^-n`, overflow and the odd-root branch. Overflow and underflow beyond the oracle's range
of exponents (1..360) are not verified. Leave them on the existing behavior and record that in
numbers.md.

## Acceptance criteria
- [ ] The 6 cells above (6462bf910ef9f28f A33, A48, A57, A86; 57fe331cb7611dbf A54, A104) leave
      eval/baseline/gen-m1.txt via `make eval-baseline`. No new F lines appear in any suite. Any
      gen-m2 cells that clear also leave eval/baseline/gen-m2.txt.
- [ ] `make eval-show CASE=probes-numeric/m9-fin-power.case.json` reports 616/616 passed. If any
      cell still fails, the PR lists it and explains why.
- [ ] Unit tests in arithmetic.rs pin `power(1e-15, 3.0)`, `power(3.25, 100.0)`,
      `power(100.0, 100.0)` and one negative integer exponent to the oracle doubles above. They
      also show that `power(0.0, -1.0)` is still #DIV/0!, `power(0.0, 0.0)` is still #NUM!, and
      `power(-8.0, 1.0/3.0)` is still -2.
- [ ] numbers.md "Integer exponents" gets an `Implemented in:` line and states the overflow and
      underflow fallback. In "POWER and `^`", `Tasks:` lists T-1302 next to T-0221.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Non-integer exponents (`3^-0.5`, `0.5^-0.5`, `100^0.3`), which need numeric fitting: T-0221.
- Financial functions that raise `(1+rate)` to a power internally: T-0901.
