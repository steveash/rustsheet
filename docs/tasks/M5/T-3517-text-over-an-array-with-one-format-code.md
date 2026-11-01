---
id: T-3517
title: "TEXT over an array with one format code checks and parses the code once per call, not once per element (probes/t2759-lunar-packed-* take 10-18 s each)"
milestone: M5
lane: perf
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(array, format_text)` with a scalar format code costs one format check and one format parse per call, not per
element, so a TEXT lifted over tens of thousands of serials costs little more than formatting them. Every TEXT
result stays exactly as it is today.

## Context
Gap triage 2026-10-31 (`make eval` on origin/main 744815d0), "Timing": nine of the ten slowest cases of the run are
probes/t2759-lunar-packed-2101 ... -9101 (10.0 to 18.2 s each, about 106 s of the probes suite's 144 s);
probes/text-locale-lunar-months-1900 (22 s, T-2908) is the tenth. Each t2759 cell runs about seven
`TEXT(SEQUENCE(36525,...),"[$-110409]...")` calls over one Gregorian century, so a case makes roughly 4.6 million
TEXT element calls. Alone, `target/release/rustsheet-eval run --suite probes --filter t2759-lunar-packed-9101`
takes 11.4 s.

Profile (callgrind on a scratch probe `=SUM(--TEXT(SEQUENCE(40000,1,80000),"[$-110409]d"))`, release build):
`fn_text`'s closure costs about 11,000 instructions per element, of which
- `rejects_format_code` (formatter/complexity.rs: length, complexity and unterminated checks of the code) is 28%,
- `formatter::parser::Parser::parse` (inside `format_number`, formatter/format.rs, which builds a `Parser` from the
  code on every call) is 30%,
- the lunar date lookup itself is small: past 2100 `lunar_ymd` already binary-searches the decoded `LATER` table.

Both the check and the parse depend only on the format code, which is the same string for every element when the
format argument is a scalar. Code paths: `Model::fn_text` (functions/text/common.rs) runs per element under
`lift_call`; it calls `calc_result_to_text(format)`, `rejects_format_code(&format_code)` and
`text_element` -> `text_node` -> `format_text` -> `format_number`, and `text_value_node` ->
`format_text_value_exact` for text values.

Suggested shape (confirm against a profile; search for an existing parsed-format helper first, CLAUDE.md
directive 3): split `format_number` into the parse (`Parser::new(format).parse()`, giving the parts) and the
formatting of a value with already-parsed parts, keep `format_number(value, format, ctx)` as the wrapper its other
callers use, and in `fn_text` remember the last format code with its check result and parsed parts, reusing them
while the next element's code is equal. Keep the existing order of error checks (the value's error first, then the
format's error, then `#VALUE!` for a rejected code; `fn_text`'s doc comment cites the evidence).

- Lane `perf`: a hot-path change with no behaviour change. T-2908 (lunar months before 2100, lunar_calendar.rs) is a
  separate hot spot and does not overlap.

## Acceptance criteria
- [ ] `target/release/rustsheet-eval run --suite probes --filter t2759-lunar-packed-9101` runs at least 30% faster
      than on the base commit (before/after wall times of three runs each in the Outcome).
- [ ] A unit test pins that TEXT over an array with a scalar format gives the same results as the per-element calls,
      for a number, a blank, a text, a boolean and an error element, under a valid, a rejected (over 255 characters)
      and an unterminated (`0;"`) code.
- [ ] `make eval` shows no verdict change in any suite (no new and no removed known-failure lines).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Caching parsed formats across calls or cells (a cell's display format, CONCAT of TEXT results).
- The lunar month walk before 2100 (T-2908) and per-element cost in other lifted text functions.
