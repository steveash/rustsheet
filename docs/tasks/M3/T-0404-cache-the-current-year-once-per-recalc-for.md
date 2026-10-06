---
id: T-0404
title: "Cache the current year once per recalc for year-less date text"
milestone: M3
lane: perf
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Coercing many year-less date texts ("7/1" in 100k cells) reads the clock once per recalc, not once per value, and every cell of one recalc sees the same year.

## Context
From the T-0167 performance review (PR #195): `Model::current_year` (tz/mod.rs) reads the wall clock and converts the timezone on every call. The date parser calls it lazily, only for possible year-less days (`formatter::date_text::CurrentYear`), but `SUMPRODUCT(--A1:A100000)` over "7/1" cells pays that on each cell, and on wasm each call crosses into JS. Cache it on `Model` for one evaluation pass, e.g. `Cell<Option<Option<i32>>>` reset at the start of `evaluate`, along with typed input (`set_user_input`), which runs outside a recalc.

## Acceptance criteria
- [ ] The clock is read at most once per `evaluate` for year-less text (unit test with a counting or mocked clock).
- [ ] Before/after timing of 100k year-less coercions recorded in the PR.
- [ ] Eval: 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Making year-less text volatile across a year change (needs oracle evidence).
