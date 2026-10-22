---
id: T-2345
title: "Probe MAKEARRAY sizes within an ulp of a whole number"
milestone: M7
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish with the Excel oracle whether MAKEARRAY treats a rows or columns that is within one ulp of a whole number (`(0.1+0.2)*10`, `3-2^-51`) as whole or as `#VALUE!`.

## Context
T-2308 made fractional MAKEARRAY sizes `#VALUE!` with an exact `fract() != 0.0` test (`makearray_kernel`, functions/logical/bycol_byrow.rs). No oracle cell shows near-integer sizes; Excel may round to 15 significant digits first. See docs/excel-semantics/arrays.md "MAKEARRAY rows and columns must be whole numbers" (Unverified line). Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with Excel values for near-integer sizes exists under eval/suites/probes.
- [ ] The arrays.md entry states the result and its Unverified line is resolved.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
