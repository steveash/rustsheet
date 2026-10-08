---
id: T-1118
title: "Capture a backtrace when rustsheet-eval dies from a signal in steward and CI runs"
milestone: M1
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
If `rustsheet-eval` ever dies from a signal again (the one SIGSEGV of T-0183), the run leaves a backtrace and the case that was running, so the cause can be found instead of guessed.

## Context
T-0183 could not reproduce the 2026-10-06 steward SIGSEGV in 80 runs (see its Outcome). The next
occurrence is the only evidence we will get, and today it leaves only make's "Segmentation fault".
Options: run the eval under `catchsegv`-like wrapping or with `ulimit -c unlimited` plus
`gdb -batch -ex bt` on the core in the steward's eval step; or log case start/finish per worker
(thread names are already `case:<id>`). The Makefile, scripts/ and .claude/ are protected paths, so
this needs a decision record (`make new-decision`).

## Acceptance criteria
- [x] A deliberately crashing run (e.g. `kill -SEGV` of a running `rustsheet-eval`) leaves a backtrace or core summary and the running case ids in the steward's log.
- [x] Decision record for the protected-path change.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
`make eval`, `eval-check` and `eval-baseline` now run through `scripts/eval_guard.sh` (decision 0041).

The wrapper:
- runs `rustsheet-eval` under `gdb -batch` when gdb can trace it, which prints every thread's backtrace on a signal;
- otherwise names the signal;
- in both modes, lists the cases that started but never ended. These come from a new opt-in case log (`RUSTSHEET_EVAL_CASE_LOG`, `crates/rustsheet-eval/src/case_log.rs`). That log is needed because Linux truncates the `case:<id>` thread names to 15 bytes.

Exit codes pass through unchanged.

Demonstrated on a release build:
- `kill -SEGV` of a running `rustsheet-eval run` under the wrapper gave 142 backtrace frames, four unfinished case ids and exit 255.
- In the no-gdb mode, `kill -ABRT` gave "died from signal SIGABRT", the unfinished cases and exit 134. An external `kill -SEGV` is swallowed by Rust's std stack-overflow handler when gdb is not attached, because no instruction faulted; a real fault still kills the process.
- `scripts/test_eval_guard.py` covers both modes.

gdb adds about 1.5 s and two or three status lines per run. Eval delta: none.
