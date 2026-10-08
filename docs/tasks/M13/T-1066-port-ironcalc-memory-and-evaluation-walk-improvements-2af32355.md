---
id: T-1066
title: "Port IronCalc memory and evaluation-walk improvements (2af32355, 18f4d524, f06531ac)"
milestone: M13
lane: perf
status: blocked
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
RustSheet gets the memory and evaluation-walk savings IronCalc landed upstream, without behavior
change: recalculation walks only formula cells (and skips sheets without formulas), a cell is
smaller, and xlsx import reads rows with fewer allocations.

## Context
Found by the 2026-10-07 steward run (`scripts/upstream-log.sh`, past 4deab8f6). Upstream commits
(port each with `Ported-From: ironcalc@<sha>`):
- `2af32355` evaluation.rs: replace `all_positions` with formula-only positions; our fork still has
  `all_positions`/`dynamic_spills` in `crates/rustsheet-engine/src/evaluation.rs`.
- `18f4d524` types.rs/model.rs/sheet_data.rs and xlsx worksheets.rs: box the origin and message
  of `FormulaValue::Error` (Cell 80 -> 48 bytes upstream).
- `f06531ac` xlsx `import/sheet_data.rs`, `worksheets.rs`: keep rows in read order, fast path
  for simple A1 refs, reuse row/reference/value buffers.
Our fork diverged (spills, rich errors, 3-D refs), so adapt rather than cherry-pick. The fourth
upstream commit `09e81f53` only adds `#[allow(clippy::approx_constant)]` in bessel_k.rs; our lint
setup already passes there, so it is not ported (no blanket allows).
Performance work is Phase 2 (decision 0027), so this task starts blocked.

## Acceptance criteria
- [ ] The three commits are ported (split per commit if the diff passes size M).
- [ ] Eval: 0 regressions, 0 fixed (no behavior change); `size_of::<Cell>()` recorded before/after.
- [ ] Import of a large constants-only workbook measured before/after (numbers in the PR).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other memory layout changes not in these commits.

## Blocked because
Deferred to Phase 2 (decision 0027). Unblock when docs/status.md has the line
`**Phase 1:** exited <date>`.
