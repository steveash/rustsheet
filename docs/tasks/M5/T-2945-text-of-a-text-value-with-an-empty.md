---
id: T-2945
title: "TEXT of a text value with an empty text section (;;;) is empty text, not the text"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
`TEXT` of a text value (or a boolean) with a four-section format whose 4th (text) section is empty returns empty
text, as Excel does: `TEXT("text",";;;")` is `""`.

## Context
Gap triage 2026-10-27 (report for a84da30f). This is the only visibly-off TEXT cell attributed to TEXT that no open task
owned (the other 184 are T-2779, T-1875, T-2759, T-2932, T-1925, T-2106, T-2684, T-2798, T-2388).
- gen-m10/0e3d8eba9432a18d.case.json T!A24 `=TEXT("text",";;;")`: Excel (excel-graph) `""`, ours `"text"`
  (`F	gen-m10/0e3d8eba9432a18d.case.json	T!A24` in eval/baseline/gen-m10.txt).
- The rule is already VERIFIED in docs/excel-semantics/number-formats.md "Text and booleans use the format's text
  section": for a text or boolean value TEXT applies the 4th section, so an empty 4th section prints nothing. This
  cell is the evidence for the empty-section case; it is cited there.
- Consistent evidence: number cells with `;;;` are `""` in Excel and already pass (gen-m10/810fee377a42575b A19,
  fbdab487cb11409e A3, 71ad5f93362f4ff4 A8). A *fifth* section is `#VALUE!` even when empty (`"0;;;;"`, probes
  text-format-length-limit-c A30), and that already passes too.
- Suspected cause: `format_text_value` (crates/rustsheet-engine/src/formatter/format.rs) returns the text unchanged when the
  text section yields no tokens (`section_tokens(part)` → `None`, or `parser.parts.get(3)` → `None` if the parser drops
  an empty trailing section). An empty 4th section should give empty text instead. Check which one with a unit
  test before changing anything. Keep the "fewer than four sections returns the text unchanged" path (`TEXT("x","4;b")`
  is "x", number-formats.md), which is a different case.
- Related: T-1736 owns the *displayed* text of text/boolean cells through the format's text section (same case,
  F!X13 and F!X20 show `;;;` hiding "TRUE" and "text"). If both share this helper, note in the Outcome whether
  those TF lines moved, but do not take on T-1736's cell-display path here.

## Acceptance criteria
- [ ] Unit test in formatter tests: `format_text_value("text", ";;;")` and `format_text_value("TRUE", ";;;")` give `""`;
      `format_text_value("x", "4;b")` is still "x"; `format_text_value("x", "0;;;;")` is still `#VALUE!`.
      The boolean and five-section expectations are inferred (poi/TextFormatTests A17, T-1736's F!X20, and
      `TEXT("1","0;0;0;;")`); add probe cells `=TEXT(TRUE,";;;")` and `=TEXT("x","0;;;;")` and follow Excel.
- [ ] The line `F	gen-m10/0e3d8eba9432a18d.case.json	T!A24` disappears from eval/baseline/gen-m10.txt (`make eval-baseline`)
      and no new F or TF line appears in any baseline.
- [ ] The "Implemented in" line of number-formats.md "Text and booleans use the format's text section" says an empty
      4th section gives empty text.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
`semantics`: a wrong text result (decision 0114). The target is every cell this task owns giving Excel's text. That is
gen-m10/0e3d8eba9432a18d T!A24, so the acceptance criteria above are the stage target.

## Out of scope
- Displayed text of cells (T-1736) and the other TEXT-on-text probes (T-1130: 1-3 section formats, `_x`/`*x`, digit and
  date tokens in the text section).
