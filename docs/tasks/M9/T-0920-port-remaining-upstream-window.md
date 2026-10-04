---
id: T-0920
title: "Port remaining IronCalc fixes: POWER ranges, finite-number parse, --A1:B4, implicit-intersection export"
milestone: M9
lane: functions
status: in-progress
priority: P3
size: M
depends_on: [T-0918]
---

## Goal
Port the upstream commits T-0918 could not apply mechanically (`git apply --check` fails against
the fork): `daf5f6c6` (POWER accepts ranges; conflicts in `model.rs`), `58279cda` + `80eafbbc`
(finite-number parse; conflicts in `text/common.rs`, `string_format.rs`), `9b1ae3fd` (`--A1:B4`;
conflicts in `cast.rs`, `date_and_time.rs`), `9212095d` (implicit-intersection export; conflicts in
`rustsheet-xlsx/src/export/worksheets.rs`).

## Context
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
