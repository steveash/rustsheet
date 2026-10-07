---
id: T-0183
title: "Intermittent segfault in rustsheet-eval update-baseline (seen once, 2026-10-06 steward run)"
milestone: M1
lane: harness
status: done
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
- [ ] (unmet: not reproducible, see Outcome; the 50-run part holds) A reproducer (case and command) and a backtrace are recorded in this task; the root cause is fixed with a regression test, or, if it is a resource limit, bounded with a DEVIATION entry; 50 consecutive `rustsheet-eval update-baseline` runs exit 0.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Not reproducible; ruled out as far as the code allows, with evidence (2026-10-07, main 06ae340 + T-0049):
- 50 consecutive `rustsheet-eval update-baseline` runs on every suite including the fetched poi
  (336 workbooks) exited 0 (idle machine, ~2.3 s each); 30 more ran under heavy CPU contention
  (two concurrent `make check`) without a signal (they exited 1 only because that binary predated
  T-0279 and refused its regressions); 15 `run --timeout 0` runs, which leave every case's worker
  thread running while the process exits, exited 0.
- Code audit: no `unsafe` in the workspace, no C dependency in rustsheet-eval's tree (zip 0.6
  without bzip2/zstd, deflate through the pure-Rust `miniz_oxide`). Every engine evaluation and xlsx import runs on a
  `case:<id>` thread with a 256 MB stack under `catch_unwind`; the work on rayon threads
  (`prepare`: quick-xml snapshot, the lexer-only `shift_formula_text`, the iterative volatility
  scan) does not recurse. A stack overflow in safe Rust is caught by std's guard-page handler,
  which prints "thread 'case:<id>' has overflowed its stack" before the process dies; the
  steward's log had no such line.
- No reproducer or backtrace exists, so the first acceptance criterion's reproducer part is unmet;
  T-1118 makes the next occurrence leave one. Eval delta: none.
