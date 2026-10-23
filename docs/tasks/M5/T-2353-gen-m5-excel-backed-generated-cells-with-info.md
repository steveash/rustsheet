---
id: T-2353
title: "gen-m5: Excel-backed generated cells with INFO and SHEETS as the outer call (M5 catalog coverage 135/135)"
milestone: M5
lane: research
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: [T-0063]
---

## Goal
gen-m5 gets a small, Excel-backed batch of generated formulas whose outer call is INFO or SHEETS. M5's
"Catalog coverage per milestone" row then reads 135/135 and that exit measure is met.

## Context
- `make eval` (steward run 2026-10-22): M5 coverage is 133/135 not met, with "M5 missing (2): INFO (b); SHEETS (b)".
  Both have the evidence floor (INFO 25 Excel cells, SHEETS 28, from probes and ironcalc-calc). Neither has an
  Excel-backed scored cell in a generated suite tagged M5 (decision 0085 item 4;
  `crates/rustsheet-eval/src/catalog_coverage.rs` counts any scored gen-m5 cell whose formula calls the function).
- T-2251 (done) dropped `oracle: none` from both, so the exemption from (b) no longer applies. It also made the
  generator emit only their accepted forms: INFO with a type text other than "numfile" (`eval/catalog/functions.yaml`
  enum) and SHEETS with a reference argument. T-2251 left the generated cells to T-2252. T-2252 lists SHEETS among its
  64 functions but leaves INFO out as a decision 0094 exemption. Nothing owns INFO's generated cell, and T-2252 is a
  large 64-function sweep. This task adds the two coverage cells now, apart from that sweep. T-2252 still owns
  SHEETS's 400-cell evidence bar. The SHEETS cells added here count toward it.
- INFO on excel-graph is `#VALUE!` for every type text, and the engine matches (T-2190). SHEETS of a reference is
  verified (probes, ironcalc-calc). Expect few or no new failures.
- Commands:
  `cd tools && uv run rustsheet-gen --milestone M5 --focus --functions INFO,SHEETS --count 1 --formulas 60 --seed <new>`
  (`--functions` comes from T-0063). Then `uv run rustsheet-oracle fill -b excel-graph --suite gen-m5` and
  `make eval-baseline`.

## Acceptance criteria
- [ ] gen-m5 has one new case (or two) in which INFO and SHEETS are each the outer call of at least 20 formulas, all
      Excel-backed (excel-graph sidecar, canary-verified; the fill reports 0 failures).
- [ ] `make eval`: report.md's "Catalog coverage per milestone" M5 row shows covered 135/135, and "M5 missing" is
      gone.
- [ ] `make eval-baseline` shows 0 regressions. Any new gen-m5 known failure has an owner task (new or extended), named
      in the Outcome, and `make exit-owners MILESTONE=M5` reports no unowned cell from the new case. gen-m5's known
      failures stay within the exit allowance (floor(0.001 x scored)).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 400-cell evidence bar for SHEETS and the other T-2252 functions (T-2252).
- Generating `SHEETS()` with no argument or `INFO("numfile")`. The canary refuses them (T-2113).
- Desktop INFO values (T-2182, M14).
