---
id: T-0183
title: "Intermittent segfault in rustsheet-eval update-baseline (seen once, 2026-10-06 steward run)"
milestone: M1
lane: harness
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The crash is found and fixed, or ruled out with evidence: `make eval-baseline` (and every other `rustsheet-eval` subcommand) never dies from a signal on the committed suites.

## Context
In the 2026-10-06 steward run, `make eval-baseline` on main c134e7b+#181+#182 (release build, all suites including the fetched poi) ended with `make: *** [Makefile:40: eval-baseline] Segmentation fault`. The last stderr lines were the importer's "Unexpected feature XmlNode { name: \"Fallback\" ... }" font warnings from a POI workbook. Two `rustsheet-eval run` and four `update-baseline` reruns on the same tree exited 0, so the crash is intermittent.
A segfault in safe Rust on untrusted input points to a stack overflow (deep recursion in the parser, evaluator or XML walk, possibly on a worker thread with a smaller stack) or to nondeterminism, such as thread scheduling or a HashMap iteration order that changes the recursion path. That makes it a CLAUDE.md directive 5 issue as well as a harness one.
Start by running `update-baseline` in a loop (e.g. 50 times), with `ulimit -c unlimited` or under `gdb -batch -ex run -ex bt`, and capture the backtrace. Check which suite or case was running, and whether the eval runs cases on threads with a non-default stack size.

## Acceptance criteria
- [ ] A reproducer (case and command) and a backtrace are recorded in this task; the root cause is fixed with a regression test, or, if it is a resource limit, bounded with a DEVIATION entry; 50 consecutive `rustsheet-eval update-baseline` runs exit 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
