---
id: T-3370
title: "Move COMBIN, COMBINA, FACT and FACTDOUBLE into a math_and_trigonometry::combinatorics module (mathematical.rs is at 980 of its 1,000 lines)"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`crates/rustsheet-engine/src/functions/math_and_trigonometry/mathematical.rs` has room under the 1,000-line file
limit again (`scripts/check_file_size.py`). The combinatorics code moves, unchanged, into a new
`math_and_trigonometry/combinatorics.rs`, so the next fix in mathematical.rs (ROUND, MOD, the trig kernels) does not
first have to split the file.

## Context
Sonnet refill 2026-10-31 (`make eval` on origin/main): every visibly-off cell of the run has an open owner
(`make unowned-off ALL=1` lists only TEXT(5,"d General"), which T-3342 owns), so no functions-math eval gap is left to
cut. mathematical.rs is at 980 lines (`wc -l`), 20 under the limit; it is not in quality/oversized-files.json, so the
gate fails any PR that takes it past 1,000. T-3313 set the pattern for the same problem in `incomplete_gamma/mod.rs`.

What moves (line numbers at origin/main 2026-10-31):
- `Model::fn_fact` (l. 646) and `Model::fn_factdouble` (l. 659);
- `Model::fn_combin`, `Model::combin_of`, `Model::fn_combina`, `Model::combina_of` (l. 801-891);
- the free functions `combin` (l. 917-944) and `descending_product` (l. 946-963) with their doc comments;
- `COMBIN_MAX_NUMBER` (l. 30), if nothing else in the file uses it.
`fn_percentof` and `percent_of` sit between them and stay. `fn_fact` calls the private helper `Model::number_fn`;
make it `pub(super)` (or `pub(crate)` if a sibling module needs it), not a copy (directive 3).
The new module is declared in `math_and_trigonometry/mod.rs` next to `gcd_lcm` and `multinomial`.

## Acceptance criteria
- [ ] The items above, with their doc comments (the Excel evidence they cite), live in
      `math_and_trigonometry/combinatorics.rs`; their bodies are not changed (a reviewer can diff them line by line).
- [ ] mathematical.rs is at most 850 lines.
- [ ] `make eval-check` (or the full eval comparison of main against HEAD) shows no cell changed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to what COMBIN, COMBINA, FACT or FACTDOUBLE compute (T-2789 probes COMBINA's number_chosen).
- Moving the trig kernels (T-3018, T-3020 and T-2651 have open work in them).
