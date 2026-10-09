---
id: T-1220
title: "Fuzz: formula_parse hits the 2 GB RSS limit after 2.5M runs with a 46 MB live heap"
milestone: M1
lane: harness
status: in-progress
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
The nightly `formula_parse` fuzz job runs its 20 minutes without libFuzzer's out-of-memory stop, or the cause is found and fixed in the engine.

## Context
Nightly fuzz run 37881397477 (2026-10-09, `main` b5dd7cd), target `formula_parse`: after 2,521,287
runs libFuzzer stopped with `out-of-memory (used: 2053Mb; limit: 2048Mb)`. The written artifact
`oom-da39a3ee5e6b4b0d3255bfef95601890afd80709` is the empty input (SHA-1 of zero bytes), so no single
input is to blame: RSS climbed slowly (2044 MB to 2048 MB over the last minute) while the live heap
report showed only 46.6 MB live and 33.7 MB in ASan quarantine, with the top contexts allocated
through libstdc++ (libFuzzer's own structures). formula_eval peaked at 1429 MB in the same run.
Decide with evidence (`cargo fuzz run formula_parse -- -runs=3000000 -print_final_stats=1`, RSS
over time, `-malloc_limit_mb`, `ASAN_OPTIONS=quarantine_size_mb=...`) whether this is (a) a leak or
growing cache in the parser (a directive-5 bug: fix it and add a test), or (b) allocator/ASan RSS
growth unrelated to live memory (then adjust the fuzz job's limits in `.github/workflows/fuzz.yml`,
a protected path, with a decision record).

## Acceptance criteria
- [ ] The cause is identified, with the measurements recorded in this task's Outcome.
- [ ] A fix (engine) or a reviewed limit change (workflow plus decision record) lands; a local `formula_parse` run of the same length stays under the limit.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Raising limits without first ruling out a leak in our code.
