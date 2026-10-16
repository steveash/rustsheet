---
id: T-1325
title: "GROUPBY/PIVOTBY function arrays and eta names: HSTACK/VSTACK of functions, PERCENTOF (SPEC-0006 step 4)"
milestone: M7
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-1322, T-1273]
spec: docs/specs/SPEC-0006-groupby-pivotby.md
---

## Goal
GROUPBY and PIVOTBY accept eta-reduced function names and HSTACK/VSTACK arrays of functions, and PERCENTOF gets its relative set.

## Context
SPEC-0006 function forms; anchors G!A1 (and every eta anchor), G!S97, G!AB97, G!S121, G!AB121, P!A121. Depends on T-1273's eta call path; do not add a second one.

From T-1322: eta names already reach GROUPBY through `Model::lambda_caller` (the plain eta anchors pass); what is left is
function arrays (HSTACK/VSTACK of functions), PERCENTOF's relative set, and probes/groupby-lambda-forms G!K193:K198:
`LAMBDA(x,Data!D2)` is `#VALUE!` in every cell in Excel, but the LAMBDA call returns the cell's value, so `aggregate`
(groupby.rs, `TODO(T-1325)`) cannot tell it from a value; it needs the call to keep a reference result.

## Acceptance criteria
- [x] The eta-form anchors of probes/groupby-pivotby sheet G and probes/m7-argument-modes F33-H33 pass and leave the baseline.
- [x] Relative sets are built once and shared; built-in PERCENTOF is O(N); a two-parameter LAMBDA is charged against the cell-visit cap, with a DEVIATION entry and test (SPEC-0006 Design step 3).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
New module functions/lookup_and_reference/group_functions.rs holds the `function` argument of GROUPBY and PIVOTBY: an
array of functions written with HSTACK, VSTACK or TRANSPOSE is read from the formula as a grid of functions and labels
(`Model::function_elements`; a LAMBDA cannot be an array element), classified (one function; a row of them as columns
under a label row; a column of them as rows after a label column; values beside or below the functions as custom
labels; anything else `#VALUE!`), paired with the value columns (n over 1, 1 over n, n over n pairwise, else
`#VALUE!`) and laid out by GROUPBY and PIVOTBY through `Functions::shape`. Each function is called through the existing
`Model::lambda_caller` path (T-1560/T-1466) with its arity: an eta name its probed minimum argument count (two for
PERCENTOF, CORREL, PEARSON, RSQ, COVARIANCE.P, IF, ROUND, SUMIF; `#VALUE!` for NOW, TEXTJOIN, XLOOKUP; one otherwise,
follow-up T-1706), a LAMBDA one or two by its required parameters. The caller reads the body in reference context
(`LambdaCaller::keeping_references`), so a reference result is `#VALUE!` (groupby-lambda-forms G!K193:K198).
Two-argument functions get the relative set (`Aggregate { rows, relative }`; GROUPBY: every kept row). PERCENTOF passed by
name sums each relative set once per value column (`percent_of`, extracted from the PERCENTOF function, O(N)); a
two-parameter LAMBDA is charged against the shared cell-visit cap before each call (`too_many_visits`, extracted from
if_ifs.rs), DEVIATION entry in arrays.md, test `two_parameter_lambdas_are_charged_against_the_visit_cap`. Detected
headers are shown with a row of several functions (probed). PIVOTBY gets the function arrays too (P!A121); its
two-argument functions and relative_to stay `#N/IMPL!` until T-1327.
Evidence: three new probe cases filled by excel-graph (probes/groupby-pivotby-functions, -functions2, -functions3, 97
questions); findings and the unprobed choices are in docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY" (T-1325
follow-up).
Eval delta: probes/groupby-pivotby +138 cells (G!S121:U128, G!AB121:AD132, G!T97:T102, P!A121:I128),
probes/groupby-lambda-forms +6 (G!K193:K198, now 491/491); new cases groupby-pivotby-functions 1092/3017,
-functions2 1543/1672, -functions3 308/700 passing, the rest PIVOTBY two-argument/relative_to (T-1327) and
CORREL/PEARSON/RSQ over different sizes (T-1707). No regressions (measured on main after T-1324, whose subtotal paths
`line_row` and `grid_row` carry the function arrays too; PIVOTBY column subtotals with several output columns stay
`#N/IMPL!` with T-1700). Follow-ups: T-1706 (every eta name's
minimum argument count), T-1707 (CORREL/PEARSON/RSQ).
