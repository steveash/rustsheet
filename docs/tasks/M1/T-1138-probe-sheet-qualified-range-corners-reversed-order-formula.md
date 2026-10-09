---
id: T-1138
title: "Probe sheet-qualified range corners: reversed order, formula text and corners on two sheets"
milestone: M1
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence settles how Excel evaluates and prints a range whose two cell corners both carry a sheet prefix when the corners are reversed, and what it does with corners on two different sheets, turning docs/excel-semantics/references.md "Sheet-qualified range corners: order and unprobed forms" from UNVERIFIED into evidenced rules.

## Context
M1 scope "References: ranges, cross-sheet, quoted sheet names". references.md "Sheet-qualified range corners" is Status UNVERIFIED: "reversed corners, round-tripping and the cross-sheet `#VALUE!` are engine behavior without a probe; T-0139 verified only the second-prefix column/row forms". The engine orders corners in `get_range` (crates/rustsheet-engine/src/reference_operators.rs) and gives `#VALUE!` for `Data!A1:Other!B2` (T-0122 Outcome). Existing probes: probes/sheet-qualified-both-corners (column/row corners, T-0139) and probes/sheet-qualified-ref-error-forms. Some forms may make Excel for the web refuse the workbook (`FileCorruptTryRepair`, see the T-0139 entry in references.md): isolate each such form as a one-formula scratch case, record the refusal in references.md and leave it out of the committed case, as T-0139 did.

Follow the `excel-probe` skill. Use the Data/Calc layout of probes/sheet-qualified-both-corners (Data A1:B2 numbers, a sheet Other with different numbers). New case probes/sheet-qualified-corner-order with at least:
- `=SUM(Data!B2:Data!A1)`, `=SUM(Data!A2:Data!B1)`, `=ROWS(Data!B2:Data!A1)`, `=SUM('Data'!$B$2:Data!A1)`, and FORMULATEXT of each SUM (does Excel keep the written order and both prefixes?).
- `=SUM(Data!A1:Other!B2)`, `=ISREF(Data!A1:Other!B2)`, `=Data!A1:Other!A1`, `=IFERROR(SUM(Data!A1:Other!B2),"e")`.
- Control: `=SUM(Data!A1:Data!B2)` and its FORMULATEXT.

## Acceptance criteria
- [ ] probes/sheet-qualified-corner-order is committed with an excel-graph oracle sidecar for every form that loads; each refused form is named in references.md with the refusal message and date.
- [ ] references.md "Sheet-qualified range corners" states the evidenced rule for reversed corners, formula text and two-sheet corners, cites the case cells, and its Status is VERIFIED (web) for every probed part.
- [ ] Engine mismatches are recorded with `make eval-baseline` as new-coverage failures only, and a follow-up engine task owning them is filed and named in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `get_range`, the lexer or the printer (file the follow-up instead).
- Column/row corner forms (already settled by T-0139) and 3-D spans (T-1139).

## Outcome
Added probes/sheet-qualified-corner-order with an excel-graph sidecar (13 cells); every form loaded, none was refused. Reversed corners are ordered, FORMULATEXT keeps the written order and both prefixes, cross-sheet corners give `#VALUE!` and ISREF FALSE. references.md updated to VERIFIED (web). Eval delta: probes known failures 166 -> 167 (new coverage: Calc!A7 ISREF, engine returns TRUE); follow-up T-1166 owns it.
