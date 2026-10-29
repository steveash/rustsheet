---
id: T-3095
title: "Probe the near-zero root snap of a LAMBDA body called by MAP, BYROW, BYCOL, REDUCE, SCAN and MAKEARRAY"
milestone: M7
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: [T-3094]
---

## Goal
Excel evidence for whether each LAMBDA helper snaps its LAMBDA's near-zero `+`/`-` root per call, so that every
element (and every accumulator step) of a helper result holds Excel's 0 or Excel's residue.

## Context
- docs/excel-semantics/numbers.md "Near-zero snapping", "Pickers, LET, LAMBDA and unary signs": a LAMBDA body called in
  place is snapped at its own root (row 17 of probes/t3049-snap-under-pickers, T-3049); the helpers are listed there
  as unprobed. T-3094 probes the other single-call forms (name, LET, group, nested call) first, so this probe can
  reuse its reading of where a call's root is.
- Element-wise snapping is already known for root operators over arrays (T-3047, probes/t3047-snap-array-operands:
  `=1+{3;7;8}*2^-52-1` snaps element by element at the scalar bound). A helper differs because its LAMBDA is called
  once per element (MAP, BYROW, BYCOL, MAKEARRAY) or once per step with an accumulator that carries the previous result
  (REDUCE, SCAN). If Excel snaps each step of REDUCE, an accumulator that cancels to the residue is reset to 0 before
  the next step, which changes later steps visibly; SCAN shows every step.
- Engine: the helpers call `Model::call_lambda` / `call_lambda_on_slice` (functions/logical/lambda.rs,
  bycol_byrow.rs, map_reduce.rs, scan.rs); whether T-3083's snap reaches them depends on where it lands.
- Why opus: the probe must separate a per-call snap from a snap of the final spilled values (a reader of the spill,
  `INDEX(MAP(..),1)`, and a `SUM` over it), and the REDUCE/SCAN rows need accumulators whose later steps differ
  between "snapped each step" and "snapped at the end only".

## Acceptance criteria
- [x] A new excel-graph probe case `probes/t3095-snap-lambda-helpers` (milestone M7, the helpers' milestone, decision
      0051) reads back: `MAP` over {0.1;1;1} with the bodies `x+0.2-0.3` and `x+7*2^-52-1` / `x+8*2^-52-1`; `BYROW` and
      `BYCOL` over a 2x3 range whose rows/columns sum through `SUM(r)-..` and through `INDEX(r,1)+INDEX(r,2)-..`;
      `MAKEARRAY(2,2,LAMBDA(r,c,..))` with a cancelling body; `REDUCE(0,{0.1,0.2,-0.3,1},LAMBDA(a,x,a+x))` and the
      same `SCAN`, plus a k = 7 / k = 8 accumulator pair; each also through `INDEX(..,n)` and `SUM(..)` readers. The
      in-place call `=LAMBDA(x,<body>)(..)` is the control row.
- [x] numbers.md records, per helper, whether each call's root is snapped (and for REDUCE/SCAN whether each step or
      only the final value is), citing the cells; the "Unprobed" sentence drops what this case answers.
- [x] Every cell the engine does not match is a known failure in eval/baseline/probes.txt, and one follow-up task
      (milestone M7, lane engine, tier opus) names those cells; `make exit-owners MILESTONE=M7` reports 0.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes (the follow-up owns them).
- GROUPBY/PIVOTBY aggregation LAMBDAs and eta-reduced helpers (`MAP(r,SUM)`): file a probe task if this one finds
  per-call snapping.

## Outcome
- probes/t3095-snap-lambda-helpers (136 Calc cells plus 12 Data cells, excel-graph 2026-10-29). Every call that MAP,
  MAKEARRAY, BYROW and BYCOL make snaps the LAMBDA body's `+`/`-` root, element by element. REDUCE and SCAN snap at
  every accumulator step, so later steps start from 0. An in-place call passed as another LAMBDA call's argument
  snaps too. k = 8 keeps its residue everywhere.
- numbers.md "Near-zero snapping" records each helper in a new paragraph after "Pickers, LET, LAMBDA and unary signs" with its cells. Its "Unprobed" sentence now
  names only GROUPBY/PIVOTBY and the eta-reduced helpers (T-3112).
- Eval delta: +1 case, 83/148 cells pass. The 65 that fail are known failures in eval/baseline/probes.txt and are owned
  by T-3111 (M7, engine, opus).
- Follow-ups: T-3111 (engine snap for the helpers and for nested argument calls) and T-3112 (probe GROUPBY/PIVOTBY
  and the eta-reduced helpers, as this task's out-of-scope note asked once per-call snapping was found).
