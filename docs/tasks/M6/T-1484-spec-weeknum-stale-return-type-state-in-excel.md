---
id: T-1484
title: "Spec: WEEKNUM stale return_type state in Excel's calc order (decision 0038 reopened)"
milestone: M6
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A spec (docs/specs/) for how RustSheet reproduces WEEKNUM's stale return_type state, which
follows Excel's calc order, and the tasks to implement it.

## Context
Decision 0052 reopened decision 0038: the stale state is a known gap, not a deviation. In Excel,
`WEEKNUM(serial, return_type)` with a return_type of 22 or more, or negative, reuses the start
weekday of the last valid WEEKNUM evaluated earlier in the same recalc, and is `#NUM!` only when
none ran (docs/excel-semantics/dates.md "WEEKNUM return_type edge cases and serial 0").
probes/weeknum-stale-return-type Up!A1, Mixed!A3, Mixed!A4, Type21!A1 and Type1!A1 are known
failures; `fn_weeknum` always gives `#NUM!`. On a fresh load the probes fit "bottom to top within
a sheet"; after edits Excel follows its calc chain (`calcChain.xml`). Decision 0038's options (a)
and (b) list the problems: RustSheet's storage-order passes, restarts on spill reordering, and a
hidden dependency that incremental recalc cannot see.

## Acceptance criteria
- [x] Probe what decides the order across sheets and after edits (excel-probe skill), and record it
      in docs/excel-semantics/dates.md.
- [x] docs/specs/SPEC-NNNN-*.md states how the state is kept and which calc order it follows, and
      files implementation tasks with `make new-task`.
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing calc-chain import in general, beyond what the spec needs.

## Outcome
- Probed the order and the state (excel-graph, 2026-10-19): 20 new probes/weeknum-stale-* cases.
  The state is Sunday, Monday or empty, one per recalc. Types 1 and 17 set Sunday; 2, 11, 21 and
  ISOWEEKNUM set Monday; types 12-16, the always-`#NUM!` types and array-lifted calls empty it; 22+
  and negative types read it. Fresh-load order is reverse load order: last sheet first, bottom row
  first, right to left, a cell reading an uncalculated formula is moved to just after it and re-run. A moved sheet keeps its load position.
  This corrects the old notes: ISOWEEKNUM does set the state, and there is no per-sheet reset.
- calcChain.xml is ignored on a full-calc load. Session edits isolate session-entered formulas.
  Both are in docs/research/t1484-weeknum-calc-order-2026-10-19.md.
- Recorded in docs/excel-semantics/dates.md ("WEEKNUM's stale return_type state") and recalc.md
  ("The order Excel calculates a freshly loaded workbook in").
- SPEC-0015 specifies a per-pass state, with an Excel-order pass that runs only when a WEEKNUM
  reader ran, so other workbooks keep today's order and cost. Follow-ups: T-2068 (engine, opus, M),
  T-2069 (engine, sonnet, S, moved sheets) and T-2070 (research, sonnet, S, open questions).
- Eval: probes known failures +41 (1914 -> 1955), all in the new M6 cases; no existing cell changed. With the 5 already listed, T-2068 and T-2069 own 46, owned by T-2068 and T-2069. No
  existing cell changed.
