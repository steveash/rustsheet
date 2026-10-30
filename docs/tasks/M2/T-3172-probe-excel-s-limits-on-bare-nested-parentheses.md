---
id: T-3172
title: "Probe Excel's limits on bare nested parentheses past 255 and on unary sign runs, stored and typed"
milestone: M2
lane: research
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
docs/excel-semantics/syntax.md "Formula nesting depth cap" states, from excel-graph evidence, how
many bare nested parentheses and how long a run of unary signs Excel for the web accepts in a
stored and in a typed formula (the largest count accepted and the smallest refused, or "accepted up
to the 8,192-character formula limit"), so the engine's depth DEVIATION names exactly which inputs
it refuses that Excel accepts.

## Context
- T-3097 (probes/t3097-nesting-*, excel-graph 2026-10-29) showed Excel caps function calls at 65,
  and that "parentheses add no level": `=((...(1)...))` loads and evaluates at 64, 65, 127, 128 and
  255 levels. Nothing past 255 was probed. A formula's stored text may hold 8,192 characters
  (T-1060, T-1091), so `(` x n, `1`, `)` x n fits up to n = 4,095. Whether Excel accepts all of
  them, or stops at some other bound (255 and 256 are plausible, given the engine's own
  `Node::Group { levels: u8 }`), is unknown.
- Unary sign runs: RustSheet refuses a run of more than about 250 signs (`=----...1`,
  `=++++..."a"`; syntax.md, T-0132, T-2723) under `MAX_FORMULA_DEPTH`. No Excel cell covers a
  sign run longer than three (`probes/cases/array-constant-signs`). Up to 8,191 signs fit the
  text limit.
- Why opus: the limit must be bisected adaptively with the oracle, and inputs Excel refuses
  (createSession 501 `FileCorruptTryRepair` on load, Graph PATCH 400 `InvalidArgument` when typed)
  can never keep a sidecar, so, as T-1091 and T-3097 did, the refused counts are recorded in the
  semantics doc with their run date and dropped. The findings decide whether the DEVIATION stays
  as written, and T-3165 (engine, the eight t3097-nesting-paren cases) picks between lifting the
  bound and pinning deviations on this evidence if it lands first.
- Templates: `eval/suites/probes/cases/t3097-nesting-paren-255-{stored,typed}.case.json`
  (ISFORMULA beside the probe cell tells a formula from text in the typed form).

## Acceptance criteria
- [ ] Committed probe cases with excel-graph sidecars, stored and typed, for bare parentheses at
      256, 511, 512 and 4,095 levels (or the largest accepted count found by bisection and its
      neighbours), and for runs of `-`, of `+` before a text literal, and of alternating `-+`
      before a reference, at 255, 256, 1,024 and the longest run the text limit allows (or the
      bisected boundary).
- [ ] syntax.md "Formula nesting depth cap" records, per form and per entry path, the largest count
      Excel accepts and, if any, the smallest it refuses (refused inputs listed with the run date
      and the error, as T-1091's are), and its DEVIATION paragraph lists which accepted counts the
      engine refuses.
- [ ] Every new failing cell is either pinned as a decision-0036 deviation in
      `eval/suites/probes/deviations` against that DEVIATION entry, or owned by an open task (file
      an engine follow-up if T-3165's scope does not cover it); `make exit-owners MILESTONE=M2`
      reports 0.
- [ ] `make eval-baseline` refuses no regression; `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_FORMULA_DEPTH`, the Group node or the parser (T-3165 or a follow-up).
- Function-call nesting (T-3097, done) and what counts toward the 65-call cap (T-3143).
- Desktop Excel (M14).
