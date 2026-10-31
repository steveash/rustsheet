---
id: T-3380
title: "Gate A slice 5: math references for eval cells (.math.json sidecars read by the runner, the mpmath fill for probes-numeric sweeps)"
milestone: M0
lane: harness
status: done
priority: P0
size: M
tier: opus
depends_on: [T-3356, T-3357]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Eval cells can carry a 50-digit math value, so decision 0188 R1(c) counts a visibly-off numeric probe cell within
ε = 1e-9 of the math as inside Gate A's envelope, and the R5-deferred probes-numeric sweep cells stop counting against
the milestone exits (SPEC-0029, "Math references for eval cells").

## Context
- SPEC-0029 (T-3355): the `rustsheet-math/v1` sidecar `<case>.math.json` next to the oracle sidecar, each cell with its
  formula text `f`; `envelope` in `crates/rustsheet-eval/src/accuracy.rs` (T-3356); the definitions of T-3357.
- Reuse the fit lab's sweep reader (`tools/src/rustsheet_tools/fitlab/sweep.py`) to parse arguments, and
  `rustsheet-eval cases` to list cases (tools never parse xlsx values).
- T-3359 reads the math value through `envelope` for R1(c); without one a visibly-off numeric cell counts as outside.

## Acceptance criteria
- [x] `rustsheet-accuracy math --suite S` writes math sidecars for every checked one-call cell with a definition and
      parseable arguments, and skips the rest; it is deterministic.
- [x] The runner reads a math sidecar when present, puts the value on `CellResult` (absent from JSON when none) and
      drops a value whose formula text differs; unit tests as in SPEC-0029's test plan.
- [x] Math sidecars for the probes-numeric cases of the functions T-3358 defines are committed; the Outcome reports
      how many visibly-off numeric cells are inside the envelope and how many have no math value. Met for the
      definitions on main (T.DIST, T.INV and RATE, T-3357); T-3358 reruns the command for its own definitions (a
      criterion added to its file).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The report rows that use it (T-3359); definitions beyond T-3358's batch (their own slices).

## Outcome
Done (2026-10-31).
- **Runner** (`runner.rs`): `run_case` reads `<case>.math.json` beside the oracle sidecar (`accuracy::math::math_sidecar`,
  `MathTable::load`) and attaches each cell's value as `CellResult::math` through `MathTable::attach`, so a value
  written for another formula is dropped. The field is absent from the JSON when there is no value, and an unreadable
  sidecar makes the case `InvalidCase`. `MathValue` gains `Deserialize`, the inverse of its `Serialize`, because
  `CellResult` is read back from report JSON. Test: `runner::tests::a_math_sidecar_attaches_math_values_to_matching_cells`
  covers an attached value, a stale value, a missing value, the JSON with and without `math`, the round trip and a
  broken sidecar.
- **Tool** (`tools/src/rustsheet_tools/accuracy/eval_math.py`, `rustsheet-accuracy math --suite S`):
  - It writes a sidecar for every spec case with a math cell, and removes only a sidecar it wrote itself once a case
    no longer has one. The output is deterministic: two runs give identical bytes.
  - A candidate is a scored cell (an oracle or `expected` key) that is one whole call of a function with a definition.
  - Arguments are read with the fit lab's sweep reader. A referenced formula cell counts only when its whole formula
    is arithmetic over number literals, as with exact-input cells such as `=(83892212*2^-28+4861903*2^-54)`.
  - Integer parameters must be given integers, and the argument count must match the definition.
  - Points the generator would drop are skipped.
  - Tests: `tools/tests/test_accuracy_eval_math.py`.
- **Generator fix** (`accuracy/tables.py` `compute`): mpmath's hypergeometric code gives up near 0 or infinity with a
  plain `ValueError`, not `NoConvergence`. That crashed the first probes-numeric run. Such a point is now dropped, like
  any non-converging one, while `NoValueError` still means redraw and a `DomainError` is still recorded.
- **Sidecars:** 16 probes-numeric cases, 25,743 math cells, 3.4 MB.
  - Skipped: 492 unparsed (formula-cell or nested-call arguments, e.g. `RATE(1,PMT(...),10000)`), 52 with a different
    argument count, 37 with a non-integer df, 14 not converged or too slow, and 1 with no single value.
  - Generating them takes 18 s.
- **Envelope count** (`rustsheet-eval run --suite probes-numeric` with the sidecars): 70,117 probes-numeric numeric
  cells are visibly off.
  - 588 have a math value, and all 588 are inside the envelope (within 1e-9 relative of the math).
  - 69,529 have no math value: their functions have no definition yet (most are BETA.DIST, F.DIST and other
    densities), which T-3358 and its slices add.
- **Eval delta:** none. probes-numeric is not gated, the runner only adds a field, and no verdict changes.
- **Review round 1:**
  - The math-sidecar envelope is now built in one place, `tables.math_sidecar_bytes` (with `dump_json` and
    `MATH_FORMAT`), for both table batches and eval cases.
  - A number given for a logical parameter is now skipped, as a non-integer df is, because there is no evidence of how
    each function reads it.
  - The sidecars are byte-identical after both changes.
- **Notes for later slices:**
  - A math domain error says only that the arguments are outside the math's domain, not which Excel error code is
    right. `T.DIST(2,0,FALSE)` is `#DIV/0!` in Excel, so a report that compares errors must not read a domain error as
    a specific code.
  - RATE's `nper` is an integer parameter, so a fractional nper is skipped. Excel's handling of it is not recorded in
    docs/excel-semantics (T-3358 if it widens RATE).

