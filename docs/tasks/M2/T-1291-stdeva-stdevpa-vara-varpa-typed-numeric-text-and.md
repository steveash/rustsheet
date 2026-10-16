---
id: T-1291
title: "STDEVA/STDEVPA/VARA/VARPA typed numeric text and MODE typed text and boolean arguments match Excel"
milestone: M2
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
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
STDEVA, STDEVPA, VARA and VARPA read direct arguments like the other A-variants (`DirectNumbers::CastA`: a typed "2" is 2, TRUE is 1, text that is no number is `#VALUE!`); `NumbersOnly` is gone. MODE, MODE.SNGL and MODE.MULT skip typed text and booleans (new probe probes/mode-typed-arguments, excel-graph), which supersedes the old "text is `#VALUE!`" rule in coercion.md (the POI cells came from a blank reference or error argument). Eval: probes 10 fixed, mode-typed-arguments 29 cells passing (stat-argument-modes H24, H27, H30, H69, H72, H75, H78; vara-single-ref-picked D37-D39), gen-m9/288b99797f58f149 A7 passes, 1 new passing case, 0 regressions. MODE also propagates a direct error and gives `#VALUE!` for an empty argument (probe A22, A27).
