---
id: T-1327
title: "PIVOTBY relative_to for two-argument functions (SPEC-0006 step 6)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1326, T-1325]
spec: docs/specs/SPEC-0006-groupby-pivotby.md
---

## Goal
PIVOTBY's relative_to 0-4 gives two-argument functions such as PERCENTOF the probed reference sets.

## Context
SPEC-0006 relative_to bullet; anchors P!Y25, P!A49, P!M49, P!Y49, P!A73. With several row or column levels, 3 and 4 are parent totals: probe a two-level case before implementing that part.

From T-1322 (groupby.rs): `aggregate` and `total_row` call the function with one argument; they need a relative-set
parameter and an arity choice (SPEC-0006 `Aggregate { rows, relative }`). GROUPBY with `_xleta.PERCENTOF` still calls it with one argument
(the values are wrong, the key column matches Excel) until T-1325 adds the relative set.

## Acceptance criteria
- [x] Those anchors pass and leave the baseline; the two-level parent totals are probed and recorded in arrays.md.
- [x] Relative sets are built once and shared; built-in PERCENTOF is O(N); a two-parameter LAMBDA is charged against the cell-visit cap, with a DEVIATION entry and test (SPEC-0006 Design step 3).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
New module functions/lookup_and_reference/relative_to.rs. `Model::relative_to` reads 0-4 (text and booleans as numbers;
others `#VALUE!`, also for one-argument functions) into a row selection crossed with a column selection (all, the
cell's item, or its parent: the item's key prefix one field shorter). Items are groups, T-1324's subtotal lines and
the Totals alike: `RelativeSets` builds every set the grid uses once, after the sorts, one pass over the records per
pair of key-prefix levels (`Grouping::prefix_ids`; the level pairs are counted against the visit cap first), and
PIVOTBY's grid cells, subtotal rows and columns, Total row and Total column look theirs up and pass it through T-1325's
`Aggregate { rows, relative }`. PERCENTOF passed by name sums each set once (O(N)); a two-parameter LAMBDA is charged
against the visit cap (DEVIATION entry extended, test `two_parameter_lambdas_are_charged_against_the_visit_cap` in
test_fn_pivotby.rs). Value sorts by a two-argument function use `RelativeTo::of_sort`: rows and columns alike by their
Total cell's set, with the parent group handed down by `Grouping::sort_by_aggregate` (its closure now also gets the
group one level up, tagged with its axis), probed in probes/groupby-pivotby-functions4 (new, 14 questions). An array relative_to is lifted per element (`Model::lifted_relative_to`: `""` or the element's error, no
function call); the signature.rs TODO is resolved. The two-level parent totals and the subtotal lines' sets are probed
(probes/groupby-pivotby-functions P!A101-A441, -functions3) and recorded in arrays.md (T-1327 follow-up), with the
unprobed choices.
Eval delta (on main after T-1324): probes/groupby-pivotby +175 cells (P!Y25:AC31, P!A49:E55, P!M49:Q55, P!Y49:AC55,
P!A73:E79; the case now passes 1432/1432), probes/pivotby-followup +21 (P!A681:A690, P!Z681:Z690, P!AA681; 3124/3124),
groupby-pivotby-functions 1092 -> 3011/3017, -functions2 1543 -> 1646/1672, -functions3 308 -> 700/700. No regressions.
Still failing in those cases: CORREL/PEARSON/RSQ over arrays of different sizes (functions G!B421:B426, functions2
G!B281:B286, G!B301:B306, P!A121 block; T-1707).
Review fixes: functions3 P!A151 was a key sort (two column fields, -2 is Year), so column value sorts were probed anew
(probes/groupby-pivotby-functions4 P!A1-A201): columns sort by their Total row cell's set like rows (3 ties at the
parent's total), so the "own records" branch and the Total-row reuse exception are gone; `{1}` and one-cell references
as relative_to are probed (P!A221-A261). `RelativeSets::new` counts the level pairs and refuses past 2^24 set entries
(`MAX_SET_ENTRIES`) before building any prefix table; the DEVIATION entry names this second refusal and test
`relative_sets_are_capped_before_they_are_built` triggers it. Set ids are `SetId { All, Pivot, Parent { axis, level,
group } }` (grouping.rs, beside `Relative`, so the grouping core imports nothing from group_functions.rs), the sort
axis is `Axis` instead of a bool, and the relative_to parsing is `Model::relative_to_arg` in relative_to.rs.
functions4: 622/622.
