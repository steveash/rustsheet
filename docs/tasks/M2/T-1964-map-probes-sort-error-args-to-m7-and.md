---
id: T-1964
title: "Map probes/sort-error-args to M7 and date-math-lift-driver-edges G43 to M6 in probe-milestones.tsv (decision record)"
milestone: M2
lane: harness
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
The two M0-M2 probe failures whose subject is a later-milestone function count toward that
function's milestone, as decisions 0051 (item 2) and 0058 require: `probes/sort-error-args` (all
cells) goes to M7, and the one cell `probes/date-math-lift-driver-edges` Sheet1!G43 goes to M6.

## Context
`make eval` on main 30bee011 ("Probe milestones (decision 0051)") puts 45 known-failure cells in
M0-M2. Two cases are there only because of their case.json tag:
- `probes/sort-error-args` (23 cells: H57:J59, H61, H65, H69:J71, H97:H99) is tagged `M2` but was
  written by the M7 task T-1850. Its subject is SORT and SORTBY only (catalog M7 in
  eval/catalog/scope.json). It has no M2 subject function, so the 0051 item 2 exception does not
  apply. The twin `sort-omitted-args` was mapped to M7 by decision 0058 on the same grounds. T-1902
  (M7, functions-lookup) owns the fix.
- `probes/date-math-lift-driver-edges` Sheet1!G43 is `=INDEX(DATEDIF(46000,45000,{"d";#N/A}),2)`.
  Excel gives #N/A and ours gives #NUM!. Its subject is DATEDIF (catalog M6), and INDEX only
  observes it (0051's wrapper list). The rest of the case tests the SPEC-0011 driver and keeps
  its M2 tag. A cell line follows the 0058 practice ("cell lines, by subject"). T-1430 (M6,
  functions-date) owns the fix.

eval/milestones/ is protected (decision 0044), so the change needs a new decision record
(`make new-decision`) that amends 0051/0058 and lists the lines and their reasons. The other
five M0-M2 failing cases (legacy @, SUBTOTAL/TAKE static analysis, sheet ops) test M2 engine
features. M2 engine tasks own them (T-1852, T-1910, T-1911, T-1914, T-1915, T-1935, T-1936), and they
stay M2.

## Acceptance criteria
- [ ] A new accepted decision record in docs/decisions/ amends 0051/0058. It names the two lines
  and cites the catalog milestones and the 0051 item 2 exception check for each.
- [ ] eval/milestones/probe-milestones.tsv gains `probes/sort-error-args.case.json	M7	SORT, SORTBY`
  and the cell line `probes/date-math-lift-driver-edges.case.json	Sheet1!G43	M6	DATEDIF`. No other
  line changes.
- [ ] `make eval` report "Probe milestones (decision 0051)": M0-M2 known failures drop by exactly 24
  (45 -> 21 at 30bee011, or the same delta at the merge base). M6 rises by 1 and M7 by 23.
  eval/baseline is unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing SORT/SORTBY (T-1902) or DATEDIF's error order (T-1430).
- Re-tagging any other case, and editing case.json files (oracle input hashes).
