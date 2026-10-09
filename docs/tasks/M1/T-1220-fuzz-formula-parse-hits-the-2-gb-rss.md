---
id: T-1220
title: "Fuzz: formula_parse hits the 2 GB RSS limit after 2.5M runs with a 46 MB live heap"
milestone: M1
lane: harness
status: done
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
- [x] The cause is identified, with the measurements recorded in this task's Outcome.
- [x] A fix (engine) or a reviewed limit change (workflow plus decision record) lands; a local `formula_parse` run of the same length stays under the limit.
- [x] `make check` green; reviewers approve.

## Out of scope
- Raising limits without first ruling out a leak in our code.

## Outcome
Cause (b): RSS growth inside ASan, not a leak in the engine. Local runs of `formula_parse` used
the same options as the job (`-dict -max_len=512 -rss_limit_mb=2048`), a seeded corpus and nightly
cargo-fuzz 0.13.2 on 4 cores. RSS is shown as (runs, MB):

| run | settings | RSS over the run | peak | exec/s |
|---|---|---|---|---|
| A | ASan defaults, 900 s | (0.1M, 1000) → (1.08M, 1043) → (1.87M, 1392) → (2.47M, 1662) | 1662 MB | 2746 |
| B | ASan `quarantine_size_mb=16`, 900 s | (0.2M, 358) → (1.30M, 820) → (3.07M, 1358) | 1358 MB | 3412 |
| C | no sanitizer (`-s none`), 300 s | (0.04M, 45) → (6.28M, 60) | 60 MB | 20851 |
| D | ASan `malloc_context_size=0`, 480 s | (0.23M, 688) → (2.60M, 688) | 688 MB | 5404 |

- C shows the engine's memory is flat over 6.28M parses, so the parser has no leak or growing
  cache. A and B grow at about 400 MB per million runs even with a small quarantine.
- D shows that recording no allocation stacks stops the growth. The growth was ASan's stack depot,
  which keeps every distinct allocation stack, and recursive parsing of fuzzed input keeps making
  new ones. The CI live-heap report fits this: 46.6 MB live at 2 GB RSS.
- Fix: `.github/workflows/fuzz.yml` sets `ASAN_OPTIONS=malloc_context_size=0` on the fuzz step for
  all targets (decision 0046). Run D, the same length as the failing job, stays at 688 MB, far
  under the limit. No engine change, so no eval delta.
