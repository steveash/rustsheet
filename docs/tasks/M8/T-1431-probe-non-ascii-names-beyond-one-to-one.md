---
id: T-1431
title: "Probe non-ASCII names beyond one-to-one Latin pairs: CJK and Turkish I in LET/LAMBDA, and defined names starting with a non-ASCII letter"
milestone: M8
lane: functions-logic
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Get oracle evidence for names outside the one-to-one Latin letter pairs that T-1387 probed, and
make the engine match it.

## Context
- T-1387 made `is_valid_a1_identifier` (crates/rustsheet-engine/src/expressions/utils/mod.rs)
  accept a name whose first character is any Unicode letter (`char::is_alphabetic`).
  probes/let-name-non-ascii and probes/let-name-sharp-s cover é/É, Ä/ä, ñ/Ñ and ß/ẞ (and show
  that ß is not SS).
- Still unprobed:
  - CJK names: `LET(名前,1,名前+1)`.
  - Turkish İ/ı, where folding changes length: `LET(İ,1,i+1)` and `LET(ı,1,I+1)`.
  - Letter numbers and circled letters that `is_alphabetic` accepts: `LET(Ⅻ,1,Ⅻ+1)` and
    `LET(Ⓐ,1,ⓐ+1)`.
  - A defined name that starts with a non-ASCII letter (`é1`, `名前`). It goes through the same
    `is_valid_identifier` check.
- spec2xlsx writes such names with the `_xlpm.` prefix (its `_IDENT` pattern accepts any letter).
  Write one probe case. If Excel refuses a form, send it as a single-cell workbook with
  `rustsheet-oracle recalc -b excel-graph` and record the refusal, as T-1387 did. The
  `same_variable` folding is in functions/logical/let.rs.
- spec2xlsx decides which body names are bound with Python `str.lower()`. For a form where Python's
  fold and Excel's may differ (Turkish İ/ı, circled letters), write the uses as `_xlpm.<name>`
  in the case, or the probe tests the tool and not Excel. probes/let-name-sharp-s A2 shows how:
  `LET(ß,1,_xlpm.SS+1)`.

## Acceptance criteria
- [ ] A probe case (tag M8 for the defined names, M7 for LET) with excel-graph values for the forms
      above, or the refusals recorded. The engine matches.
- [ ] docs/excel-semantics/references.md is updated for the T-1387 paragraph.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Probes added: probes/let-name-non-ascii-scripts (M7: CJK, dotless ı, letter number Ⅻ) and probes/defined-name-non-ascii (M8: `é1`, `名前`), excel-graph 2026-10-18. The engine already matched every committed cell (+8 probe cells in eval/baseline/probes.txt, 0 regressions). Excel refuses a LET naming Turkish `İ` or circled `Ⓐ` (not committable); the engine accepts them, a known gap recorded in references.md (no follow-up: names that Excel refuses at load cannot occur in a real workbook).
