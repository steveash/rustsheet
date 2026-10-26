---
id: T-0430
title: "Five SpreadsheetBench workbooks time out on load and calculation"
milestone: M11
lane: perf
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0455, T-0468, T-0469, T-0477]
---

## Goal
The five corpus-bench workbooks that exceed the harness's 60 s case timeout load and calculate well within it.

## Context
Found by T-0426 (first crash-only run of corpus-bench, 912 SpreadsheetBench inputs): `corpus-bench/17049/1_17049_input.xlsx`, `41978/1_41978_input.xlsx`, `524-2/1_524-2_input.xlsx`, `52450/1_52450_input.xlsx`, `59932/1_59932_input.xlsx` time out (known failures in eval/baseline/corpus-bench.txt). Fetch with `rustsheet-eval fetch --suite corpus-bench`. Profile each (import vs evaluation; `slowest` in eval/runs/latest/report.json), find the shared cause, and file a task per distinct cause if there are several.

Progress: T-0455 (the cheap number comparison) brought 524-2 under the timeout. For 59932 the
remaining cost is whole-column arrays (T-0456, SPEC-0020, T-0469). Profiled after T-0448/T-0449 (release):
17049 is the same whole-column cost (405 `MAX(IF(F:F=A1,G:G,""))`, T-0456); 41978 is conditional
formats over near-million-row areas evaluated at load and at every recalculation (T-0468);
52450 is 36,600 COUNTIFS (342 s): first thought a new criterion each (T-0465), it
turned out to be its linked-workbook criteria ranges built as arrays (T-0477).

## Acceptance criteria
- [x] The five cases no longer time out; their `!Timeout` lines leave eval/baseline/corpus-bench.txt via `make eval-baseline`; release timings before/after in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
All five cases now calculate within the timeout; none is a `!Timeout` line in
eval/baseline/corpus-bench.txt (release timings, load + evaluate):
| case | cause | fix | before | after |
|---|---|---|---|---|
| 524-2 | number comparisons cut to 15 digits | T-0455 | over 60 s | 44 s |
| 17049 | 405 `MAX(IF(F:F=A1,G:G,""))`, million-row arrays | T-0469 (SPEC-0020) | over 120 s | 0.27 s |
| 59932 | 150 `MIN/MAX(IF($D:$D=K2,$F:$F))` | T-0469 | about 55 s | 2.6 s |
| 41978 | conditional formats over near-million-row areas, at load and at every `evaluate()` | T-0468 (SPEC-0021) | 184 s + 191 s | 5 ms + 6 ms |
| 52450 | 36,600 COUNTIFS building a linked sheet's whole columns as arrays, then `#VALUE!` | T-0477 | 3 s + 338 s | 3 s + 48 ms |

Each fix kept results bit-identical (0 regressions in every gated suite each time); the cases now
score 405/405 (17049), 14,734/32,948 (59932: the rest are linked-workbook VLOOKUPs, T-0457),
24/24 (41978) and 37,641/37,641 (52450). Follow-ups: T-0476 (bound a style read's conditional-format
work), T-0465 (value indexes, calipers pv26_002), T-0470 and T-0471 (more of SPEC-0020).
