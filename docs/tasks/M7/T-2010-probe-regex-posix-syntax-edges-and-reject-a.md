---
id: T-2010
title: "Probe REGEX* POSIX syntax edges and reject a range ending at a class"
milestone: M7
lane: functions-text
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The POSIX-syntax rules T-1985 added from PCRE2's source are backed by oracle cells, and a range that ends at a class is `#VALUE!` as in PCRE2.

## Context
T-1985 (functions/text/regex_posix.rs `invalid_ast`, `posix_shaped`) follows PCRE2's `check_posix_syntax` beyond the probed cells. Unprobed: `[[:alpha:]-]`, `[-[:alpha:]]` (valid), a top-level `[.a.]` and `[..]`, multi-character `[[.ab.]]`, and `[a-[:alpha:]]` (PCRE2 rejects; ours accepts). docs/excel-semantics/text.md "REGEX* POSIX classes are Unicode (T-1903)" lists them as UNVERIFIED.

## Acceptance criteria
- [ ] A probe in eval/suites/probes (excel-graph values) covers each case above; the semantics entry moves them to VERIFIED.
- [ ] `[a-[:alpha:]]` is an invalid pattern if the oracle says so; no other probes cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
