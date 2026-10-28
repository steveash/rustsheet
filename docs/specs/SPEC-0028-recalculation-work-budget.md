# SPEC-0028 — A work budget for one recalculation

- **Status:** draft (T-2807)
- **Milestone / lane:** M7 / engine
- **Tasks:** T-2807 (this spec); the rollout steps below are filed as T-3008 and T-3009

## Problem
SPEC-0024 bounds the work of one formula evaluation at `MAX_FORMULA_WORK` (2^28 work units, 6.2 to
7.7 s at its slowest measured kinds: arrays.md "One formula evaluation does at most 268,435,456 work
units"). Nothing bounds the work of one recalculation (`Model::evaluate`): it runs every formula at
least once, so its work is the number of formula runs times what each may spend. The SPEC-0024
non-goals record this as a residual risk. The multipliers, all in a workbook of a few megabytes:

| multiplier | where | how many |
|---|---|---|
| formula cells | `run_pass` (evaluation.rs): every cell of `all_positions` | 1,000 near-limit cells (2^28 units each) are about 2 hours |
| passes of one round | `evaluate_passes`: a restart abandons the pass and runs it again (restart_log.rs: at most n(n-1) facts and n marks for n anchors); an Excel-order pass after a pass that read the order state (SPEC-0017) | unbounded by a constant |
| runs cut short | `evaluate_formula_cell`: past `max_depth` the stack unwinds and its cells run again from the top | once per unwinding |
| iterative sweeps | iteration.rs `sweep`: at most `MAX_SWEEP_EVALUATIONS` (2^25) member runs | 2^25 runs of up to 2^28 units each |
| data tables | data_tables.rs: at most `MAX_DATA_TABLE_EVALUATIONS` (2^25) runs, then a second round of passes | 2^25 runs, then every formula again |

The per-call caps (`MAX_LIFTED_CELL_VISITS`, the area budgets) are drawn from the formula budget
(SPEC-0024 section 1), so they need no rule of their own here: bounding the formula runs bounds
them. The count caps of the sweeps and of the data tables bound how many runs there are, not what
they spend.

## Goals / non-goals
Goals:
1. One recalculation does a bounded amount of work, whatever the number of formulas, restarts,
   sweeps and data tables. The bound is in SPEC-0024's work units, summed over the recalculation,
   so it is deterministic and tests use counters (T-2282).
2. No eval workbook changes value: every recalculation in eval stays far below the bound (section 5).
3. Which cells give up past the bound depends only on the model (formulas, values, settings), not
   on the order in which the evaluator happens to visit the cells, wherever that is possible
   (section 3 says where it is not).
4. Past the bound the recalculation ends quickly: its remaining work is a fixed multiple of the
   bound at most.
5. One budget, extended from the one that exists (directive 3): the units are SPEC-0024's, charged
   at the same chokepoints, and the formula budget's lifetime (`Model::with_area_budget`) is where
   they are summed.

Non-goals:
- Conditional formats and `evaluate_formula`. They run outside a recalculation (on a style read,
  SPEC-0021; on a call), each with its own formula budget; a round of conditional formats over many
  cells is bounded by the conditional-format round's own caps (cf_styles.rs), not by this spec.
- Work that is linear in the size of the workbook and charges no unit: walking the cells of
  `all_positions` (constants cost nothing), restoring spills, `begin_iteration`'s copy of every
  formula cell. Its size is bounded by the input.
- Dense single-range walks (SPEC-0024 non-goals, T-2710, T-2711, T-2713).
- Wall-clock deadlines: non-deterministic (Alternatives).

## Excel behavior to match
Excel has no documented limit on the work of a recalculation. A long one runs until done (Excel for
Windows shows "Calculating (4 threads): n%" in the status bar and can be interrupted by a key; the
interrupted cells stay uncalculated). Excel for the web's session times out before giving a value
(arrays.md "How deep LAMBDA calls nest": createSession failed), so a workbook that reaches the bound
cannot be probed. Any bound is a **DEVIATION** (decision 0014): directive 5 wins over directive 1.
The bound must sit far above every eval workbook (Goal 2), which the gate enforces.

## Design

### 1. Two numbers: an allowance per pass and a ceiling per recalculation
```rust
/// Work units the formula runs of one pass may commit before the round is re-run with each
/// formula's share (SPEC-0028): four formula budgets.
pub(crate) const MAX_PASS_WORK: u64 = 1 << 30; // provisional, section 5

/// Work units one recalculation (`Model::evaluate`) may spend in all, committed or thrown away,
/// in every phase (SPEC-0028): the time bound of directive 5.
pub(crate) const MAX_RECALC_WORK: u64 = 1 << 32; // provisional, section 5
```

- The **ceiling** (`MAX_RECALC_WORK`) is the time bound. Every unit any formula run spends during
  `Model::evaluate` counts against it: every pass, the runs cut short, the sweeps, the data tables,
  the second round of passes. It is the only number that bounds the time; the allowance below is
  what makes the result deterministic.
- The **pass allowance** (`MAX_PASS_WORK`) decides whether a round of passes is computed normally.
  It counts two things separately, each against `MAX_PASS_WORK`:
  - the **committed** work of the current pass: the units of the formula runs whose values the
    pass keeps. It starts at 0 with each pass (`run_pass`), since each pass recomputes every
    formula;
  - the **thrown-away** work of the round (`evaluate_passes`): a pass abandoned by a restart or
    replaced by an Excel-order pass adds its committed work here, and a run cut short by the depth
    limit (`unwinding`) adds what it spent so far.

  Keeping them apart is what makes the decision independent of the order (section 3): the committed
  work of a complete pass is the same whatever the order, the thrown-away work is not.

The two numbers are each a power of two and a multiple of `MAX_FORMULA_WORK`: four formulas at the
formula limit fit in one pass, and the ceiling is four pass allowances.

### 2. What is summed, and where
The units are SPEC-0024's, charged where they are charged now; the hot path does not change
(`spend_work` stays one subtraction and a branch, SPEC-0024 section 2). The sums are made at the
boundaries of formula runs, in `Model::with_area_budget`, which every formula run goes through:

- A new `RecalcBudget` lives in `Evaluation` (evaluation.rs), reset by `Model::evaluate` and absent
  (`None`) outside it, so `evaluate_formula` and conditional formats draw on no pool:

  ```rust
  pub(crate) struct RecalcBudget {
      /// Units left of [`MAX_RECALC_WORK`]: everything this recalculation spent.
      left: u64,
      /// Units the current pass committed ([`MAX_PASS_WORK`]).
      committed: u64,
      /// Units the round threw away ([`MAX_PASS_WORK`]).
      thrown_away: u64,
      /// Normal, Share(limit) or Exhausted (section 3).
      mode: RecalcMode,
      /// Set when a run stopped at a pool, not at its own limit (section 3).
      overrun: bool,
  }
  ```

- **Settling.** When `with_area_budget` starts a run, it first settles what the interrupted run
  spent so far (its `limit - work`, since the last settle) into the pools, so a dependency run
  started mid-formula sees the outer run's work. It then gives the fresh run
  `min(formula limit, ceiling left, allowance left)` units, where the formula limit is
  `MAX_FORMULA_WORK` in normal mode and the share in share mode, and the allowance left is
  `MAX_PASS_WORK - committed` in normal mode (no allowance applies in share mode). It records
  whether the formula limit was the smallest. When the run ends, its spend is settled the same way
  (committed, or thrown away when the run is unwinding), and the interrupted run's work left is
  clamped to what the pools have left. Invariant: at every moment the units spent are at most the
  ceiling, and the committed and thrown-away sums are at most `MAX_PASS_WORK` plus one charge.
- **Overrun.** A run whose budget ran out (`work_exceeded`) when its formula limit was *not* the
  smallest bound ran out of a pool, not of its own work. That is not the formula's `#VALUE!`: it
  sets `overrun`, and the pass is abandoned at once like a restart (`evaluation.restart`-style
  flag checked where `run_pass` checks the restart, so every evaluation unwinds without work,
  SPEC-0024 section 3). `evaluate_passes` handles it before `RestartLog::record`, which learns no
  fact from it.

### 3. Which cells give up: normal, share, exhausted
A recalculation starts in **normal mode**: each formula may spend `MAX_FORMULA_WORK`, as today.

**Share mode.** When a round of passes overruns its allowance (committed or thrown away), the round
is abandoned and run again from the same sheet (`restore_dynamic_spills(&spills_before)`, as after a
restart), in share mode: every formula run of the rest of the recalculation may spend at most

```
share = clamp(MAX_PASS_WORK / (4 * N), 1, MAX_FORMULA_WORK)
```

where `N` is the number of formula cells (`Cell::CellFormula` and `Cell::ArrayFormula` anchors) of
the workbook when `evaluate` began. A complete pass then commits at most `N * share`, a quarter of
the allowance, whatever the formulas are. A formula past its share is `#VALUE!` with the message
`TOO_MUCH_RECALC_WORK` ("Recalculation needs too much work to calculate this formula"). Like the
formula budget's error it is set by `with_area_budget` and cannot be caught inside the formula
(`IFERROR(<past its share>,0)` is `#VALUE!`); its readers see an ordinary `#VALUE!` value, which
they may catch, as with a dependency cell past the formula budget (SPEC-0024 section 3).

**Exhausted.** When the ceiling runs out in a round of passes (in either mode), the recalculation
stops: every formula cell of the workbook gets `#VALUE!` with the message `RECALC_EXHAUSTED`
("Recalculation needs too much work"), each dynamic anchor spills nothing and its stale spill cells
are dropped, and the remaining phases do not run. Writing that is linear in the number of cells and
charges nothing.

**Why this is order-independent.** In normal mode each formula's spend is a function of its formula
and its inputs' values (SPEC-0024: each run has its own budget, a dependency's work is its own), so
the committed work of a complete pass is the sum of a fixed set of numbers, whatever the order the
cells are visited in. Whether a round overruns on committed work is therefore a function of the
model. When it overruns, the result is share mode's, which is again a function of the model:
`N` is, and each formula's value under a fixed per-formula limit is. Which cells happened to run
before the allowance ran out does not matter, because that pass is thrown away.

