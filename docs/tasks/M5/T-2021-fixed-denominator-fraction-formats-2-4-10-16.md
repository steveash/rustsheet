---
id: T-2021
title: "Fixed-denominator fraction formats (# ?/2, # ?/4, # ?/10, # ??/16, ?/100)"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-2020]
---

## Goal
Fraction formats whose denominator is a number (`# ?/2`, `# ?/4`, `# ?/10`, `# ??/16`, `?/100`) render
like Excel in `TEXT()` and in the displayed text, using the fraction renderer T-2020 adds.

## Context
Gap triage 2026-10-19: same root cause as T-2020 (no fraction support in
`crates/rustsheet-engine/src/formatter/`), split off because the numerator rule is different and
needs no search: numerator = the fraction part times the denominator, rounded half up
(`floor(f*d + 0.5)`); a numerator equal to the denominator carries into the whole part, and a zero
numerator leaves blank padding. This rule gives the Excel numerator in all 314 numeric, non-overflow
gen-m10 text cells of these five codes (triage check over the case and oracle files). Examples:
`TEXT(45000.75,"# ?/10")` "45000 8/10" (ours "450 0/11"), `TEXT(1.5,"# ?/4")` "1 2/4", `TEXT(0.9999999999,"# ?/4")`
"1    ", `TEXT(-1/7,"?/100")` "-14/100", `TEXT(0,"?/100")` "0/100", `TEXT(61,"# ??/16")` "61      ".
Rules and cells: docs/excel-semantics/number-formats.md "Fraction formats".
- Value known failures, eval/baseline/gen-m10.txt (16 lines, `TEXT()` on sheet T):
  039accabcfe1e6c7 T!A6; 03a284c57e24ff37 T!A11; 05c1c646c95a2d60 T!A1, A4; 4398098060a7fa9d T!A14, A16;
  8e073c5b2145567a T!A11, A22; 9a8b7d68507d7ada T!A1; bfb1a4b09b0c998e T!A19; bfbcd16cee266336 T!A22, A25;
  e1c29d3fe448682c T!A24, A26; eec351e03ccedfe0 T!A6, A9.
- Displayed text: the five codes fail on 329 of their 360 gen-m10 text cells (report "Displayed text").

## Acceptance criteria
- [ ] The 16 gen-m10 lines above disappear from eval/baseline/gen-m10.txt (`make eval-baseline`) with no
      new F lines in any baseline.
- [ ] In the report's "Displayed text" section no gen-m10 cell under these five codes fails, except the
      255 `#` overflow cells (`?/100` on 1000 and up, T-1740).
- [ ] Unit tests cover the half-up numerator, the carry into the whole part and the zero numerator.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 255 `#` overflow threshold (T-1740); digit-count denominators (T-2020).
