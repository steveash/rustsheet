---
id: T-1830
title: "Benchmarks for lifted REGEX* and TEXTSPLIT calls over a column"
milestone: M7
lane: perf
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
crates/rustsheet/benches/text.rs measures lifted REGEX* and TEXTSPLIT calls over a column, so changes to their engines
ship with before/after numbers.

## Context
T-1272's performance review: the counted regex engine (functions/text/regex_engine.rs) shipped without bench cases.
The reviewer's release measurements (100,000-row columns): `SUM(--REGEXTEST(A1:A100000,"x"))` over 200-byte cells 91 ms;
`\w{10}` over 60-byte cells 55 ms; `ROWS(REGEXEXTRACT(A1:A100000,"(\w+)@(\w+)\.com"))` 38 ms;
`SUM(LEN(REGEXREPLACE(A1:A100000,"o","0")))` 156 ms. Follow the existing bench file's style.

## Acceptance criteria
- [ ] Bench cases for column REGEXTEST, REGEXEXTRACT, REGEXREPLACE and TEXTSPLIT exist and run under `cargo bench`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
