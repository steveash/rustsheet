---
id: T-2109
title: "Generator edge pools: a catalog pool key on params, with numeric pools for the M5 math functions"
milestone: M5
lane: harness
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
A catalog parameter can name an edge pool (`pool: <name>`). For such a parameter the generator
takes a value from that pool 35% of the time, instead of the free expression it builds now. The
numeric pools below exist, and the M5 math parameters they fit carry the key. A regenerated gen-m5
then hits the domain edges where Excel's math functions differ: trig limits, the ends of an
inverse function's domain, 2^53 and date serials.

## Context
- M5 exit asks for "≥ 20k cells across those functions, with edge pools" (the M2 wording that M5's
  "generated suites" criterion inherits). Today every number literal comes from the one
  `NUMBER_POOL` in tools/src/rustsheet_tools/gen/generator.py (20 values). Nothing aims at a
  function's own edges. Design: docs/architecture/synthetic-generator.md §3.2 (`pool:` on an
  argument, e.g. `pool: number_formats`) and §6.1 (number edge classes).
- Order inside `Gen.argument_text` for a `value` param: the domain (`domain_value`) comes first,
  as now. If the domain gives no value, a `pool` param draws from its pool with probability 0.35,
  and otherwise the free expression is built as today. A pool entry is formula text, so `PI()/2`
  and `2^53+1` are allowed. Negative numbers are written in parentheses, as `literal` does.
- Pools to add, in a new module tools/src/rustsheet_tools/gen/pools.py (a dict from name to list of
  formula-text strings; one definition, no copies in generator.py):
  - `number_edges`: `0`, `1`, `-1`, `2^31-1`, `2^31`, `2^32`, `2^53-1`, `2^53`, `2^53+1`,
    `999999999999999`, `9999999999999999`, `1E+21`, `2.2250738585072E-308`, `1E-307`,
    `1E+307`, `0.1`, `0.3`, `1/3`, `2/3`, `1+2^-52`, `1-2^-53`, `2.675`, `0.285`, `-2.5`, `0`,
    `59`, `60`, `61`, `2958465`, `2958466`, `0.999988426`.
  - `unit_interval` (ASIN, ACOS, ATANH, and FISHER later): `-1`, `-1-2^-52`, `-0.9999999999999999`,
    `-0.5`, `-1E-300`, `0`, `1E-300`, `0.5`, `0.9999999999999999`, `1`, `1+2^-52`, `1.5`.
  - `angle` (SIN, COS, TAN, COT, SEC, CSC and the hyperbolics): `PI()`, `-PI()`, `PI()/2`,
    `PI()/4`, `3*PI()/2`, `2*PI()`, `100*PI()`, `1E-15`, `0.00625`, `28000`, `54267`, `1E+8`,
    `134217727`, `134217728`, `1E+9`, `1E+15`, `1E+16`, `710`, `-710`.
  - `code_point` (CHAR, UNICHAR, BASE/DECIMAL radix later): `0`, `1`, `9`, `10`, `31`, `32`, `65`,
    `127`, `128`, `129`, `141`, `143`, `144`, `157`, `160`, `255`, `256`, `8364`, `55295`,
    `55296`, `57343`, `57344`, `65535`, `65536`, `1114111`, `1114112`.
- Literal limits Excel enforces on the whole workbook (docs/excel-semantics/numbers.md): a formula
  literal ≥ 1E+308 makes Excel reject the file. Keep such values out of every pool, and build them
  by computation only if needed (`1E+307*10`). A `hard: true` domain is never overridden by a pool
  (MUNIT; synthetic-generator.md §3.2).
- Catalog keys to add (eval/catalog/functions.yaml, plus a line in its header comment): `angle` on
  the number param of SIN, COS, TAN, COT, SEC, CSC, SINH, COSH, TANH, COTH, CSCH, SECH; `unit_interval`
  on ASIN, ACOS, ATANH; `number_edges` on the first number param of ROUND, ROUNDUP, ROUNDDOWN, MROUND,
  TRUNC, INT, MOD, QUOTIENT, CEILING.MATH, FLOOR.MATH, EVEN, ODD, SQRT, LN, LOG10, EXP, FACT, POWER;
  `code_point` on CHAR and UNICHAR.

## Acceptance criteria
- [ ] tools/src/rustsheet_tools/gen/pools.py holds the four pools above. A tools test checks that every
      `pool:` name in functions.yaml exists in it, and that no entry is a literal ≥ 1E+308.
- [ ] A tools test shows that a `pool` param draws pool entries, about 35% of the time over 2,000
      draws (bounds 25%–45%), and that a `hard` domain param never draws one.
- [ ] The catalog keys listed above are present. `make eval` and `rustsheet-catalog-view` still read the catalog.
- [ ] `generate_case` output for a catalog with no `pool:` keys is unchanged (the pinned-digest test
      from T-2108, or an equivalent one here if T-2108 has not landed).
- [ ] docs/architecture/synthetic-generator.md §6 says which pools are implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Text, number-format and error pools (T-2110).
- Regenerating gen-m5 (T-2114).
- Pool-aware coverage counting (the §13 coverage matrix).
