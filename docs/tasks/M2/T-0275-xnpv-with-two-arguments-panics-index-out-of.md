---
id: T-0275
title: "XNPV with two arguments panics (index out of bounds on args[2])"
milestone: M2
lane: functions-financial
status: todo
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
`=XNPV(rate, values)` (two arguments) returns Excel's result or error instead of panicking.

## Context
Found by the T-0272 audit: `fn_xnpv` accepts 2..=3 arguments but indexes `args[2]` (`crates/rustsheet-engine/src/functions/financial/mod.rs`, `fn_xnpv`), so `=XNPV(1,A1:B2)` panics (index out of bounds). Untrusted input must never panic (directive 5). Probe Excel for the two-argument form (likely rejected as an argument-count error) before choosing the result.

## Acceptance criteria
- [ ] A probe case records Excel's result for `XNPV(1,A1:B2)` and `XNPV(1,A1:B2,A1:B2)`; the finding goes in docs/excel-semantics/.
- [ ] No argument count panics in XNPV; unit test covers it.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
