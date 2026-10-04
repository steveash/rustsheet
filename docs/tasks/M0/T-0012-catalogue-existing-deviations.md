---
id: T-0012
title: "Catalogue existing directive-5 deviations as DEVIATION entries"
milestone: M0
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every place the engine deliberately differs from Excel for safety has a
`Status: DEVIATION (decision 0014)` entry in docs/excel-semantics/, so the canonical list matches the code.

## Context
Decision 0014 requires an entry per directive-5 deviation. Only the BESSELJ/Y order cap has one.
Known candidates: `MAX_SIZE = 1_000_000` in
`crates/rustsheet-engine/src/functions/math_and_trigonometry/array_size.rs` (SEQUENCE/RANDARRAY return `#ERROR!`,
not even an Excel error value, above 1M cells; Excel allows the full grid), `MAX_DEGREES_OF_FREEDOM` in `chisq.rs`, the 2^53 limit in
`gcd_lcm.rs`, parser/evaluator recursion limits, importer limits, and functions that would do I/O
in Excel (WEBSERVICE, external links). Some may be real Excel limits: those become VERIFIED rules
with oracle evidence (`excel-probe`), not deviations.

## Acceptance criteria
- [ ] `rg` sweep for limit constants and I/O-function stubs, listed in the PR body.
- [ ] Each true deviation has a DEVIATION entry stating Excel's behavior and ours, with an `Implemented in:` path and a code comment pointing back to it.
- [ ] Each limit that matches Excel has a VERIFIED entry with oracle evidence (or an UNVERIFIED entry with a pending probe).
- [ ] `make check` green; reviewers approve.

## Out of scope
Changing any limit's value.
