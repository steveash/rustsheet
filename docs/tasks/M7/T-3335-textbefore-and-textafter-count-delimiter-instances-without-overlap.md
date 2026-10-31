---
id: T-3335
title: "TEXTBEFORE and TEXTAFTER count delimiter instances without overlap: probe and match Excel (TEXTBEFORE(\"-2.5\",{\"2.5\",\"-2.5\"},2) is #N/A, ours \"-\")"
milestone: M7
lane: functions-text
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
TEXTBEFORE and TEXTAFTER count `instance_num` the way Excel does when delimiter matches overlap (one delimiter
inside another, or a delimiter that overlaps itself), so the gen-m9 cell below and a new probe pass.

## Context
Gap triage 2026-10-31 (`make eval` on origin/main, excel-graph; `scripts/exit_owners.py --all-off` listed it as
unattributed and no open task named it):
- gen-m9/cde551ea787f713a Calc!A37
  `=BETADIST(TEXTBEFORE((Data!C11&Data!E3),Data!B11:C11,2),3,Data!B$11,(NPV(..)+..),RADIANS(INDEX(Data!D9:D11,5)))`:
  Excel `#N/A`, ours `#VALUE!`. With the case's data this is `TEXTBEFORE("-2.5",{2.5,-2.5},2)` (C11 = -2.5, E3 blank,
  B11:C11 = 2.5, -2.5, read as the texts "2.5" and "-2.5"). The other arguments are `#VALUE!` (NPV at the text rate
  " x ") and `#REF!` (INDEX row 5 of 3), so Excel's `#N/A` can only be TEXTBEFORE's not-found: Excel finds no second
  instance. Ours finds two, "-2.5" at unit 0 and the overlapping "2.5" at unit 1, returns "-", and BETADIST of "-" is
  `#VALUE!` (scratch run of the sub-expressions, 2026-10-31).
- Ours: `text_sensitive` and `text_sensitive_reverse` in crates/rustsheet-engine/src/functions/text/util.rs step one
  character after every match, so instances overlap (as SUBSTITUTE's do, which is VERIFIED for SUBSTITUTE only:
  docs/excel-semantics/text.md "SUBSTITUTE instances overlap"). No case in eval/ has a literal TEXTBEFORE or TEXTAFTER
  whose overlapping and non-overlapping instance counts differ (scan of every case, 2026-10-31), so the rule is
  unprobed beyond this one nested cell. TEXTSPLIT's split points (`textsplit.rs`) may follow the same scan.
- Semantics entry (inferred, this triage): docs/excel-semantics/text.md "TEXTBEFORE and TEXTAFTER: overlapping
  delimiter instances".

Cells:
- gen-m9/cde551ea787f713a.case.json (1): Calc!A37

Probe first (excel-probe skill, excel-graph), one probe case, e.g. probes/t3335-textbefore-overlapping-instances:
- the cell itself and its neighbours: `TEXTBEFORE("-2.5",{"2.5","-2.5"},k)` and TEXTAFTER for k = 1, 2, -1, -2, and the
  list in the other order;
- one delimiter overlapping itself: `TEXTBEFORE("aaa","aa",2)`, `TEXTBEFORE("aaaa","aa",2)`, `(...,3)`,
  `TEXTAFTER("aaaa","aa",2)`, `TEXTBEFORE("aaa","aa",-1)`, `(...,-2)`, `TEXTAFTER("aaaaa","aa",-2)`;
- a short delimiter inside a longer one: `TEXTAFTER("abcabc",{"abc","bc"},2)`, `TEXTBEFORE("xabcx",{"b","abc"},2)`;
- the same with match_mode 1 (`TEXTBEFORE("AAA","aa",2,1)`) and with match_end 1 (`TEXTBEFORE("aaa","aa",2,,1)`);
- whether the `#VALUE!` bound on instance_num (positions of the shortest delimiter, text.md "instance_num past the
  text") still holds at the overlap edge (`TEXTBEFORE("aaa","aa",3)` vs `TEXTBEFORE("aaaa","aa",3)`);
- TEXTSPLIT over the same texts (`TEXTSPLIT("aaa","aa")`, `TEXTSPLIT("-2.5",{"2.5","-2.5"})`) to confirm or rule out the
  same scan there.

## Acceptance criteria
- [ ] The probe is oracle-filled and committed; its findings replace the inferred entry in
      docs/excel-semantics/text.md (Status VERIFIED, cells cited).
- [ ] gen-m9/cde551ea787f713a Calc!A37 passes: this known-failure line disappears from eval/baseline/gen-m9.txt with no
      new known-failure lines in any baseline (`make eval-baseline`).
- [ ] Every probe cell passes, or a remaining one is named in a follow-up task.
- [ ] SUBSTITUTE's overlapping instances (probes/substitute-overlapping-instances) still pass: the change does not
      reuse `nth_overlapping_match` for TEXTBEFORE/TEXTAFTER.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cell this task owns is visibly off (wrong error), so the task is in the campaign at the
`semantics` stage. Its target is that every visibly-off cell it owns has Excel's type, error and text.

## Out of scope
- BETADIST, NPV and INDEX themselves: they already give the errors Excel's arguments would.
- SUBSTITUTE (VERIFIED overlapping, T-1644).
