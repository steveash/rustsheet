---
id: T-0430
title: "Five SpreadsheetBench workbooks time out on load and calculation"
milestone: M11
lane: perf
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-0465, T-0468, T-0469]
---

## Goal
The five corpus-bench workbooks that exceed the harness's 60 s case timeout load and calculate well within it.

## Context
Found by T-0426 (first crash-only run of corpus-bench, 912 SpreadsheetBench inputs): `corpus-bench/17049/1_17049_input.xlsx`, `41978/1_41978_input.xlsx`, `524-2/1_524-2_input.xlsx`, `52450/1_52450_input.xlsx`, `59932/1_59932_input.xlsx` time out (known failures in eval/baseline/corpus-bench.txt). Fetch with `rustsheet-eval fetch --suite corpus-bench`. Profile each (import vs evaluation; `slowest` in eval/runs/latest/report.json), find the shared cause, and file a task per distinct cause if there are several.

Progress: T-0455 (the cheap number comparison) brought 524-2 under the timeout. For 59932 the
remaining cost is whole-column arrays (T-0456, SPEC-0020, T-0469). Profiled after T-0448/T-0449 (release):
17049 is the same whole-column cost (405 `MAX(IF(F:F=A1,G:G,""))`, T-0456); 41978 is conditional
formats over near-million-row areas evaluated at load and at every recalculation (T-0468);
52450 is 36,600 COUNTIFS with a new criterion each (342 s, T-0465).

## Acceptance criteria
- [ ] The five cases no longer time out; their `!Timeout` lines leave eval/baseline/corpus-bench.txt via `make eval-baseline`; release timings before/after in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
