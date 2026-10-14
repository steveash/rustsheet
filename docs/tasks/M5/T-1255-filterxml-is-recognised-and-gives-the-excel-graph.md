---
id: T-1255
title: "FILTERXML is recognised and gives the excel-graph golden #VALUE! (5 gen-m6 cells)"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
FILTERXML is a known function: its arguments are evaluated (an argument error propagates, left to right) and otherwise it returns `#VALUE!`, the excel-graph golden, instead of `#NAME?`.

## Context
docs/excel-semantics/web.md "FILTERXML is `#VALUE!` in Excel for the web": `FILTERXML("<a>1</a>","//a")` is `#VALUE!`
on excel-graph (probes/catalog/function-name-forms A7). Decision 0039 makes the web value the golden for Phases 1-3; the
desktop question is T-1081 (M14). The engine does not know the name, so every formula using it is `#NAME?`.
T-1239's gen-m6 suite (excel-graph sidecars, 2026-10-09) has five such cells:
- gen-m6/3ee7ffc8d6ab061f Calc!A11 `=((Data!C10-FILTERXML("TRUE","?b"))>(..))`: Excel `#VALUE!`, engine `#NAME?`.
- gen-m6/42b001dc2803fe24 Calc!A19 `=FILTERXML("",(VLOOKUP("1",Data!E3:E9,1e-15,TRUE)&ASC("ü")))`: Excel `#VALUE!`.
- gen-m6/abcfd9979f316267 Calc!A9 `=FILTERXML((Data!D$4&NA()),ADDRESS(2,10,3,FALSE))`: Excel `#N/A` (the argument error wins).
- gen-m6/f94b5b8236742618 Calc!A13 `=(((Data!A9*0)+FILTERXML(Data!E9,Data!$B8))/ATAN2(..))`: Excel `#VALUE!`.
- gen-m6/f94b5b8236742618 Calc!A21 `=HLOOKUP(Data!$B4,Data!C6:E10,1,(FILTERXML(Data!D1,Data!A2)<>ROUNDDOWN((-1),-6)))`: Excel `#VALUE!`.
Follow ENCODEURL's handling (T-1219, functions/text/encodeurl.rs) for where the function lives; the catalog entry
(eval/catalog/functions.yaml) already lists FILTERXML in scope. Whether FILTERXML lifts over an array argument is unprobed:
do not lift; give `#VALUE!` for the whole call. Cite decision 0039 and web.md in a code comment.

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/4629e931955aa4b8 Calc!A34 `=FILTERXML((Data!A7&DROP(Data!B8,1.005,Data!$B$12)),Data!D11)`: Excel `#CALC!` (DROP's error propagates first), engine `#NAME?`.
- gen-m7/7c48e2456500644f Calc!A35 `=IF(FILTERXML(EXPAND(Data!$A$1,1,2,Data!B8),INDEX(Data!C8:C10,2,Data!B4)),..)`: Excel `#VALUE!`, engine `#NAME?`.
- gen-m7/c5d278a8662ef4b2 Calc!A2 `=FILTERXML("1.5",((Data!D3&"ABC")&LEFT(" x ",2)))`: Excel `#VALUE!`, engine `#NAME?`.

More cells (T-1289's gen-m9 suite, excel-graph, 2026-10-12; `./target/release/rustsheet-eval show gen-m9/<case>.case.json`):
- gen-m9/b58b6d7cd39d195f Calc!A8 `=FILTERXML(Data!D$7,"1")`: Excel `#VALUE!`, engine `#NAME?`.
- gen-m9/d5bd4272655c5211 Calc!A37 `=(Data!A5+UNIQUE(FILTERXML(Data!C9,Data!A12)))`: Excel `#VALUE!`, engine `#NAME?`.

## Acceptance criteria
- [ ] gen-m6/3ee7ffc8d6ab061f A11, gen-m6/42b001dc2803fe24 A19, gen-m6/abcfd9979f316267 A9 and gen-m6/f94b5b8236742618 A13, A21
      pass in `./target/release/rustsheet-eval show <case>`, with no new failing gen-m6 cell and no new known failures in eval/baseline/.
- [ ] probes/catalog/function-name-forms A7 still passes (or now passes); web.md's entry gets an `Implemented in:` line.
- [ ] gen-m7/4629e931955aa4b8 A34, gen-m7/7c48e2456500644f A35 and gen-m7/c5d278a8662ef4b2 A2 pass. (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [ ] gen-m9/b58b6d7cd39d195f A8 and d5bd4272655c5211 A37 pass, with no new failing gen-m9 cell and no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Real XML/XPath evaluation (only if T-1081 shows desktop Excel computes it).

## Outcome
FILTERXML is `Function::Filterxml` (functions/text/filterxml.rs): arguments evaluated left to right, first error wins, otherwise `#VALUE!` (decision 0039, web.md). `language.bin` regenerated (debug build of tools/upstream/generate_language; the release build hung). `functions/mod.rs` stays at its 2182-line ratchet by merging four compat-alias dispatch arms. Eval: probes known failures 5615 -> 5613 (2 fixed, 0 regressions); all nine listed gen-m6/m7/m9 cells pass.
