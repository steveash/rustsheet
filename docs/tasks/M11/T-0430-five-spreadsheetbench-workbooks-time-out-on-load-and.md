---
id: T-0430
title: "Five SpreadsheetBench workbooks time out on load and calculation"
milestone: M11
lane: perf
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The five corpus-bench workbooks that exceed the harness's 60 s case timeout load and calculate well within it.

## Context
Found by T-0426 (first crash-only run of corpus-bench, 912 SpreadsheetBench inputs): `corpus-bench/17049/1_17049_input.xlsx`, `41978/1_41978_input.xlsx`, `524-2/1_524-2_input.xlsx`, `52450/1_52450_input.xlsx`, `59932/1_59932_input.xlsx` time out (known failures in eval/baseline/corpus-bench.txt). Fetch with `rustsheet-eval fetch --suite corpus-bench`. Profile each (import vs evaluation; `slowest` in eval/runs/latest/report.json), find the shared cause, and file a task per distinct cause if there are several.

## Acceptance criteria
- [ ] The five cases no longer time out; their `!Timeout` lines leave eval/baseline/corpus-bench.txt via `make eval-baseline`; release timings before/after in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
