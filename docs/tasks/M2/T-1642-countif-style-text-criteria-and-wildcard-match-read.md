---
id: T-1642
title: "COUNTIF-style text criteria and wildcard MATCH read lone surrogates exactly (probe first)"
milestone: M2
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`COUNTIF`-style `=`, `<>`, `<`, `>` criteria and wildcard patterns (`*`, `?`) that contain or meet a lone surrogate match as Excel does.

## Context
T-1585 made `compare_values`, EXACT and exact MATCH lone-surrogate-exact, but the criteria matchers in functions/criteria.rs still call `cmp_text_excel(&str)` (lossy U+FFFD), and `wildcard_text_matcher_for` only handles a lone-half pattern with no wildcards. Excel's behavior here is unprobed (docs/excel-semantics/text.md, lone surrogates entry): write a probe first (excel-probe skill).

## Acceptance criteria
- [x] A probe case records Excel's COUNTIF/SUMIF and wildcard-MATCH results for a lone half; the engine matches it.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
- Done: probe probes/criteria-lone-surrogates (43 Calc formulas, excel-graph sidecar). Excel's rules are recorded in docs/excel-semantics/text.md under "Lone surrogates through CODE, ...", in the "Criteria and wildcards" paragraph. The baseline has the case's 30 failing cells as new coverage.
- To do: make the engine match those cells.
  - The criteria matchers in functions/criteria.rs (`result_is_*_string`) take `&str` targets and `cmp_text_excel`. They should keep the criterion as `Text` and compare cells with `cmp_text`, as exact MATCH already does since T-1585.
  - The wildcard path (`from_wildcard_to_regex`, `wildcard_text_matcher[_for]`) runs a regex over lossy strings. It needs a matcher over UTF-16 units (`Text::chars_lossless` or the units) in which `?` is one unit and `*` is any run. Keep case-insensitivity and `~` escaping; `"~"&H` matches nothing.
  - Then check MATCH(..,0) and XMATCH(..,2), which share the wildcard matcher.
- Pause reason: session time budget (opus worker, 2026-10-16). No attempt failed.

## Outcome
- Text criteria (`=`, `<>`, `<`, `>`, `<=`, `>=`) take their operand as `Text` and compare with `cmp_text`, so a lone half is its own unit (never U+FFFD).
- `WildcardPattern` (functions/criteria.rs) replaces the regex translation for criteria and MATCH/XMATCH/XLOOKUP wildcard mode: `?` is one UTF-16 unit, `*` any run, matched over lower-cased units. That also fixes line breaks, which the regex's `.` skipped. A second probe (probes/criteria-wildcard-newline, 17 cells) shows Excel matches them.
- Eval: probes/criteria-lone-surrogates 18/48 → 48/48; probes/criteria-wildcard-newline 17/17 new.
- Follow-up: T-1666 (SEARCH still uses the regex wildcard translation).
