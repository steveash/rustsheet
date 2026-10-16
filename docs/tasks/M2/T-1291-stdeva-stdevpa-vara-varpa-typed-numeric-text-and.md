---
id: T-1291
title: "STDEVA/STDEVPA/VARA/VARPA typed numeric text and MODE typed text and boolean arguments match Excel"
milestone: M2
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
STDEVA, STDEVPA, VARA and VARPA count a typed numeric text argument as its number, and MODE, MODE.SNGL and MODE.MULT treat typed text and boolean arguments as Excel does.

## Context
probes/stat-argument-modes (excel-graph, 2026-10-09, T-1080; legend in docs/excel-semantics/statistics.md "Statistical function argument modes"): `STDEVA(1,"2",4,TRUE)` is 1.4142135623730951 (as STDEV of 1,2,4,1; ours 2.1213203435596424), STDEVPA 1.224744871391589
(ours 1.5), VARA 2 (ours 4.5), VARPA 1.5 (ours 2.25): Excel counts the typed "2" as 2. `MODE(1,"2",4,TRUE)`,
MODE.SNGL and MODE.MULT are `#N/A` in Excel (ours `#VALUE!`), although STDEV/VAR coerce the same typed TRUE to 1: probe
whether MODE skips typed text and booleans (e.g. `MODE(1,"1",TRUE,1)`) before implementing.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/288b99797f58f149 Calc!A7 `=((Data!$C2/STDEVPA("Hello World",Data!C4:C9,Data!C5:C5))>..)`: Excel `#VALUE!` (typed non-numeric text), engine TRUE (STDEVPA 0.8048 ignores the typed "Hello World").

## Acceptance criteria
- [ ] Cells D37-D39 of probes/vara-single-ref-picked pass (direct `"x"`, `"2"`, TRUE in VARA); array literals (D40, D41) must stay numbers-only.
- [ ] The H24, H27, H30, H69, H72, H75 and H78 cells of probes/stat-argument-modes pass.
- [ ] Any new rule is recorded in docs/excel-semantics/statistics.md with probe evidence.
- [ ] gen-m9/288b99797f58f149 A7 passes (typed non-numeric text in STDEVPA is `#VALUE!`), with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
STDEVA, STDEVPA, VARA and VARPA read direct arguments like the other A-variants (`DirectNumbers::CastA`: a typed "2" is 2, TRUE is 1, text that is no number is `#VALUE!`); `NumbersOnly` is gone. MODE, MODE.SNGL and MODE.MULT skip typed text and booleans (new probe probes/mode-typed-arguments, excel-graph), which supersedes the old "text is `#VALUE!`" rule in coercion.md (the POI cells came from a blank reference or error argument). Eval: probes 10 fixed, mode-typed-arguments 29 cells passing (stat-argument-modes H24, H27, H30, H69, H72, H75, H78; vara-single-ref-picked D37-D39), gen-m9/288b99797f58f149 A7 passes, 1 new passing case, 0 regressions. MODE also propagates a direct error and gives `#VALUE!` for an empty argument (probe A22, A27, A30-A33).

## Handoff (resumed)
The implementation, probe (probes/mode-typed-arguments, 33 cells, excel-graph), baseline, docs, decision-0010 ratchet lines and `make check` (green on the previous head) are done and pushed on this branch; no PR is open yet. Reviewers: security, architecture and quality (round 1) approved or only had doc findings; the quality reviewer requested changes twice, both times for stale documentation (coercion.md MODE entry; SPEC-0009 and T-1617 still naming the removed `DirectNumbers::NumbersOnly`), all fixed in the head commit. The Excel-fidelity reviewer's verdict had not arrived when this run ran out of its time budget. Next: run `make check`, re-run the quality and Excel-fidelity reviewers on the head, open the PR (title `T-1291: ...`, eval delta: probes 10 fixed, new probe passing, 0 regressions, gen-m9/288b99797f58f149 A7 passes), set `status: done` and the tier back, merge per the steward policy.

### Open blocker from the Excel-fidelity reviewer (unverified by this run; check it before merging)
The new MODE rules (skip typed text, propagate every direct error) may regress three Excel-cached POI cells that pass today and are absent from eval/baseline/poi.txt: EverythingTests G936 `MODE(C9,E16,"1",B8)`, H936 `MODE(B8,"1",2,3)` and K936 `MODE("1",1,#DIV/0!,2,3)` are `#VALUE!` in Excel 15; by the reviewer's trace the branch now gives `#N/A`, `#N/A` and `#DIV/0!` (fixture data in docs/tasks/M9/T-0947-*.md: B8=1, C9=1.1, E16="1.1" as text). The poi suite is not fetched locally or in CI, so `make eval` cannot show it (fetch it with `rustsheet-eval fetch --suite poi` and run it). Next: probe the exact shapes on excel-graph (`MODE(1,"1",2,3)`, `MODE("1",1,#DIV/0!,2,3)`, `MODE(1.1,<ref to text "1.1">,"1",1)`, `MODE("1",2,3)`, more distinct numbers plus typed text); the probe's A21/A22 used a duplicate or a different argument order, so they do not settle it. If M365 differs from Excel 15, document that in docs/excel-semantics and rebaseline poi (may need a decision record); otherwise find the real rule. Then rewrite the coercion.md MODE entry (it still says the POI cells come from blank/empty/error arguments, which the reviewer says is false for H936 and G936). Minor: a direct LAMBDA is now `#VALUE!` for VARA/VARPA/STDEVA/STDEVPA (CastA) with no probe for the variance family; add a probe cell or a note in statistics.md.
