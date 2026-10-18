---
id: T-1893
title: "KernelWork check: value-dependent kernels in the lifting table are Weighted or name their fixed cap"
milestone: M9
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1776]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
Every value-dependent kernel in the SPEC-0011 lifting table declares its work: `Weighted`, or `Constant` with the fixed iteration cap named, and a test enforces it.

## Context
Split from T-1776 (SPEC-0011 "Security rule for kernel work"). A unit test in crates/rustsheet-engine/src/functions/lift/kinds.rs (or the eval crate over `rustsheet_engine::lift_kinds_table()` / eval/catalog/lift-kinds.json) holds a list of value-dependent functions: BINOM.DIST, BINOM.DIST.RANGE, NEGBINOM.DIST, POISSON, COMBIN, FACT, FACTDOUBLE, MULTINOMIAL, PERMUT, RATE/IRR/YIELD and the other solvers, the Bessel functions, the coupon walks, CUMIPMT/CUMPRINC, ... Each one present in the table must be `Weighted`, or `Constant` with an entry naming its fixed cap (today BINOM.DIST, BINOMDIST, BINOM.DIST.RANGE, NEGBINOM.DIST, NEGBINOMDIST, FACTDOUBLE, PERMUT, PERMUTATIONA are `Constant`: read each kernel and cite its cap, or make it `Weighted`).

## Acceptance criteria
- [x] The check runs in `make check` and fails on a seeded mismatch (shown once in the PR).
- [x] Every `Constant` entry on the list names its cap; any kernel without one is made `Weighted` with a per-call cap.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`crates/rustsheet-engine/src/functions/lift/tests/kernel_work.rs` lists 79 value-dependent
functions. Each entry carries the fixed cap that lets it be `Constant`, or `None`, meaning it must
be `Weighted`. The test fails for any listed function whose table entry is `Constant` with no cap
named, or `Weighted(1)`. Functions with no entry yet (FACT, COMBIN, COMBINA, MULTINOMIAL, IRR,
XIRR, DB, VDB) are checked once they are entered. A second test seeds PERMUT with no cap and
checks that the violation is reported, so the check itself is pinned.

I read each `Constant` kernel and found a fixed cap, so none was made `Weighted`:
- BINOM.DIST, BINOMDIST, NEGBINOM.DIST: one statrs incomplete beta, at most 140 steps; the pmf
  is `ln_gamma`'s fixed Lanczos sum.
- BINOM.DIST.RANGE: two incomplete betas.
- NEGBINOMDIST: `ln_gamma` only.
- FACTDOUBLE: `descending_product` overflows within 151 factors.
- PERMUT: at most 171 factors.
- PERMUTATIONA: one `powf`.

I also listed and checked HYPGEOMDIST, the beta-cdf family (BETA.DIST, F.DIST, T.DIST and their
variants), ERF/ERFC (`erfc_cf` caps its depth at 600) and SERIESSUM (charged per coefficient
cell). Eval delta: none (tests only).
