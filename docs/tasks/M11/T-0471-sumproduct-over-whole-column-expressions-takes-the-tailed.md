---
id: T-0471
title: "SUMPRODUCT over whole-column expressions takes the tailed path (SPEC-0020)"
milestone: M11
lane: perf
status: done
priority: P2
size: S
tier: opus
depends_on: [T-0469]
---

## Goal
SUMPRODUCT over whole-column element-wise expressions (`SUMPRODUCT((A:A=x)*B:B)`) does work
proportional to the used rows, with bit-identical results (SPEC-0020 Rollout 3).

## Context
SPEC-0020: SUMPRODUCT is a second consumer of `Tailed`; each argument's element sequence must equal
today's, including the shape checks between arguments and the order of products and sums.

## Acceptance criteria
- [x] SUMPRODUCT reads tailed arguments; equivalence tests (shapes, errors, text and booleans).
- [x] A bench case and before/after timings in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other array consumers.

## Outcome
- **How SUMPRODUCT reads its arguments now** (`sumproduct.rs`). Each argument is read through
  `read_tailed`, with SUMPRODUCT's own reading (`evaluate_value_or_grid`).
  - An element-wise expression of whole columns becomes `Factors::Tailed`: the head's numbers and
    the tail's number, with the rows it stands for. Every other argument is `Factors::Grid`, read
    as before. A head of an unexpected shape is also read as before, its cells already evaluated.
  - Shapes are compared at the logical size (head plus tail rows), so mismatches are the same.
  - Elements are coerced by one shared `sumproduct_factor`, argument by argument and element by
    element. So the first element error and a lambda's `#VALUE!` come out as before.
- **The product loop.** When every argument is tailed, the rows past the longest head share one
  product.
  - A zero product is skipped. `extended_sum` adds a zero plainly, and the total is never -0.
  - Any other product is added row by row, so the sum rounds as before.
  - With a grid argument, every row is walked as before.
- **One shared helper.** `Tailed::<RefArg>::elements`, the head and tail elements of a tailed
  argument, moved from aggregate_args.rs into tailed_array.rs. The aggregate walk and SUMPRODUCT
  now share it.
- **Bench** `cargo bench -p rustsheet --bench aggregate -- sumproduct` (new case): 400
  `SUMPRODUCT((F:F=A1)*G:G)` over 2,000 rows. Release, fastest of 5: 127.6 s on `main`, 0.232 s
  here (about 550x).
- **corpus-bench 55049** (`SUMPRODUCT(book2!A:A=book1!A3)*book2!H:J`, the only whole-column
  SUMPRODUCT in the fetched suites): `evaluate()` takes 3.52 s on `main` and 1.55 ms here.
- **Tests** (test_tailed_array.rs) compare each formula with the tailed reading off.
  - One tailed argument, over operators, IF, comparisons and error tails. Also columns without
    errors (C, Sheet2), so head and tail products are actually summed.
  - Several arguments, mixing tailed and grid arguments:
    - logical shapes against a bounded range (`#VALUE!`);
    - element errors and whole-argument errors;
    - heads of different lengths on two sheets;
    - nonzero products past every head.
  - Each test also checks the count of tailed arguments.
- **Mutation:** skipping the nonzero tail rows fails both tests.
