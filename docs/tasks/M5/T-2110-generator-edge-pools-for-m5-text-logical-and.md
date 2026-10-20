---
id: T-2110
title: "Generator edge pools for M5 text, logical and information params: text, numeric-text, number-format and error pools"
milestone: M5
lane: harness
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: [T-2109]
---

## Goal
The generator's `pool:` mechanism (T-2109) gets the pools that the M5 text, logical and information
functions need: edge text, numeric-looking text, number-format strings and error values. Their
parameters carry the key, so a regenerated gen-m5 passes TEXT real format strings, gives VALUE and
NUMBERVALUE the text it has to parse, and gives the IS* functions, ERROR.TYPE, IFNA and TYPE real
error values.

## Context
- Today TEXT's `format_text` is a free text expression, mostly "abc"-style pool text, so TEXT
  almost always returns its literal. No generated argument is ever an error value: `TEXT_POOL` has
  only the text `"#N/A"`. That leaves ISERR/ISERROR/ISNA/ERROR.TYPE/IFNA unexercised on errors.
  Design: docs/architecture/synthetic-generator.md §6.2 (text) and §6.5 (errors).
- Pools to add to tools/src/rustsheet_tools/gen/pools.py, the T-2109 module:
  - `text_edges`: `""`, `" "`, `CHAR(9)`, `CHAR(10)`, `CHAR(160)`, `UNICHAR(12288)`, `UNICHAR(8203)`,
    `" 42 "`, `"a"`, `"A"`, `"ä"`, `"ß"`, `"İ"`, `"é"`, `"e"&UNICHAR(769)`, `UNICHAR(128512)`,
    `"'a"`, `"a-b"`, `REPT("x",255)`, `REPT("ab",300)`, `"#N/A"`, `"=1+1"`.
  - `numeric_text`: `"42"`, `" 42 "`, `"4.2e1"`, `"1E3"`, `"1,000"`, `"$1,000.50"`, `"(100)"`,
    `"50%"`, `"-0"`, `"0x1F"`, `"TRUE"`, `"2024-02-29"`, `"1900-02-29"`, `"12:30"`, `"12:30 PM"`,
    `"25:00"`, `"1.5"`, `".5"`, `"1e400"`.
    Leave out `"1/2"` and any other year-less date: it parses as a date in the current year, so its
    value depends on the clock (§6.2).
  - `number_formats`: the format codes in tools/src/rustsheet_tools/gen/number_formats.py
    (`BUILTIN` and `CUSTOM`), imported and quoted as formula text, never copied (CLAUDE.md
    directive 3). A code longer than 255 characters cannot be a formula string literal; leave it out.
  - `errors`: `NA()`, `1/0`, `#N/A`, `#DIV/0!`, `#VALUE!`, `#REF!`, `#NAME?`, `#NUM!`, `#NULL!`,
    `SQRT(-1)`, `VALUE("x")`.
- Catalog keys to add (eval/catalog/functions.yaml): `number_formats` on TEXT `format_text`;
  `numeric_text` on VALUE, NUMBERVALUE `text`, and on the `value` param of N and T; `text_edges` on the
  text param of LEN, LENB, TRIM, CLEAN, PROPER, UPPER, LOWER, CODE, UNICODE, EXACT (both), SUBSTITUTE
  `text`, REPT `text`, FIND/SEARCH `within_text`; `errors` on the `value` param of ISERR, ISERROR, ISNA,
  ISBLANK, ISTEXT, ISNONTEXT, ISNUMBER, ISLOGICAL, IFNA and IFERROR, and on ERROR.TYPE `error_val`.
  T-2109's pool draw covers `value`-mode params only. TYPE and ISREF take an `array`-mode param, so they
  stay out unless this task extends the draw to the expression branch of `array` params. If it does,
  add `errors` to TYPE and say so in the Outcome. The param names above were checked against the
  catalog on 2026-10-20.
- An error literal such as `#REF!` in a generated formula must be accepted by spec2xlsx and by the
  oracle's workbook upload. Check both on one case before relying on it. If one is refused, keep
  only the computed forms (`NA()`, `1/0`, ...) and note it in the Outcome.

## Acceptance criteria
- [ ] The four pools above exist in pools.py. `number_formats` is built from number_formats.py's lists,
      and a tools test asserts that it is a quoted subset of `BUILTIN + CUSTOM`.
- [ ] No pool entry is a string literal longer than 255 characters (tools test; Excel's formula
      string-literal limit, synthetic-generator.md §6.2). No `numeric_text` entry is a year-less date.
- [ ] The catalog keys listed above are present, and the pool-name test from T-2109 passes.
- [ ] One generated M5 case that uses every new pool converts with spec2xlsx, and `rustsheet-eval run`
      executes it without a crash (stated in the Outcome with the command used).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Error cells on the generated Data sheet (a later generator change).
- Regenerating gen-m5 (T-2114).
