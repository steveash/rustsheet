---
id: T-2115
title: "Probe SHEETS with reference arguments and INFO type_text values on excel-graph (oracle: none evidence gap)"
milestone: M5
lane: research
status: todo
priority: P2
size: S
tier: sonnet
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
- [ ] `probes/info-sheets-info-types` is committed with its excel-graph oracle sidecar, and the report's
      evidence table shows SHEETS and INFO with ≥ 20 Excel cells each.
- [ ] docs/excel-semantics/information.md has a VERIFIED (excel-graph) entry for SHEETS and one for INFO,
      citing the probe cells, with the host-dependent values marked.
- [ ] Every failing cell of the new case is fixed in this PR or listed in a filed task (milestone M5,
      lane functions-logic), and the baseline records it (`make eval-baseline`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Removing `oracle: none` from the catalog. T-2116 decides how such functions count, and the catalog
  edit can follow once the evidence is in.
- Bare `SHEETS()` and `INFO("numfile")` (canary-dependent).
