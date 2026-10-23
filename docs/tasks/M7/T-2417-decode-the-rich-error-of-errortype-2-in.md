---
id: T-2417
title: "Decode the rich error of errorType 2 in the errorType, ptg, subType structure (GROUPBY function-argument #VALUE!)"
milestone: M7
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's rich error with errorType 2 in the `errorType, ptg, subType` `_error` structure is decoded from evidence (or shown to need a different reading), so the cells stop being `#UNDECODED!`.

## Context
Found by T-2397. excel-graph stores the `#VALUE!` that `GROUPBY(...,LAMBDA(w,REGEXEXTRACT("a","[0-9]",-1)))` spills
(ERROR.TYPE 3 through INDEX, probes/groupby-lifted-args A33 and B33) as `t="e"` `#VALUE!` with a `vm` resolving to the
structure `errorType, ptg, subType` = (2, 6, 0) (gen-m7/18b80d3fd292d63a C951:E962; docs/excel-semantics/arrays.md,
rich errors). `rich_error_name` in crates/rustsheet-xlsx/src/import/rich_errors.rs maps no errorType 2, so the
harness reads `#UNDECODED!` and the cells stay pending. errorType here may be a 0-based code (2 = `#VALUE!`, as
`#NULL!`=0, `#DIV/0!`=1), which would conflict with the 1-based codes 8–13 already mapped; that needs probes:
other errors raised inside a GROUPBY/PIVOTBY/MAP LAMBDA (`1/0`, `NA()`, `SQRT(-1)`) and whether ptg differs. Use the
excel-probe skill.

## Acceptance criteria
- [x] A probe records ERROR.TYPE and the stored (errorType, ptg, subType) for at least three error kinds raised in a spilled LAMBDA result.
- [x] The mapping (or the reason none is possible) is in docs/excel-semantics/arrays.md with that evidence; the importer and the snapshot reader share it.
- [x] gen-m7/18b80d3fd292d63a, 0624b684889e9228, 21b4eef18424ce94 and 9bb61f6ae34c7e20 are re-filled; any cell that now
      decodes is scored. Their undecoded cells (T-2397) come from REGEXEXTRACT, EXPAND and GROUPBY. The cached
      excel-graph recalcs of 2026-10-22 store them as `_error` values with errorType 2 (subType 0, 39, 59 or 133,
      some with `ptg` 3, 6 or 8 and an `iftab` field) and errorType 6 (subType 7). Every combination needs evidence
      of the error it stands for (ERROR.TYPE through INDEX).
- [x] `make check` green; reviewers approve.

## Out of scope
- Rich errors outside spilled LAMBDA results.

## Outcome
Probe probes/rich-errors-spilled-lambda (excel-graph, 2026-10-22) spills one error kind per block from MAP, GROUPBY,
EXPAND and plain array arithmetic. Only REGEXEXTRACT's `#VALUE!` is stored as a rich error, `_error(errorType 2,
subType 0)`, and its ERROR.TYPE and VALUETOTEXT through INDEX are 3 and `#VALUE!` (MAP, GROUPBY and EXPAND alike).
`#DIV/0!`, `#N/A`, `#NUM!`, `#REF!`, `#NULL!` and an ordinary `#VALUE!` are plain literals there. So errorType 2 =
`#VALUE!`, as errorType = ERROR.TYPE - 1 predicts. `rich_error_name` maps it (one decoder for the importer and the
snapshot reader), with unit tests for the code and for the `errorType, ptg, subType` structure.
The four gen-m7 cases were re-filled with `--force` from the cached recalcs: 105 cells went from `#UNDECODED!` to
`#VALUE!` and all match ours. Decoding A2004's anchor in 0624b684889e9228 made the rest of its footprint
(AH2004:CV2008, a T-2397 spill hole) scored. That adds 335 new-coverage failures of T-2415's EXPAND family (T-2415
extended to 647 cells). Eval: gen-m7 known failures 637 -> 972 (335 new coverage, 0 regressions); probes +1 case,
0 known failures. Visible parity: 2,845, under the lock of 2,947.
Every errorType 2 combination in the cached recalcs (structures with `iftab` and/or `ptg`; subType 0, 33, 39, 40, 59,
133) decodes through the key-name lookup. No ERROR.TYPE was probed per subType: the T-0030 rule says subType does not
change the error. errorType 6 (subType 7) has no ERROR.TYPE evidence yet and stays undecoded: T-2440.
