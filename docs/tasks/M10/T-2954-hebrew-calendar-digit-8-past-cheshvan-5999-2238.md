---
id: T-2954
title: "Hebrew calendar digit 8 past Cheshvan 5999 (2238-11-07): Excel's drifting dates and its #VALUE! limit"
milestone: M10
lane: functions-text
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
`[$-80409]yyyy-mm-dd` prints Excel's Hebrew date for every serial after 2238-11-07 (29 Cheshvan 5999), and `#VALUE!` exactly where Excel does.

## Context
T-2779 modelled the Hebrew calendar (digit 8) by the arithmetic calendar, which matches Excel to 2238-11-01
(probes/t2779-hebrew-saka-more A1:A10). From Kislev 5999 Excel drifts: 2238-12-01 is "5999-03-24" (arithmetic 23),
2238-12-31 "5999-04-25" (23), 2239-09-24 "6000-01-01" (5999-13-24), 2241-10-01 "6001-13-28" (6002-01-25), 2500-01-01
"6260-04-03" (6260-03-29), 4000-01-01 "7760-04-24" (7760-04-11); 5000-01-01 to 9999-12-31 are `#VALUE!`
(t2779-hebrew-saka-more A11:A35, t2779-hebrew-saka-names G1:G9, t1923-calendar-digits A204). Windows' Hebrew calendar
tables end at Gregorian 2239 (year 5999), so Excel's later dates likely read past them; find the rule (probe 2238-11-02..2238-11-30 to confirm where the drift starts, the month
starts of 5999-6002 and bisect the `#VALUE!` start between 4000 and 5000) and model it.
Code: `formatter/calendar_dates.rs` `HEBREW_LAST_SERIAL` (`TODO(T-2954)`), past which ours is `#VALUE!`.

## Acceptance criteria
- [ ] The cells above past 2238-11-07 leave eval/baseline/probes.txt, with a new probe of the drift's month starts.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
