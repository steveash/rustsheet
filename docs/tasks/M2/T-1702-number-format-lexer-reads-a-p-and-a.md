---
id: T-1702
title: "Number-format lexer reads A/P and a locale-only [$-409] bracket"
milestone: M2
lane: functions-text
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXT accepts the `A/P` token and a locale-only `[$-409]` bracket (no currency symbol), as Excel does, at least in a text section next to `@`.

## Context
Found by T-1656. The number-format lexer (`crates/rustsheet-engine/src/formatter/lexer.rs`) has two gaps:
- It rejects `A/P` ("A/P is not supported", `Token::ILLEGAL`).
- It reads `[$-409]` as a currency `-` followed by a malformed locale, so `[$-409]` becomes `Token::ILLEGAL`.

Excel results (excel-graph, 2026-10-16):
- probes/text-at-with-tokens A22: `TEXT("abc","x@A/P")` is "xabcA".
- probes/text-at-with-tokens B22: `TEXT("abc","A/P@")` is "Aabc".
- probes/text-at-with-tokens A35: `TEXT("abc","x@[$-409]")` is "xabc".
- probes/text-at-token-sections A20: `TEXT("abc","0;0;0;x@A/P")` is "xabcA".

Ours gives "abc", "abc", "abc" and `#VALUE!`. In a text section, `AM/PM` already prints "AM" (`format_text_value`, T-1656). `A/P` presumably prints "A" there.

For numbers and dates, what `A/P` prints (A or P, and whether the case follows the code) and what `[$-409]0` and `[$-409]d` give are unprobed. Probe them before changing number output.

## Acceptance criteria
- [ ] Probe cells for `A/P` (`a/p`, too) in date formats and for `[$-409]` on numbers and dates, with oracle values. The engine matches them.
- [ ] The four cells above pass, with no new known failures.
- [ ] number-formats.md is updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other LCIDs and BCP-47 tags in `[$-...]`.
