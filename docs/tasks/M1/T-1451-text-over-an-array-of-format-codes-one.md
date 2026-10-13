---
id: T-1451
title: "TEXT over an array of format codes: one result per format, error elements keep theirs (follow-up of T-1434)"
milestone: M1
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXT accepts an array (or range) of format codes and returns one result per format.

## Context
`TEXT(1,{"0",#N/A})` is "1", `#N/A` in Excel (probes/text-format-length-limit-f A54:B54); we return `#N/IMPL!` because `fn_text` (functions/text/common.rs) reads the format with `get_string`. Each format element should get its own complexity check (formatter/complexity.rs) and error elements keep their error. Check the existing broadcast helpers (`broadcast_text` in functions/text/text_arg.rs) before writing a new one.

## Acceptance criteria
- [ ] probes/text-format-length-limit-f A54:B54 pass; a unit or spec case covers a value array against a format array (broadcast shape).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Value arrays (already handled).

## Outcome
TEXT now broadcasts over an array/range of format codes via `broadcast_text`; each format gets its own complexity check and error elements keep theirs. probes/text-format-length-limit-f A54:B54 pass (probes known failures 2771 -> 2769). Unit test in fn_text_arrays.rs.
