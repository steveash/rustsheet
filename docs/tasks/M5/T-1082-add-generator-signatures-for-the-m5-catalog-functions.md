---
id: T-1082
title: "Add generator signatures for the M5 catalog functions"
milestone: M5
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-0040]
---

## Goal
`rustsheet-gen --milestone M5` can generate formulas for the math, logical, text and information functions (134 in scope, decision 0027): each in-scope catalog
entry with milestone M5 has a full signature (`returns`, `params` with `mode`, `type`, `domain`, `prior`).

## Context
T-0040 cataloged every Excel function, but only the 35 M2 entries carry signatures; the others
are one-line entries (`name`, `category`, `milestone`, `scope`) that the generator skips
(`Catalog.scoped`). Minimum argument counts are already vetted in eval/catalog/excel-functions.tsv,
and tools/tests/test_catalog_arity.py checks every signature against them. Argument modes
(value / aggregate / range / criteria) decide coercion, so take them from oracle evidence
(excel-probe skill), not from the documentation alone. Functions marked `oracle: none` need
another evidence source before a generated suite can score them. Size M: split by category
if the signatures do not fit one PR.

## Acceptance criteria
- [ ] Every `scope: in` catalog entry with `milestone: M5` (and no `oracle: none`) has a signature
  whose required-argument count matches excel-functions.tsv.
- [ ] A seeded `gen-m5` sample (`rustsheet-gen --milestone M5`) builds, and Excel for the web
  accepts every generated workbook (no arity-invalid calls).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the functions; filling the generated suite's oracle values (own task).
