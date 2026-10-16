---
id: T-1710
title: "Probe INDEX over a LET picking an external reference and LET bodies mixing external and other arms"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for INDEX, OFFSET and the reference readers over a LET that picks a reference into another workbook in the cases T-1535 left unprobed, with ours fixed if it differs.

## Context
T-1535 made a LET whose body picks only references into another workbook stand for the reference it picks (`Model::pick_reference`, external_reference.rs; probes/external-ref-no-link-let). Unprobed, raised by its excel-fidelity review: INDEX over such a LET (we read it as INDEX over the IF), OFFSET with such a LET as its reference, and a LET body that may also pick a non-external argument (`ROWS(LET(c,TRUE,IF(c,…!A1:B3,"x")))`, `ROWS(LET(c,TRUE,IF(c,…!A1:B3,D1:D2)))`), which `picks_only_external` leaves on the old path (likely `#BLOCKED!` where Excel may give 3). docs/excel-semantics/references.md "External reference with no cached data is `#BLOCKED!`".

## Acceptance criteria
- [ ] A probe (tag M8) with oracle values for `ERROR.TYPE(INDEX(LET(c,TRUE,IF(c,…!A1:C4)),2,2))`, `ROWS(INDEX(LET(c,TRUE,IF(c,…!A1:C4)),2,0))`, `ISREF(OFFSET(LET(c,TRUE,…!A1),1,1))`, `ROWS(LET(c,TRUE,IF(c,…!A1:B3,"x")))`, `ROWS(LET(c,TRUE,IF(c,…!A1:B3,D1:D2)))`, `ISREF(LET(c,TRUE,IF(c,…!A1,D1)))`, and `COUNTA(LET(c,TRUE,IF(c,…!A1:B2,1)))`.
- [ ] references.md updated; ours fixed if Excel differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
