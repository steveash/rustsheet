---
id: T-1740
title: "Displayed text: a fraction format whose numerator overflows is 255 #"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2029, T-2030]
---

## Goal
A fraction format whose numerator Excel cannot show displays 255 `#`, as Excel's displayed text does; the rule's threshold is established first.

## Context
`?/100`: 255 `#` on 1000, 999.9999, 45000.75, -45000, 123456.789, -123456.789, 1234567890 and -1E15, while
99.5 is "9950/100" and 61 "6100/100" (every probed numerator of 100000 or more overflows, every one below
shows). `?/?` and `??/??`: 255 `#` on 1E15 and -1E15, and `?/?` on 123456789012345, yet `?/?` on 1234567890
is "1234567890/1". Mixed fractions with a whole part (`# ?/?`, `# ??/??`) show 1234567890 as "1234567890    ".
The threshold is unknown: probe numerators around 1E5 (fixed denominator) and 1E10..1E15 (`?/?`) with
excel-probe before implementing. Ours prints digits (`?/100` on 1000 "10/100"). TEXT() on an overflowing
fraction is unprobed (no gen-m10 TEXT() cell overflows; date and clock codes give `#VALUE!` where the display is
`#`): add `TEXT(1000,"?/100")` to the probe. The other fraction
failures (wrong digits) are T-2029 (fixed denominators), T-2030 (placeholder denominators) and T-0982
(gap triage 2026-10-19: the engine has no fraction section yet, so this task waits for them).
Evidence: suite gen-m10 (excel-graph, 2026-10-16, T-1611): `make eval` report, "Displayed text" section, which lists each failing text cell (Excel vs ours). Text cells are scored apart from values and are not baselined (decision 0068); the TEXT() cells on sheet T that use the same codes are value known failures in eval/baseline/gen-m10.txt. Formatter: crates/rustsheet-engine/src/formatter/ (shared by `Workbook::formatted` and TEXT). docs/excel-semantics/number-formats.md.

## Acceptance criteria
- [ ] The gen-m10 text cells under these codes pass in the report's "Displayed text" section (or the remaining ones are filed); the matching TEXT() cells on sheet T leave eval/baseline/gen-m10.txt; no regressions.
- [ ] number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
