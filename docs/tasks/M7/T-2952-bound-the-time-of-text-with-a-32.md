---
id: T-2952
title: "Bound the time of TEXT with a 32,767-character format over an array"
milestone: M7
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A TEXT call over an array with a very long format code finishes within the SPEC-0024 time rule (a formula at its work
limit within 10 s of a release build), or is refused the way Excel refuses it.

## Context
Found by T-2806 (release build, 2026-10-27): `=TEXT(SEQUENCE(1024),REPT("0",32767))` takes 23 s, 22 ms a call (the
formatter looks quadratic in the format's length), and `=TEXT(SEQUENCE(1024,1024),REPT("a",32767))` ran past 60 s.
Memory stays small (48 MB); only time is unbounded, so 2^20 cells would take hours. The lift driver charges TEXT's
elements by `KernelWork`, which does not weigh the format's length. Excel refuses long format codes by a complexity
limit (number-formats.md "TEXT format code complexity limit"; `REPT("$",250)` fails), but the engine accepted the
32,767-zero format: first check what Excel does with `REPT("0",n)` formats near that limit (excel-probe skill; the
existing ladders are probes/text-format-length-limit-*), then either refuse as Excel does or charge the format's length
per element (formatter/, functions/lift kernel table).

## Acceptance criteria
- [x] The two formulas above finish under 10 s in a release build (measured in the Outcome), with Excel's value or a
      documented DEVIATION.
- [x] No baseline gains a line.
- [x] `make check` green; reviewers approve.

## Out of scope
- Memory (T-2806).

## Outcome
Already bounded on `main` by T-2842's 255-character limit on a TEXT format code (`rejects_format_code` in
formatter/complexity.rs, merged 2026-10-27, after T-2806 measured the 23 s): a code over 255 characters is `#VALUE!`
before any formatting, the rule Excel follows (`"0"` + 254 commas works and 255 fail, probes/t2842-dollar-run-contexts
J, K; number-formats.md "TEXT format code complexity limit"). So no engine change was needed. Measured on the release
`rustsheet-eval show` of `main` fa70f73 (wall time of one scratch case holding one cell, minus nothing):
- `=TEXT(SEQUENCE(1024),REPT("0",32767))`: 0.03 s (was 23 s), 1,024 `#VALUE!`.
- `=TEXT(SEQUENCE(1024,1024),REPT("a",32767))`: 2.0 s (was past 60 s), 1,048,576 `#VALUE!`.
Added `text_with_a_32767_character_code_over_an_array_is_value_error` (test/text_functions/fn_text_arrays.rs) so the
early rejection stays ahead of the formatter. Eval delta: none; no baseline changes.
