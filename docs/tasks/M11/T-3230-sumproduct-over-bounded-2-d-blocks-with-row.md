---
id: T-3230
title: "SUMPRODUCT over bounded 2-D blocks with row and column masks (calipers pv26_002: 2,000 calls over I11:AW1450)"
milestone: M11
lane: perf
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
SUMPRODUCT over a bounded block multiplied by a row mask and a column mask costs about its matching
cells per call, not the whole block, with bit-identical results. The form is
`SUMPRODUCT(data*(header_row=x)*(key_column=y))`.

## Context
After T-3227, calipers `pv26_002` calculates in 71 s (release, 2026-10-30) on top of a 17 s load.
Its remaining time is SUMPRODUCT, by stack sampling: `fn_sumproduct`, `evaluate_operator_chain`,
`evaluate_range`. About 1,500 calls have the form
`IFERROR(SUMPRODUCT('Quarterly ARR Rollup'!$I$11:$AW$1450*('Quarterly ARR Rollup'!$I$6:$AW$6=D$3)*('Quarterly ARR Rollup'!$C$11:$C$1450=$C6)),"")`.
Each one builds and multiplies three arrays of 1,440 x 41 cells, about 59,000 elements each. The
header row and key column masks change per call, and the data block is shared.

Possible approach, to be checked: evaluate the broadcast masks first. Only the cells where both
masks are nonzero can contribute, but every element's reading must stay the walk's. An error
element anywhere in the data block is the result even where a mask is 0, and text is 0. So a
skipped cell must provably contribute exactly 0 and hide no error. A cached error-and-number
summary of the shared data block would answer that.

## Acceptance criteria
- [x] Equivalence tests against today's evaluation: errors and text in masked-out cells, and
      shape mismatches.
- [x] calipers `pv26_002` calculation time in the Outcome. If load plus calculation is now within
      60 s, its and `pv26_2_002`'s `exclude` entries go and the calipers baseline is updated.
- [x] `make check` green; reviewers approve.

## Out of scope
- `pv26_002`'s 17 s load.

## Outcome
- **New path.** A SUMPRODUCT argument `block*mask*...` is read without building its arrays
  (`read_masked_block`, sumproduct.rs). The argument must be a `*` chain whose first operand is a
  range. The block's cells must already be cached: `ifs_range_values`, the T-0449 rule that no read
  evaluates anything. Every later operand must be a mask: a 1-row array as wide as the block, or a
  1-column array as tall, of booleans or errors. Comparisons give exactly that.
  - A block's cells are read as numbers once per pass, the way `*` reads them. Each block also
    keeps its first cell that is no number (`BlockNumbers`, `block_numbers` in ifs_cache.rs).
  - A mask element is 1 or 0, so each product element is its cell's number or a zero. The zero
    leaves the total unchanged, as the walk's does.
  - The first error element, row by row, is found from the positions alone. That is the block's
    first error, or a mask's first error at the start of its column (column mask) or row (row
    mask). At equal positions the block wins, then the masks in operator order, as `*` picks the
    left operand's error first.
- **Evaluation order.** The operands are evaluated one by one, as `evaluate_node_in_context`
  evaluates the chain. Only the block's reads move, and those evaluate nothing. When an operand is
  not a mask, the operators run over the operands read so far and then over the rest of the chain,
  with the same per-link charges (`charge_chain_link`, now shared with `evaluate_operator_chain`).
- **Timings,** release, `evaluate()` after load, on the same machine:

  | file | before | after |
  |---|---|---|
  | calipers `pv26_002` | 38.3 s | 12.6 s |
  | calipers `pv26_2_002` | 37.1 s | 13.4 s |

  Each takes about 9 s to load. Under the parallel harness, `rustsheet-eval run --suite calipers`,
  each case takes 44 s, inside the 60 s timeout, so their `exclude` entries are gone. Both
  pass every scored cell (2.75M cells). The calipers baseline is not updated:
  `update-baseline --suite calipers` refuses one regression from main. It is IRR in
  `formulas_financial` E132, after T-3120, filed as T-3246. calipers is not gated, so the two cases
  are recorded with the next baseline update. After evaluation,
  all 1,765,265 cell values of each workbook are identical to `origin/main`'s.
- **Tests** (test_sumproduct_masked.rs) compare each formula with the pass caches off
  (`no_pass_caches`), including whether the cell is marked circular:
  - three blocks (clean; text, `#DIV/0!` and `#NUM!` cells; formulas not evaluated yet), header
    rows and key columns with and without errors, five operand orders;
  - errors of the block and the masks meeting at one position;
  - operands that are no masks: a mismatched width, a 2-D or one-cell range, a number, an array
    constant, a mask already multiplied out, text, `/`;
  - other arguments beside a masked one, IFERROR around it, and a block too small to cache;
  - a cycle through the block;
  - one value worked out by hand.

  A test-only counter checks that the masked path ran.
- **Mutations:** each of these fails a test: ignoring a mask's error, ignoring the block's first
  error, letting a later operand's error win at an equal position, and placing a row mask's error
  at its row number instead of the start of its row.
- **Eval delta:** calipers runs `pv26_002` and `pv26_2_002` again (all cells pass; recorded once
  T-3246 unblocks the ratchet). The gated suites do not change.
- **Follow-up:** T-3246.
