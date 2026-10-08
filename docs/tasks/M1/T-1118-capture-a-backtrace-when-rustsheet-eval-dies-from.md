---
id: T-1118
title: "Capture a backtrace when rustsheet-eval dies from a signal in steward and CI runs"
milestone: M1
lane: harness
status: in-progress
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
- [ ] A deliberately crashing run (e.g. `kill -SEGV` of a running `rustsheet-eval`) leaves a backtrace or core summary and the running case ids in the steward's log.
- [ ] Decision record for the protected-path change.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