**Where it is not.** Three things can depend on more than the model; each is deterministic for a
given model, its history and its settings, and each is named in the DEVIATION entry:
- The thrown-away work depends on the anchor order the restarts learned (it persists across
  evaluations, evaluation.rs `anchor_order`) and on `set_max_evaluation_depth` /
  `set_max_evaluation_stack`. It can trigger share mode only when it alone exceeds `MAX_PASS_WORK`.
  The doc comments of those two setters ("never changes a result") gain "except a recalculation
  whose thrown-away work reaches the bound of SPEC-0028".
- A run in flight when the allowance runs out counts as committed even if the depth limit would
  later have cut it short.
- Share mode and the ceiling: a share-mode round that restarts often enough to spend the ceiling is
  exhausted, and how often it restarts depends on the learned anchor order. Each share-mode pass
  commits at most a quarter of the allowance, and the normal round spent at most two allowances of
  the ceiling's eight, so this needs more than eight share-mode passes in one round.

### 4. Iterative calculation and data tables
They run after the first round of passes and are not passes: their order is already canonical
(sweeps go in row-major sheet order, iteration.rs; data tables in storage order, data_tables.rs),
so "stop at the bound" is deterministic for them, as their count caps already are.
- **Sweeps.** Each member run draws on the ceiling (and runs under the share in share mode). It does
  not count against the pass allowance. When the ceiling runs out during a sweep, the sweeps stop
  where they are, the member being evaluated keeps its value from before that run, exactly as when
  `MAX_SWEEP_EVALUATIONS` runs out (recalc.md "Iterative sweeps evaluate at most 2^25 members per
  recalculation"). `MAX_SWEEP_EVALUATIONS` stays: it bounds the count of cheap runs and keeps its
  entry.
- **Data tables.** Each run of a substitution draws on the ceiling (and the share in share mode).
  When the ceiling runs out during a result cell, that result cell and every later one are `#NUM!`,
  exactly as when `MAX_DATA_TABLE_EVALUATIONS` runs out (recalc.md "Data tables re-evaluate the
  workbook at most 2^25 formulas per recalculation"). The second round of passes after the tables
  is a round like the first: its own allowance, the same share rule (share mode, once entered, stays
  for the rest of the recalculation), and the ceiling's remainder; if the ceiling runs out there, the
  recalculation is exhausted.

A sweep or data-table run that stops at its formula limit or its share (not at the ceiling) is the
formula's `#VALUE!`, as today.

### 5. Calibration
- **Lower** (Goal 2): step 1 measures, with a local uncommitted instrumentation run over all suites,
  the highest committed work of one pass and the highest total of one recalculation in eval. Both
  must be at most a quarter of their number (`MAX_PASS_WORK / 4`, `MAX_RECALC_WORK / 4`). If not,
  step 1 stops and reports rather than raising the numbers past the upper constraint. The gate
  checks it mechanically: a workbook in share mode turns cells into `#VALUE!`, which
  `make eval-check` refuses as regressions.
- **Upper:** at the slowest kinds SPEC-0024 measured (6.2 to 7.7 s per 2^28 units, 23 to 29 ns a
  unit, release build), a pass at its allowance takes about 25 to 31 s, and a recalculation at its
  ceiling about 100 to 125 s. That is the worst case of directive 5, not the expected time: a
  recalculation that triggers share mode costs at most its normal round (two allowances, about a
  minute) plus share-mode passes of a quarter allowance each. Step 1 measures the worked example
  below to the ceiling and records the times in the DEVIATION entry. The numbers stay powers of two
  and multiples of `MAX_FORMULA_WORK`; step 1 may lower them if the lower constraint leaves room
  (no higher).

**Worked example: 1,000 near-limit cells.** A1:A1000 each hold a formula that computes after
spending `U` = 250,000,000 units (93% of `MAX_FORMULA_WORK`, about 6 to 7 s), and B1:B99000 hold
light formulas `=C1*2` (3 nodes, 18 units).

| case | today | with this spec |
|---|---|---|
| A1:A1000 and B1:B99000, `N` = 100,000 | 2.5E11 units, about 1.6 to 2 hours; every value computed | normal round: after 4 heavy cells and some light ones (4U = 1.0E9 ≤ 2^30), the 5th heavy run gets the 73.7M units left of the allowance and stops at it: overrun, about 25 to 31 s. Share mode: share = ⌊2^30 / 400,000⌋ = 2,684 units; the light cells compute (18 units), the heavy cells stop at 2,684 and are `#VALUE!` (`TOO_MUCH_RECALC_WORK`); the pass commits at most 1000 × 2,684 + 99,000 × 18 ≈ 4.5M units, under 0.2 s. In all about 30 s; every B value correct; every A value `#VALUE!`, whichever heavy cells the normal round happened to run first |
| A1:A1000 alone, `N` = 1,000 | about 1.6 to 2 hours | normal round as above; share = 268,435; the share pass commits at most 2.7E8 units, about 7 s; about 38 s in all; every A value `#VALUE!` |
| A1:A4 alone | 4U, about 26 s | 4U ≤ 2^30: normal mode, every value computed, as today |
| A1:A5 alone | 5U, about 33 s | 5U > 2^30: share mode with share = ⌊2^30 / 20⌋ = 53,687,091; all five `#VALUE!` |

The step from 4 to 5 heavy cells is the price of Goal 3: computing "the first four" would make the
result depend on which four the evaluator visited first.

### 6. DEVIATION entry
Step 1 adds this entry to docs/excel-semantics/recalc.md, with the calibration measured:

> ## One recalculation does at most 2^32 work units, and a pass past 2^30 shares them out
> Rule: Excel has no known limit on the work of a recalculation; a long one runs until done or
> until interrupted, and a workbook that long cannot be probed (Excel for the web's session times
> out).
> Evidence: none for Excel; a resource limit (CLAUDE.md directive 5, SPEC-0028).
> Ours: the formula runs of one recalculation may spend 2^32 work units in all (SPEC-0024's units:
> every pass, restarted pass, sweep and data table). A pass whose formulas commit more than 2^30
> units, or a round whose restarted passes and cut-short runs spend more than 2^30, is run again
> with every formula limited to 2^30 / (4 × the number of formula cells) units; a formula past
> that share is `#VALUE!` ("Recalculation needs too much work to calculate this formula"), which
> `IFERROR` cannot catch. Past 2^32 every formula cell is `#VALUE!` ("Recalculation needs too much
> work"); sweeps then stop where they are and data-table results are `#NUM!`. Which cells give up
> does not depend on the order the cells are evaluated in, except through restarted passes and
> runs cut short by the evaluation depth limit (SPEC-0028 section 3). The heaviest eval pass
> commits <measured> units (<case>), the heaviest eval recalculation <measured> (<case>).
> Status: DEVIATION (decision 0014)

The sweep and data-table entries of recalc.md each gain one sentence: their runs also draw on this
ceiling, and running out of it stops them as their count cap does.

### 7. Incremental recalculation (SPEC-0002 requirement)
There is no incremental recalculation today: every `evaluate` is a full one. M3 (T-0401, SPEC-0002)
must keep "incremental == full recalculation" for workbooks near this bound too:
- The decision to enter share mode is made on the full recalculation's committed work, so the
  graph keeps each formula cell's last normal-mode spend (one `u64` per formula cell) and sums it:
  an incremental recalculation enters share mode exactly when a full one would.
- A cell whose normal-mode spend is unknown (it last ran in share mode and stopped at its share)
  counts as dirty for a normal-mode decision.
- `N`, and so the share, changes when formula cells are added or removed. In share mode every cell
  whose last spend lies between the old and the new share is dirty, and a cell's spend is kept so
  that the graph can find them.
- The thrown-away work of an incremental recalculation counts against the allowance as in a full
  one; the ceiling bounds it.

## Alternatives considered
- **Wall-clock deadline per recalculation.** Exact in time but non-deterministic: the same workbook
  would give different values under load. Rejected, as in SPEC-0024.
- **One pool, cells past it give up in evaluation order.** Simplest: once `MAX_PASS_WORK` is spent
  every later formula run is `#VALUE!`. But which cells those are depends on the anchor order, on
  the order dependencies are reached, and, under M3, on which cells an incremental recalculation
  happens to recompute. Rejected (Goal 3).
- **A static share for every workbook.** Every formula always gets `MAX_PASS_WORK / N`. Order-free
  and needs no re-run, but a workbook of 100,000 light formulas and one legitimately heavy one
  (the eval's heaviest formulas spend 2^27 units) would refuse the heavy one although the whole
  recalculation is cheap. Share mode applies the same rule only to recalculations that overran.
- **Halving the per-formula limit until a round fits.** Order-free too, but needs up to
  `log2(MAX_FORMULA_WORK)` re-runs, each costing up to the allowance. The share is chosen so that
  one re-run fits.
- **Natural-order prefix** (the heavy cells first in `(sheet, row, column)` order keep their full
  budget until the allowance is spent). Order-free in principle, but who is heavy is known only
  after a cell has run, and a heavy cell's value changes its readers' spend; computing the prefix
  needs runs the bound forbids.
- **Counting the sweeps and data tables against the pass allowance.** They have a canonical order
  and a "stop where they are" rule already, so share mode would only make their results worse.
- **A separate counter per charge.** A second subtraction in `spend_work` would put the recalculation
  pool on the hot path; settling at run boundaries keeps SPEC-0024's one-decrement charge.

## Test plan
Counter-based, with test-lowered limits (`#[cfg(test)]` fields like SPEC-0024's
`formula_work_limit`):
- A workbook whose pass commits exactly the allowance computes; one unit more puts it in share mode,
  and the cells past the share are `#VALUE!` (`TOO_MUCH_RECALC_WORK`), the others computed.
- Order independence: the same workbook evaluated after permuting the learned anchor order
  (`anchor_order`) and with formulas placed in different rows gives the same values in normal mode
  and in share mode, including which cells give up.
- `IFERROR(<past its share>,0)` is `#VALUE!`; a reader `IFERROR(A1,0)` of such a cell is 0.
- Settling: a formula whose dependency is heavy draws on the same pass allowance (a dependency run
  started mid-formula sees the outer run's work), and the units a recalculation spent equal the sum
  of its runs' units.
- Thrown-away work: a restart's pass and a depth-limited unwinding add to the thrown-away sum, not
  to the committed one; past `MAX_PASS_WORK` thrown away, share mode.
- Exhausted: past the ceiling every formula cell is `#VALUE!` (`RECALC_EXHAUSTED`), spills are
  dropped, sweeps and data tables do not run.
- Sweeps and data tables (step 2): past the ceiling, sweeps stop where they are and data-table
  results are `#NUM!`; their runs do not move the pass allowance.
- `evaluate_formula` and conditional formats outside `evaluate` draw on no pool.
- The worked example (section 5) at the default numbers in a release-equivalent sense: assert the
  values and the units spent, no timing assertion.
- `make eval`: 0 regressions, no baseline line added. The criterion calc bench must not slow by
  more than 2% (the settle runs once per formula run, not per charge).

## Rollout
1. **T-3008 (M, opus)**: `MAX_PASS_WORK`, `MAX_RECALC_WORK`, `RecalcBudget` in `Evaluation`, the
   settling in `with_area_budget`, overrun detection and the share-mode re-run of a round of
   passes, the exhausted state, the two errors and messages, the setters' doc comments, the
   DEVIATION entry with the calibration measured (section 5). Sweeps and data tables run under the
   share in share mode but do not draw on the ceiling yet. Acceptance: the tests above except
   step 2's, no baseline gains a line, `make check` green.
2. **T-3009 (S, opus)**: the sweeps and the data tables draw on the ceiling and stop as their count
   caps do; the two recalc.md entries gain their sentence. Acceptance: step 2's tests, no baseline
   gains a line.

Nothing stays behind an old path: until step 2 merges, sweeps and data tables keep today's count
caps, and the DEVIATION entry of step 1 says so.
