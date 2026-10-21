---
id: T-2230
title: "Probe TEXT scientific edge cases: E+ with no exponent digits, lowercase e, multi-# zero mantissa"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence decides three TEXT scientific-format cases that T-0937's rule only infers, and the formatter matches it.

## Context
T-0937 (review follow-up) made the formatter emit `E` as a literal where the format's `E+`/`E-` sits
(`formatter/parser.rs`) and print a `0` for the last `#` integer placeholder of a zero mantissa
(`formatter/format.rs`). docs/excel-semantics/number-formats.md "Scientific formats with literals after `E`".
Unevidenced:
- a format with `E+`/`E-` and no exponent digit after it (`0E+`, `0.0E-"x"`): we now print "E" with no sign;
- a lowercase `e+` in a TEXT format: we print an uppercase "E" (see the CONFLICTING lowercase `e` entry: Excel for
  the web returns `#VALUE!`, so the probe records that);
- a zero mantissa with several `#` integer placeholders (`##0.0E+0`, `###.#E+0`, `##E+0`, `?.?E+0` on 0).

## Acceptance criteria
- [ ] A probe case `eval/suites/probes/cases/text-scientific-edges.case.json` (tag M10) with one cell per case
      above (on 0, 1.5, -123456.789 and 1.23E-5), oracle-filled from excel-graph.
- [ ] The engine matches every probe cell, or each mismatch is filed as its own task.
- [ ] Findings added to the number-formats.md entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The CONFLICTING web-vs-desktop lowercase `e` question itself (T-1075).
