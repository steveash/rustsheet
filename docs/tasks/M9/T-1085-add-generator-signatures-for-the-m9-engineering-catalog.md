---
id: T-1085
title: "Add generator signatures for the M9 engineering catalog functions"
milestone: M9
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0040]
---

## Goal
`rustsheet-gen --milestone M9` can generate formulas for the engineering functions (54 in scope): each in-scope catalog
entry with milestone M9 has a full signature (`returns`, `params` with `mode`, `type`, `domain`, `prior`).

## Context
T-0040 cataloged every Excel function, but only the 35 M2 entries carry signatures; the others
are one-line entries (`name`, `category`, `milestone`, `scope`) that the generator skips
(`Catalog.scoped`). Minimum argument counts are already vetted in eval/catalog/excel-functions.tsv,
and tools/tests/test_catalog_arity.py checks every signature against them. Argument modes
(value / aggregate / range / criteria) decide coercion, so take them from oracle evidence
(excel-probe skill), not from the documentation alone. Functions marked `oracle: none` need
another evidence source before a generated suite can score them. One of the per-category tasks split from the M9 signature work; categories here: engineering.

## Acceptance criteria
- [x] Every `scope: in` catalog entry with `milestone: M9` and category engineering (and no `oracle: none`) has a signature
  whose required-argument count matches excel-functions.tsv.
- [x] A seeded `gen-m9` sample (`rustsheet-gen --milestone M9`) builds, and Excel for the web
  accepts every generated workbook (no arity-invalid calls).
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the functions; filling the generated suite's oracle values (own task).

## Outcome
- **Signatures.** All 54 in-scope M9 engineering entries now carry signatures in eval/catalog/functions.yaml.
  None is `oracle: none`. test_catalog_arity checks each required count against excel-functions.tsv, and the
  new test_generator_emits_every_m9_engineering_function checks that the generator emits all 54.
- **Modes.** They come from the new probe probes/eng-argument-modes (231 cells, excel-graph, 2026-10-11;
  engineering.md "Engineering function argument modes").
  - Every scalar lifts over an array literal, so it is `value`.
  - Over a multi-cell range only the BIT* functions lift; the rest are `#VALUE!`.
  - IMSUM's and IMPRODUCT's inumbers are a repeating `array` param. They take ranges and arrays whole,
    count a blank cell as 0, and give `#VALUE!` for booleans and `#NUM!` for non-complex text.
- **Domains.** Binary and octal inputs are number enums. Hex and complex inputs are text enums. CONVERT's
  units are a text enum. `places`, bit widths, shifts and Bessel orders are soft int domains.
- **Seeded sample.** `rustsheet-gen --milestone M9 --count 20 --seed 17` was run from a scratch catalog
  holding only the engineering signatures, and was not committed. It made 800 formulas calling all 54
  functions. excel-graph accepted all 20 workbooks (0 failures), and the engine passed 744/800 cells with
  no abort.
- **Eval delta.** probes gained 1 case: 231 cells, 112 pass and 119 are new known failures.
  - T-1383, lifting: 94 cells.
  - T-1384, IMSUM/IMPRODUCT and booleans: 21 cells.
  - T-0907, IM* last digit: 3 cells.
  - Regressions: 0.
- **Follow-ups.** T-1383, T-1384, and T-1385 for the sample's other misses: error order in
  CONVERT/DELTA/GESTEP, HEX2OCT `places`, IMSQRT(""), negative fractional Bessel order and E-notation
  IM* parts. The Bessel value misses belong to T-0900 and T-0908.
