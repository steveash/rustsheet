---
id: T-0920
title: "Port remaining IronCalc fixes: POWER ranges, finite-number parse, --A1:B4, implicit-intersection export"
milestone: M9
lane: engine
status: in-progress
priority: P3
size: M
tier: opus
depends_on: [T-0918]
---

## Goal
Port the upstream commits T-0918 could not apply mechanically (`git apply --check` fails against
the fork): `daf5f6c6` (POWER accepts ranges; conflicts in `model.rs`), `58279cda` + `80eafbbc`
(finite-number parse; conflicts in `text/common.rs`, `string_format.rs`), `9b1ae3fd` (`--A1:B4`;
conflicts in `cast.rs`, `date_and_time.rs`), `9212095d` (implicit-intersection export; conflicts in
`rustsheet-xlsx/src/export/worksheets.rs`).

## Context
Lane: `engine` (decision 0022): the ports span `cast.rs`, `model.rs`, `date_and_time.rs`, text formatting and the xlsx export, so no single functions lane owns them.

Use the patch recipe from T-0911 (`format-patch`, rewrite paths, `git apply --reject`) and resolve
hunks by hand. Port each as its own commit with `Ported-From: ironcalc@<sha>`; split the task if a
single port exceeds size S.

## Acceptance criteria
- [ ] Each commit ported or a note on why not.
- [ ] Upstream Excel-authored workbooks (POWER.xlsx, MINUS_MINUS.xlsx) cited as evidence; `make check` green.

## Follow-ups from T-0918 review (minor)
- Oracle-confirm INDIRECT's `a1` coercions (`0`, `""`, `"x"`) and text-function array broadcasting
  for EXACT/FIND/LEN/REPT/SEARCH/SUBSTITUTE/TRIM/CLEAN/PROPER/REPLACE (only LEFT/MID/RIGHT have
  Excel-authored evidence); add `docs/excel-semantics/` entry for R1C1 offset wrapping.
- Explicit `is_finite` check in `rept_element`; fold `parse_reference_r1c1` and the new R1C1 parser together.

## Progress
Partial (2026-10-04): ported `daf5f6c6` (POWER accepts ranges; evidence: Excel-authored POWER.xlsx, now
in calc_tests) and `9212095d` (implicit-intersection export; needed for the POWER.xlsx round-trip).
POWER.xlsx adds 51 checked cells; 4 are NumericUlp (Sheet1!D3, F9, inside_a_lambda!C3/C4: `powf` vs
Excel's pow) and are baselined as new coverage. Still open: `58279cda`+`80eafbbc` (finite-number
parse) and `9b1ae3fd` (`--A1:B4`, MINUS_MINUS.xlsx; conflicts in `cast.rs`, `date_and_time.rs`),
plus the T-0918 follow-ups above.

Update (2026-10-05): `9b1ae3fd` (`--A1:B4`, MINUS_MINUS.xlsx) was ported by T-0132, except its
cast-to-time hunk, which T-0146 covers. Still open here: `58279cda` + `80eafbbc` (finite-number
parse) and the T-0918 follow-ups.

Review follow-ups (2026-10-04): match Excel's 1-ulp result for odd roots (`-1.9999999999999998`, not
`-2`) in `arithmetic::power`; oracle-probe POWER overflow (#NUM! vs #DIV/0!), the odd-root tolerance
(`POWER(-8,0.333333333333333)`) and the `^` operator (see `docs/excel-semantics/numbers.md`).
