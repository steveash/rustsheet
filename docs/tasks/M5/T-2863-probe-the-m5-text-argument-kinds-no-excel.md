---
id: T-2863
title: "Probe the M5 text argument kinds no Excel cell covers yet: an error, empty text or a range in FIND, SEARCH, SUBSTITUTE, REPLACE, CHAR, FIXED, DOLLAR, TEXTJOIN"
milestone: M5
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of the M5 text functions below has at least one Excel-backed cell for each argument kind the
report's argument coverage expects.

## Context
Gap triage 2026-10-27 (`make eval` on 7bd9090e, report.json `argument_coverage`, decision 0031 evidence gaps;
decision 0094 asks for each argument mode to be covered). The kinds are those of
`crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative, fraction,
numeric_text, text, empty_text, boolean, error, blank, range. The generator's number pool
(tools/src/rustsheet_tools/gen/generator.py `NUMBER_POOL`) has no empty text or error literal (T-2866), so
generated suites never reach most of these. `array`-mode parameters are not listed: their coverage is not
classified by kind.
These M5 text parameters have no Excel cell of the listed kinds in any suite (highest catalog prior first;
the DBCS `*B` variants, JIS and BAHTTEXT are left out: en-US behavior is the plain function's, and JIS is web-only
evidence, T-1081):
- TEXT.format_text: error.
- FIND.find_text, FIND.within_text: error; FIND.start_num: empty text, error, range.
- SEARCH.find_text: error; SEARCH.start_num: numeric text, empty text, error, range.
- SUBSTITUTE.text: error; SUBSTITUTE.old_text: range; SUBSTITUTE.new_text: error, range;
  SUBSTITUTE.instance_num: empty text, error.
- CHAR.number: empty text, error, range.
- EXACT.text2, CLEAN.text, PROPER.text, NUMBERVALUE.text/decimal_separator/group_separator, ASC.text: error.
- REPLACE.old_text, REPLACE.new_text: error; REPLACE.start_num: empty text; REPLACE.num_chars: empty text, error.
- TEXTJOIN.ignore_empty: zero, empty text, error.
- REPT.number_times: empty text.
- DOLLAR.number: error; DOLLAR.decimals: empty text, error.
- FIXED.number: empty text, error; FIXED.decimals: numeric text, text, empty text, error;
  FIXED.no_commas: zero, empty text, error.
- ARRAYTOTEXT.format: empty text, error, range. VALUETOTEXT.format: empty text. UNICODE.text: range.

Write one probe case, probes/m5-text-argument-kind-gaps (excel-probe skill, excel-graph, tag M5), with each kind in two forms where
they differ: a literal (`""`, `NA()`, `#DIV/0!`) and a cell holding it (a formula cell `=""`, `=NA()`). Give every
other argument an ordinary valid value, so the probed argument decides the result, and wrap each error-valued
cell in `ERROR.TYPE` too.

## Acceptance criteria
- [ ] probes/m5-text-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed
      above; after `make eval`, report.json `argument_coverage` shows `covered == expected` for them.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's
      own module) or listed in a new functions-* task (campaign: semantics, with a `## Visible-parity stage`
      section) filed with `make new-task`, naming the case and cells; the new known failures are recorded with
      `make eval-baseline` and each has that owner.
- [ ] Any new Excel rule is recorded in docs/excel-semantics/text.md or coercion.md citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- M2 text functions (LEFT, RIGHT, MID: T-2861); M5 math (T-2862); generator pools (T-2866).
- TEXT format codes (the functions-text TEXT tasks).
