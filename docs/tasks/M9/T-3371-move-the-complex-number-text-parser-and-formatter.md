---
id: T-3371
title: "Move the complex-number text parser and formatter into engineering::complex::parse (complex/mod.rs is at 981 of its 1,000 lines)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`crates/rustsheet-engine/src/functions/engineering/complex/mod.rs` has room under the 1,000-line file limit again
(`scripts/check_file_size.py`). The code that turns an inumber's text into parts and parts back into text moves,
unchanged, into a new `complex/parse.rs`, so the next IM* fix (T-2536, T-2774, T-2538) does not first have to split the
file.

## Context
Sonnet refill 2026-10-31 (`make eval` on origin/main): every visibly-off engineering cell has an open owner (IMTAN:
T-2774, IMSQRT: T-2536; `make unowned-off ALL=1` lists none), so no functions-engineering eval gap is left to cut.
complex/mod.rs is at 981 lines, 19 under the limit, and not in quality/oversized-files.json, so the gate fails any PR
that takes it past 1,000. T-3313 set the pattern for `incomplete_gamma/mod.rs`; `complex/reading.rs` (47 lines, the
text clean-up before parsing) and `complex/trig.rs` are the existing siblings.

What moves (line numbers at origin/main 2026-10-31):
- `enum Suffix` with its `impl Suffix` (`join`) and `impl fmt::Display` (l. 38-66);
- `struct Complex` with its `impl fmt::Display` (l. 68-116), and the `use std::fmt` that only they need;
- `parse_complex_number` (l. 118-237) with its doc comment.
`complex/tests.rs` imports `super::parse_complex_number`, `super::Complex` and `Suffix`: keep them reachable from
`mod.rs` with a `use parse::{...}` (as `reading` and `trig` are used), so the tests and the callers in mod.rs and
trig.rs do not change. The visibility is `pub(super)`, not wider.

## Acceptance criteria
- [ ] The items above, with their doc comments, live in `complex/parse.rs`; their bodies are not changed (a reviewer
      can diff them line by line).
- [ ] complex/mod.rs is at most 800 lines.
- [ ] `make eval-check` (or the full eval comparison of main against HEAD) shows no cell changed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to how inumbers are read or printed (T-3059 probes leading spaces before NBSP and LF).
- The IMTAN/IMSQRT kernels (T-2774, T-2536).
