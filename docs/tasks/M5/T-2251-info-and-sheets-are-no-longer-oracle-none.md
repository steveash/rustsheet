---
id: T-2251
title: "INFO and SHEETS are no longer oracle: none since T-2113: correct the catalog and generate their accepted forms"
milestone: M5
lane: harness
status: in-progress
priority: P1
size: S
tier: sonnet
campaign: visible
depends_on: []
---

## Goal
The catalog and the evidence report treat INFO and SHEETS as oracle-checkable, which they are
since T-2113. The report stops listing them as "need an evidence source other than the oracle".

## Context
- `eval/catalog/functions.yaml` still has `oracle: none` on INFO and SHEETS, and its header
  comment (around line 79) says the canary refuses every workbook that calls them.
- Since T-2113 (#1312) the canary (`tools/src/rustsheet_tools/oracle/canary.py`) refuses only
  `SHEETS()` with no argument and `INFO("numfile")`. T-2115 then filled oracle evidence: INFO has
  25 Excel cells and SHEETS 28.
- `crates/rustsheet-eval/src/catalog.rs` `OracleSupport` has one variant, `none` ("cannot return
  any value"). That no longer describes either function.
- The decision 0085 coverage exemption (b) for `oracle: none` therefore no longer applies, so M5's
  coverage then needs a generated cell for each.

## Acceptance criteria
- [ ] INFO and SHEETS lose `oracle: none`. Either `OracleSupport` gains a `partial` variant that
      names the refused forms (preferred, documented in the header comment), or the header
      comment explains the two refused forms and where their evidence comes from.
- [ ] The generator emits only the accepted forms of INFO (a literal type other than "numfile")
      and SHEETS (a reference argument). A tools test checks this.
- [ ] `make eval`: report.md no longer lists INFO and SHEETS as needing other evidence. M5
      catalog coverage stays 135/135 once T-2252's sweep adds their generated cells. If it
      drops in this PR, the Outcome says so and T-2252 restores it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- INFO's wrong values (T-2190).

## Visible-parity stage
Bookkeeping for decision 0094: its acceptance criteria are the whole stage.
