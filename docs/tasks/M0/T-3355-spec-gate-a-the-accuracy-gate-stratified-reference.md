---
id: T-3355
title: "SPEC: Gate A, the accuracy gate: stratified reference tables with 50-digit math and Excel values, checked by make check"
milestone: M0
lane: harness
status: in-progress
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
A spec, `docs/specs/SPEC-0029-gate-a-accuracy-gate.md` (take the next free SPEC number; this path is the expected one), designs Gate A as decision
0188 R2 defines it, and splits its implementation into the slices T-3356, T-3357 and T-3358 (amending them, or filing
more, if the design needs it).

## Context
- Decision 0188 R1(a), R2 and R3 (owner's 2026-10-31 assessment): accuracy is Phase 1's first gate. Of the 248,892
  numeric cells that differ from Excel, 99.5% agree to at least 12 digits, while real bugs (IRR and RATE `#NUM!`,
  T.INV.2T's cancellation, CHISQ.INV at a huge df, F.DIST at a tiny x) are not caught by any gate. The fit lab scores
  fits only on the cells it tunes them on, and nothing checks the engine against the math.
- Gate A: every numeric function in scope has a committed, stratified reference table (about 40% typical, 30% tails
  and domain edges, 20% extreme-but-valid, 10% log-uniform over the domain). Each point carries a 50-digit mpmath
  value and, where filled, Excel's value. `make check` fails on a point that is neither within ε = 1e-9 relative of the
  math (absolute floor 1e-300 near 0) nor within 15 significant digits of Excel, and where one side is an error and the
  other a number. R3 points are checked against the math only.
- Existing pieces to reuse (directive 3): the fit lab (docs/architecture/fit-lab.md, tools/src/rustsheet_tools/fitlab),
  the oracle client and fill (`rustsheet-oracle`), the visibly-off rule (`Visible::is_off`, crates/rustsheet-eval),
  the catalog (eval/catalog/) for the in-scope numeric functions, and `rustsheet-eval snapshot` (tools never parse
  xlsx values themselves).
- This is architectural (a new gate in `make check`, new committed data, a new tool), so it is a spec first
  (docs/process/agent-workflow.md, "Specs before architecture"). The architecture reviewer reviews it.

## Acceptance criteria
- [ ] The spec (docs/specs/TEMPLATE.md) decides:
  - the reference-table format and location (one file per function, a committed directory outside eval/baseline/ and
    quality/), the fields of a point (formula or arguments, 50-digit math value as text, Excel value and its fill
    metadata, stratum, an R3 mark naming its `MATH-OVER-EXCEL` entry);
  - the mpmath generator in tools/ (mpmath as a uv dependency, dps ≥ 60 internally, deterministic seeds) and the
    stratified sampler, per function, from the catalog signature's domains: how typical, tail, edge, extreme and
    log-uniform points are drawn, and how many (a floor per function, and the total CI cost);
  - how points are filled by the oracle (excel-graph) and refilled, and how a pending point is checked (math only);
  - the check in `make check`: which binary runs it, its runtime budget, its output (failing points with engine,
    Excel and math values), and how it fits the eval gate without hand-edited baselines;
  - how R3 points are marked and how a new R3 case is added (a decision record, decision 0188 R3);
  - how a math reference is attached to an eval cell so the R1(c) probe rows and Gate B can read it (T-3359),
    including the existing probes-numeric sweep cells in bulk (for example an mpmath fill per sweep case keyed by its
    formulas), so the R5-deferred sweep cells stop counting against the exit once they are within ε of the math;
  - how the tables join the R4 guards: a fresh log-uniform sweep (R4.2) is a table stratum, not a second format.
- [ ] Implementation slices are filed or amended (T-3356 format and gate, T-3357 generator and fill, T-3358 the first
      tables), each size S or M, with the spec linked.
- [ ] `make check` green; architecture reviewer approves.

## Out of scope
- Implementing the gate (T-3356 to T-3358).
- Changing the CI bit-regression gate or any baseline.
