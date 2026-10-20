---
id: T-2115
title: "Probe SHEETS with reference arguments and INFO type_text values on excel-graph (oracle: none evidence gap)"
milestone: M5
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: [T-2113]
---

## Goal
In-scope M5 functions SHEETS and INFO each have at least 20 Excel-valued cells in eval, enough for
the evidence floor that M5's catalog coverage needs. Each Excel rule found is recorded in
docs/excel-semantics/information.md. Each engine mismatch is fixed, if it is a one-line change in
the same function, or filed as a functions-logic task with milestone M5.

## Context
- report.md "Excel evidence per function" (2026-10-20): "Need an evidence source other than the oracle
  (`oracle: none`) (2): INFO, SHEETS". Their Excel cell counts are 0 and 1. The canary refused every
  workbook that used them; T-2113 narrows the refusal to bare `SHEETS()` and `INFO("numfile")`.
- One probe case, `eval/suites/probes/cases/info-sheets-info-types.case.json` (excel-probe skill),
  with milestone tag M5 (decisions 0051, 0057). Sheets S1, S2, S3 and Data, in that order.
  - SHEETS: `SHEETS(A1)`, `SHEETS(A1:B2)`, `SHEETS(S2!A1)`, `SHEETS(S1:S3!A1)`, `SHEETS(S2:S3!A1:B2)`,
    `SHEETS(S1:Data!A1)`, `SHEETS((A1,B1))` (union), `SHEETS(INDIRECT("S1:S3!A1"))`,
    `SHEETS(OFFSET(A1,1,1))`, a defined name for a 3-D span, `SHEETS(#N/A)`, `SHEETS("S1")`,
    `SHEETS(1)`, `SHEETS(TRUE)`, `SHEETS({1,2})`, `ISNUMBER(SHEETS(A1))`, `SHEETS(S1:S3!A1)+0`.
  - INFO: each documented type_text, "directory", "origin", "osversion", "recalc", "release" and
    "system", in lower and upper case. The legacy "memavail", "memused" and "totmem", `INFO("")`,
    `INFO("x")`, `INFO(1)`, `INFO(TRUE)`, `INFO(#N/A)`, `INFO({"recalc","system"})`, `TYPE(INFO("release"))`
    and `ISTEXT(INFO("directory"))`.
  - Host-dependent values ("directory", "osversion", "release" and "origin" can depend on the oracle's
    server or session) are scored through shape companions only: ISTEXT, LEN>0, or ISERROR/ERROR.TYPE.
    Record the raw value in the excel-semantics entry, and do not make it an exact expectation that
    a different host would break. Say in the entry which values looked host-dependent.
- Fill with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`. If excel-graph
  refuses a formula, drop that cell and note it.
- Bare `SHEETS()` and `INFO("numfile")` stay without web evidence. If their behaviour needs desktop
  Excel, file or extend an M14 item (decision 0039). Never block M5 on them.
- Engine code: the information functions in crates/rustsheet-engine/src/functions/information/
  (`rg -n "fn_sheets|fn_info"`).

## Acceptance criteria
- [x] `probes/info-sheets-info-types` is committed with its excel-graph oracle sidecar, and the report's
      evidence table shows SHEETS and INFO with ≥ 20 Excel cells each.
- [x] docs/excel-semantics/information.md has a VERIFIED (excel-graph) entry for SHEETS and one for INFO,
      citing the probe cells, with the host-dependent values marked.
- [x] Every failing cell of the new case is fixed in this PR or listed in a filed task (milestone M5,
      lane functions-logic), and the baseline records it (`make eval-baseline`).
- [x] `make check` green; reviewers approve.

## Out of scope
- Removing `oracle: none` from the catalog. T-2116 decides how such functions count, and the catalog
  edit can follow once the evidence is in.
- Bare `SHEETS()` and `INFO("numfile")` (canary-dependent).

## Handoff (resumed)
A sonnet run drafted the case and handed off the INFO/3-D question; resolved as the Outcome says.

## Outcome
probes/info-sheets-info-types (excel-graph, 2026-10-20) holds 24 SHEETS cells, 3 SUM/COUNT 3-D controls and 25 INFO
cells, all Excel-valued, so both functions clear the 20-cell floor.
- SHEETS counts the sheets a span covers, in either order (information.md "SHEETS counts the sheets a reference
  spans"). The handoff's "3-D SHEETS is `#VALUE!`" came from sheets named S1:S3, which Excel for the web misreads in
  an unquoted span. That is now its own committed case, probes/sheet-names-like-cells-3d-spans, recorded as a probe trap.
- INFO is `#VALUE!` for every type_text on the web. As with ENCODEURL and FILTERXML, these are the goldens
  (decision 0039). The engine follows them in T-2184 (M5), and T-2182 (M14, blocked) asks desktop Excel.
- The engine's `#N/IMPL!` for `SHEETS(ref)` is T-2181 (M5).
- Eval: the two new cases add only new-coverage failures, owned by T-2181 (24 SHEETS cells + 3 in
  the trap case) and T-2184 (INFO cells); no regressions.
