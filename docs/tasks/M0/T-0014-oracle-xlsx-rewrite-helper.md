---
id: T-0014
title: "Oracle tooling: one xlsx part-rewrite helper; rels-based formula-part scan"
milestone: M0
lane: quality
status: todo
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
- [ ] A single helper rewrites package parts; both callers use it and the same workbook lookup.
- [ ] Refusal scan follows rels; tests cover a worksheet at a nonstandard target.
- [ ] `make check` green; reviewers approve.

## Out of scope
Changing canary contents or the Graph session flow.

## Outcome
New `tools/src/rustsheet_tools/oracle/package.py` is the one place that reads and rewrites an xlsx package: `read_parts`/`write_parts`, a rels-based `workbook_part`, `formula_parts` (the workbook plus every worksheet/macrosheet/dialogsheet its rels target) and `patch_workbook`. `force_full_calc_on_load` (no longer hardcodes `xl/workbook.xml`) and `embed_canary` both use it. The canary's SHEETS()/INFO() scan follows the rels, so a worksheet at a nonstandard path is covered, and `GET.WORKBOOK(` (XLM, defined names and macro sheets) is now refused too, as a conservative refusal rather than an Excel-semantics claim. Tests added for a nonstandard sheet path, GET.WORKBOOK and `force_full_calc_on_load`. The rels are parsed with ElementTree (a DTD is refused) and the scan fails closed: a rels entry pointing at a missing sheet part, or rels that do not parse, are errors, and everything under `worksheets/` is scanned too. The optional total-size cap was not added (T-0054); spec2xlsx's own zip loop is T-0053. No eval change.

## Handoff
Sonnet run; stopped after the security reviewer requested changes twice (work-loop 5b).
- Done on branch `agent/quality/T-0014-oracle-xlsx-rewrite-helper`: `oracle/package.py` (read_parts, write_parts, workbook_part, rels_part, formula_parts, patch_workbook) used by `force_full_calc_on_load` and `embed_canary`; rels parsed with ElementTree after a byte-level DTD refusal; fail-closed on a rels entry pointing at a missing sheet part; `worksheets/` directory also scanned; GET.WORKBOOK refusal; tests in tools/tests/test_oracle_canary.py (33+ pass, ruff/mypy/hygiene clean, full tools pytest passed before round 2). Quality and architecture reviewers APPROVE; follow-ups T-0053 (spec2xlsx zip loop) and T-0054 (size cap, duplicate names) filed.
- Open security findings (round 2, REQUEST_CHANGES):
  1. The DTD refusal is an ASCII byte search; a UTF-16 rels part (BOM) with a DOCTYPE/entity bypasses it. Fix: require strict UTF-8 or install a StartDoctypeDeclHandler that raises on the XMLParser.
  2. The refusal scan (SHEETS/INFO/CELL("width")/GET.WORKBOOK) still runs regexes over raw text decoded as utf-8 with "replace", as before this task; it misses UTF-16 parts, character references (`SHEETS&#40;`) and comment/CDATA splits. Suggested fix: parse each scanned part with ElementTree (DTD forbidden), run the patterns over the unescaped text of `<f>`, `<definedName>` and formula elements, fail closed on parse or encoding errors. This goes beyond the original task scope, so decide whether to do it here or file it.
  3. Minor: reject duplicate zip entry names (now also in T-0054).
- Next: apply 1 and 2 (or scope 2 into its own task), `make check`, re-run the security reviewer, then merge.
