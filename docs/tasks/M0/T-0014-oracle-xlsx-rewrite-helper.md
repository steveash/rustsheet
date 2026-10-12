---
id: T-0014
title: "Oracle tooling: one xlsx part-rewrite helper; rels-based formula-part scan"
milestone: M0
lane: quality
status: done
priority: P3
size: S
tier: opus
depends_on: [T-0013]
---

## Goal
One definition for "rewrite parts of an xlsx package" in tools/oracle, and a canary refusal scan
that cannot miss worksheets stored at nonstandard paths.

## Context
Review follow-ups from T-0013:
- `force_full_calc_on_load` (oracle/backends.py) and `embed_canary` (oracle/canary.py) both
  unzip, patch and rezip a package; the former hardcodes `xl/workbook.xml`, the latter resolves
  the workbook part from `_rels/.rels`. Share one helper and one workbook-part lookup.
- The `SHEETS()`/`INFO()` refusal scan in `embed_canary` covers the workbook part and
  `<wb_dir>/worksheets/`; resolve worksheet/macro-sheet targets from the workbook rels instead
  (also XLM `GET.WORKBOOK` in defined names).
- Optional: refuse packages whose total uncompressed size exceeds a cap before reading parts.

## Acceptance criteria
- [x] A single helper rewrites package parts; both callers use it and the same workbook lookup.
- [x] Refusal scan follows rels; tests cover a worksheet at a nonstandard target.
- [x] `make check` green; reviewers approve.

## Out of scope
Changing canary contents or the Graph session flow.

## Outcome
New `tools/src/rustsheet_tools/oracle/package.py` is the one place that reads and rewrites an xlsx package: `read_parts`/`write_parts`, a rels-based `workbook_part`, `formula_parts` (the workbook plus every worksheet/macrosheet/dialogsheet its rels target) and `patch_workbook`. `force_full_calc_on_load` (no longer hardcodes `xl/workbook.xml`) and `embed_canary` both use it. The canary's SHEETS()/INFO() scan follows the rels, so a worksheet at a nonstandard path is covered, and `GET.WORKBOOK(` (XLM, defined names and macro sheets) is now refused too, as a conservative refusal rather than an Excel-semantics claim. Tests added for a nonstandard sheet path, GET.WORKBOOK and `force_full_calc_on_load`. Every scanned part is parsed (`package.parse_part`: an expat pass refuses any DTD or entity declaration in whatever encoding, then ElementTree) and the patterns run over the unescaped text and attribute values (`text_runs`), so character references, comment/CDATA splits and UTF-16 parts cannot hide a function name, and formulas outside cells (data validations, defined names) are covered. The scan fails closed: a part that does not parse, a rels entry pointing at a missing sheet part, duplicate zip entry names, or a non-UTF-8 part the canary must edit are errors; everything under `worksheets/` is scanned too. Handed off by a sonnet run after two security rounds; an opus run applied the parse-based scan. Not covered (unchanged from before): table parts' `calculatedColumnFormula`. The optional total-size cap was not added (T-0054); spec2xlsx's own zip loop is T-0053. No eval change.
