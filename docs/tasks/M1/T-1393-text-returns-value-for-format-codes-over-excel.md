---
id: T-1393
title: "TEXT returns #VALUE! for format codes over Excel's complexity limit"
milestone: M1
lane: functions-text
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
TEXT returns #VALUE! for format codes beyond Excel's complexity limit, matching the measured boundaries.

## Context
T-1388 found that Excel rejects far shorter format codes than 255 characters (16 placeholder/literal pairs ok, 17 fail; ~82 unquoted literal characters ok, 84 fail). See docs/excel-semantics/number-formats.md "TEXT format code complexity limit". The model (element weights) is unknown: needs more probing or fitting, hence opus. The engine only caps results at 255 (`format_number`).

## Acceptance criteria
- [x] The probe cases text-date-255-char-limit, text-format-length-limit and text-format-length-limit-b match, or the remaining diffs are recorded as DEVIATION/unprobed
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Three more excel-graph probe rounds (probes/text-format-length-limit-c, -d and -e; 209 cells) pinned a model, recorded in docs/excel-semantics/number-formats.md "TEXT format code complexity limit":
- at most 4 sections;
- at most 32 elements per section, where literal runs and digit runs merge;
- a cost of at most 166 over the code: 2 per placeholder element, 2 per unquoted or escaped literal character (`$` 1), about 1.5 per quoted character, 1 per `;`.

`formatter/complexity.rs` (`exceeds_complexity_limit`) implements it, and TEXT returns `#VALUE!` above it. The three target cases now pass fully. Eval: probes known failures 2797 -> 2759 (fixed 41, 3 new-coverage failures in the new cases, 0 regressions). The three remaining cells are recorded as "Not explained" in the entry: `"d;d;d;d"`, a run of colons after `d`, and an unmatched `[>5]` condition. The quoted-character cost is only bounded (1.08–1.8), so strings of 91–149 characters next to a placeholder are unprobed. Follow-up: T-1434.
