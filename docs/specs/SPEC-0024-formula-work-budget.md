# SPEC-0024 — A work budget for one formula evaluation

- **Status:** draft (accepted on merge of T-2712, once reviewers approve)
- **Milestone / lane:** M7 / engine
- **Tasks:** T-2712 (this spec); steps 1–3 below are filed as T-2803, T-2804 and T-2805

## Problem
Each array is capped at 2^20 cells (arrays.md "Array results above 1,048,576 cells are #ERROR!").
LAMBDA call depth is capped by a stack share (arrays.md "How deep LAMBDA calls nest"). Lifted
kernels are capped per call (`limits::MAX_LIFTED_CELL_VISITS`, SPEC-0011 "Security rule for kernel
work"). Union walks are capped per formula (area_budget.rs `MAX_AREA_WALK`). None of these caps
bounds the *product* of LAMBDA iterations and the work done in each call. Every capped quantity
is charged again on each call, so a short formula multiplies them. These three formulas, each
under 100 characters, run past the harness's 30 s case timeout (release build, `--timeout 30`,
T-2712):

| formula | element steps |
|---|---|
| `=REDUCE(0,SEQUENCE(100000),LAMBDA(a,b,a+SUM(SEQUENCE(100000))))` | 1E10 |
| `=SUM(MAP(SEQUENCE(1000),LAMBDA(x,SUM(MAP(SEQUENCE(1000),LAMBDA(y,SUM(SEQUENCE(1000))))))))` | 1E9 |
| `=SUM(MAKEARRAY(1000,1000,LAMBDA(r,c,SUM(SEQUENCE(1000)))))` | 1E9 |

`=REDUCE(0,SEQUENCE(3000),LAMBDA(a,b,a+SUM(SEQUENCE(100000))))` (3E8 element steps) takes 20 s,
about 67 ns per element step. One more nesting level multiplies the work by up to 2^20, so a few
more characters reach years. That violates CLAUDE.md directive 5 on untrusted workbooks.

## Goals / non-goals
Goals:
1. One formula evaluation does a bounded amount of work, whatever its LAMBDA nesting. The bound is
   counted in a deterministic unit, not in wall-clock time, so results are reproducible and tests
   use counters (T-2282).
2. No formula that Excel computes in eval loses its value: every eval cell computed today stays
   computed, and no baseline gains a line.
3. One budget, extended from the per-formula budgets that already exist (directive 3). This is not
   a new mechanism alongside them.
4. Once the budget runs out, evaluation stops quickly. A formula past it must not keep running
   2^20 LAMBDA calls that each return an error.

Non-goals:
- Dense single-range walks (T-2710, T-2711, T-2713). They cost something per walk and are bounded
  by the walk budgets; this spec does not cover them.
- The per-call caps (`MAX_LIFTED_CELL_VISITS`, `KernelWork`, the D-function cap of T-2518, the
  lifted-quantile charging of T-2607). They stay as they are. Their charges are also drawn from the
  new budget (step 1), so a cap that is fine for one call cannot be multiplied by LAMBDA.
- The iterative-calculation sweep cap (`iteration.rs` `MAX_SWEEP_EVALUATIONS`) and the whole
  recalculation's cost. The budget is per formula evaluation. A pass costs at most the number of
  formulas times the budget, the same scaling as the area budgets.
- Wall-clock deadlines. They are non-deterministic and are not used (Alternatives).

## Excel behavior to match
Excel has no documented per-formula work limit. It computes long formulas slowly; at worst it shows
the "Excel ran out of resources while attempting to calculate one or more formulas" dialog, which
leaves no cell value the oracle can read. The three formulas above cannot be probed: Excel for the
web's session would time out before giving a value, as the 1,000- to 20,000-level LAMBDA
workbooks did (arrays.md "How deep LAMBDA calls nest": createSession failed). So any bound is a
**DEVIATION** (decision 0014). Directive 5 wins over directive 1 here. The bound must sit above
every formula with Excel evidence (Goal 2). These evidenced formulas are the heaviest LAMBDA
shapes in eval and must stay computed:

- probes/union-walk-repeated-in-lambda Calc!E1:E11 and Big!E1:E4. Excel computes
  `=SUM(MAP(A1:A10000,LAMBDA(r,SUM((B:B,C:C)))))`, 2E8 walked positions, as 80000. The engine
  walks only the existing cells (references.md "Union area cap", T-1497), so in this spec's units
  this is about 10,000 calls × a few dozen units, far below the bound.
- probes/lambda-call-depth H1:H6 and I1:I6 (300-deep self-calls and REDUCE closure chains).
- The m7-argument-modes lifted SEQUENCE and MAKEARRAY cells, which are charged `KernelWork::Output`
  today.
- Every gen-m7 / gen-m10 LAMBDA-helper case. Step 1 measures them (Calibration).

## Design

### 1. What is charged: one *work unit*
The budget counts **work units**. Each unit is roughly one cheap evaluation step, about 10–70 ns
in a release build. Units are charged at three chokepoints, which together cover every way a short
formula can multiply work:

| charge | where | units |
|---|---|---|
| node evaluation | `Model::evaluate_node_in_context`, once per call (after `ungrouped`) | 1 |
| array cells built | where an array value is allocated: SEQUENCE, RANDARRAY, MAKEARRAY, the result of MAP/SCAN/BYROW/BYCOL, `evaluate_range` (range → array), `broadcast_arrays` and the operator/IF broadcasts, `broadcast_text`, the SPEC-0011 lift driver's result, HSTACK/VSTACK/EXPAND/WRAP*/TEXTSPLIT, MMULT | rows × columns, charged before allocating, at the same call that runs `limits::check_array_size` / `check_array_cells` / `check_built_array_size` |
| array cells read | the aggregate walker's array arm (`aggregate_args.rs` `walk_array_items`: SUM, COUNT, AVERAGE, MIN, MAX, PRODUCT and the other `for_each_value` readers), then the other whole-array readers (step 3) | cells of the array argument |
| per-call kernel work | the SPEC-0011 driver's `kernel_units` and the *IF(S) cell visits already computed for `MAX_LIFTED_CELL_VISITS` | the same number, charged again to the formula |

Why these three:
- **Nodes** bound everything that builds no array. Examples are a long scalar LAMBDA body called
  2^20 times by MAKEARRAY, and nested MAPs over small arrays. The LAMBDA call itself needs no
  separate charge, because its body's nodes pay for it.
- **Built cells** bound the three formulas above. Each call builds a SEQUENCE.
- **Read cells** bound the variant that builds once and reads many times:
  `=LET(s,SEQUENCE(100000),REDUCE(0,SEQUENCE(100000),LAMBDA(a,b,a+SUM(s))))`. It builds 2E5 cells
  and reads 1E10.

Union walks keep their own budget (`MAX_AREA_WALK`, 2^24 steps per formula). It is already per
formula and already bounded, so it is not charged twice.

### 2. Where the counter lives
`area_budget.rs` `AreaBudget` already has the right lifetime. `Model::with_area_budget` gives each
formula run a fresh one: `run_cell_formula` (cell formulas and defined names),
`evaluate_outside_cell` (conditional formats, `evaluate_formula`). It restores the outer one
afterwards, so a dependency cell evaluated mid-formula has its own budget. Step 1 extends it and
does not add a second struct (directive 3):

```rust
/// Most work units one formula evaluation may spend (SPEC-0024).
pub(crate) const MAX_FORMULA_WORK: u64 = 1 << 28; // provisional, see Calibration

/// Message of the formula's error once it spent [`MAX_FORMULA_WORK`].
pub(crate) const TOO_MUCH_WORK: &str = "Formula needs too much work to calculate";

pub(crate) struct AreaBudget {
    // ... existing fields ...
    /// Work units left (SPEC-0024).
    work: u64,
    /// Set once a charge did not fit; every later charge then fails at once.
    work_exceeded: bool,
}

impl Model {
    /// Takes `units` from the formula's work budget; past it, marks the run as exceeded and gives
    /// the error. Inlined fast path (one subtraction and branch); the error path is `#[cold]`.
    pub(crate) fn spend_work(&mut self, units: u64, cell: CellReferenceIndex) -> Result<(), CalcResult>;
    /// Whether the run has spent its work budget: the LAMBDA helpers stop their loops on it.
    pub(crate) fn work_exceeded(&self) -> bool;
}
```

The struct's doc comment and the module doc are reworded to say "the budgets of one formula
evaluation". A rename to `FormulaBudget` is optional and, if done, done in the same step.

`evaluate_node_in_context`'s frame size matters. Each self-calling LAMBDA level nests three of its
frames (T-0444). The node charge is therefore a field decrement and a branch to an out-of-line,
`#[cold]` function that builds the error. The probes/lambda-call-depth cells must keep passing
(the 408 and 491 levels in arrays.md "How deep LAMBDA calls nest" may move slightly; the 300-level
cells must not).

### 3. The result past the limit
The error is **`#VALUE!`** with the message `TOO_MUCH_WORK`. That is the error of the two other
per-formula resource caps in the same struct, the union area cap and the seen-range cap
(references.md). Excel shows nothing to match (see "Excel behavior to match"). `#CALC!` and `#NUM!`
were considered: `#NUM!` is the stack-depth and capture caps, which are per call rather than per
formula, and `#CALC!` is Excel's own error for specific dynamic-array cases, which a reader would
take for an Excel result.

**The error cannot be caught.** It is the error of the whole formula, set when its run ends, as the
seen-range cap does (`with_area_budget`'s `seen_ranges_exceeded` path):
`with_area_budget` returns `#VALUE!` (`TOO_MUCH_WORK`) whenever `work_exceeded` is set, whatever
`f` returned. So `IFERROR(<over-budget>,0)` is `#VALUE!`, not 0. A value computed after part of the
work was cut short would be wrong, and a wrong value is worse than an error that says why. No
DEVIATION wording has to cover a caught budget error, because none can be caught.

**Stopping quickly (Goal 4).** After the first failed charge, every charge fails without
subtracting. Each chokepoint returns its error, so each node unwinds at once. The LAMBDA helpers
(MAP, REDUCE, SCAN, BYROW, BYCOL, MAKEARRAY, and the GROUPBY/PIVOTBY aggregation loops) check
`work_exceeded()` after each call and break. A run past the budget thus costs the budget plus one
loop iteration per open helper.

A dependency cell has its own budget (above). If the cell that runs out is a dependency, its own
value is the error, and the outer formula sees an ordinary `#VALUE!` value it may catch. That is
correct: the dependency cell's value *is* `#VALUE!`, as it is when evaluated on its own.

### 4. Calibration
The limit has two constraints:
- **Upper:** a formula at the limit finishes well inside the harness's 30 s timeout. At the
  measured 67 ns per element step, and with ~2 units per element step for the formulas above
  (1 built + 1 read; a node charge per call), 30 s is about 9E8 units. A case can hold several such
  formulas, so the limit is set to at most a third of that.
- **Lower:** every eval formula computed today stays computed (Goal 2). The gate enforces this
  mechanically. A limit set too low turns cells into `#VALUE!`, which `make eval-check` reports as
  regressions, and the gate refuses them.

Step 1 sets `MAX_FORMULA_WORK = 2^28` (268,435,456 units, about 9 s at 33 ns per unit). Before
committing, it measures the highest units any eval formula spends, with a local, uncommitted
instrumentation run over all suites. It records the maximum and the case in the DEVIATION entry. If
the maximum is above 2^26 (a quarter of the limit), step 1 stops and reports it, rather than raising
the limit past the upper constraint.

Worked examples at 2^28:

| formula | units per LAMBDA call | stops after |
|---|---|---|
| `REDUCE(0,SEQUENCE(1E5),LAMBDA(a,b,a+SUM(SEQUENCE(1E5))))` | ≈ 2E5 + 5 | ≈ 1,340 calls, `#VALUE!` |
| `SUM(MAP(SEQUENCE(1000),LAMBDA(x,SUM(MAP(SEQUENCE(1000),LAMBDA(y,SUM(SEQUENCE(1000))))))))` | inner ≈ 2,005 | ≈ 134 outer calls, `#VALUE!` |
| `SUM(MAKEARRAY(1000,1000,LAMBDA(r,c,SUM(SEQUENCE(1000)))))` | ≈ 2,005 | ≈ 134,000 calls, `#VALUE!` |
| `LET(s,SEQUENCE(1E5),REDUCE(0,SEQUENCE(1E5),LAMBDA(a,b,a+SUM(s))))` (step 2) | ≈ 1E5 read + 5 | ≈ 2,680 calls, `#VALUE!` |
| `REDUCE(0,SEQUENCE(1000),LAMBDA(a,b,a+SUM(SEQUENCE(1000))))` (1E6 steps) | ≈ 2,005 | completes, 500500000 |
| probes/union-walk-repeated-in-lambda Calc!E6 | ≈ a few dozen | completes, 80000 |

### 5. DEVIATION entry
Step 1 adds this entry to docs/excel-semantics/arrays.md, with the measured calibration filled in:

> ## One formula evaluation does at most 268,435,456 work units
> Rule: Excel has no known per-formula work limit; a formula whose LAMBDA helpers multiply work
> (`REDUCE(0,SEQUENCE(1E5),LAMBDA(a,b,a+SUM(SEQUENCE(1E5))))`) runs until done, and Excel's
> result for it cannot be probed (the session times out).
> Evidence: none for Excel; a resource limit (directive 5).
> Ours: each formula evaluation (a cell formula, a defined name, a conditional format,
> `evaluate_formula`) may spend 2^28 work units: one per node evaluated, one per array cell built
> or read whole, plus the lifted kernels' own charges (SPEC-0024). Past it the formula is `#VALUE!`
> ("Formula needs too much work to calculate"), which `IFERROR` cannot catch. The heaviest eval
> formula spends <measured> units (<case>).
> Status: DEVIATION (decision 0014)

## Alternatives considered
- **Wall-clock deadline per formula.** Simple and exact in time, but non-deterministic. The same
  workbook would give different values on different machines and under load, and tests would need
  timing (T-2282 forbids that). Rejected.
- **Charge LAMBDA calls only (1 per call).** A call of `SUM(SEQUENCE(1E5))` costs 1E5 times more than
  a call of `x+1`. Bounding calls tightly enough for the first would refuse
  `MAKEARRAY(1024,1024,LAMBDA(r,c,r+c))`. Rejected.
- **Charge each call its body's node count, once at prepare time.** No hook in the hot evaluator,
  but it overcharges short-circuited `IF`s and misses arrays built inside. Rejected in favour of
  counting nodes as they run, which costs one decrement per node.
- **A cost model per function.** Every function would declare its cost from its argument sizes,
  like `KernelWork`. That is precise but has a long tail of 500 functions, and it would leave holes
  wherever one is missing. The three chokepoints cover the multiplying shapes now. Step 3 adds the
  whole-array readers that the aggregate walker does not cover.
- **A separate `WorkBudget` struct with its own reset.** It would duplicate `AreaBudget`'s lifetime
  and its entry-point discipline (directive 3). Rejected.
- **A catchable error at the charge site.** `IFERROR` would then turn a half-computed formula into
  a confident fallback value. Rejected (see section 3).

## Test plan
- Unit tests in area_budget.rs and a new `test/test_formula_work.rs`, counter-based:
  - The units a formula spends, read from the budget after a run through a `#[cfg(test)]` accessor:
    `=1+2` is 3 nodes; `=SUM(SEQUENCE(10))` is the nodes plus 10 built plus 10 read.
  - With a test-only lower limit (a `#[cfg(test)]` override of the model's limit, like
    `set_max_evaluation_stack`), a formula one unit over the limit is `#VALUE!` (`TOO_MUCH_WORK`) and
    one at the limit computes.
  - `IFERROR(<over>,0)` and `IF(ISERROR(<over>),0,1)` are `#VALUE!`.
  - A dependency cell over the budget is `#VALUE!` and the outer `IFERROR(A1,0)` is 0.
  - After an over-budget formula, the next cell's formula has a full budget.
  - Stopping quickly: past the limit, a MAKEARRAY makes at most one more LAMBDA call per open helper
    (a call counter, not a timer).
- The three formulas from the Problem section, the LET variant (step 2) and the 3E8 REDUCE, each as
  an engine test at the default limit in a release-equivalent sense: assert the result is `#VALUE!`
  and the units spent are ≤ the limit + 1 loop iteration. No timing assertion.
- probes/lambda-call-depth H3, H6 keep passing (frame size, section 2).
- `make eval`: 0 regressions, no baseline line added (the gate).
- A benchmark note in step 1's PR. The criterion calc bench (`cargo bench -p rustsheet-engine`)
  must not slow by more than 2% from the per-node decrement. If it does, charge nodes per
  `evaluate_function` call instead and add the call's argument count.

## Rollout
1. **T-2803 (M, opus)**: the budget in `AreaBudget` (`MAX_FORMULA_WORK`, `spend_work`,
   `work_exceeded`), the uncatchable formula error in `with_area_budget`, node and built-cell
   charges, the lifted kernels' charges, the helpers' early stop, the DEVIATION entry with the
   calibration measured. Acceptance: the three Problem formulas finish within the harness timeout
   (as `#VALUE!`), no baseline gains a line, `make check` green.
2. **T-2804 (S, sonnet)**: read-cell charges in the aggregate walker's array arm, and the LET variant
   as a test. Acceptance: the LET variant finishes within the harness timeout, no baseline gains a
   line.
3. **T-2805 (M, opus)**: an audit of the other whole-array readers that a LAMBDA can call on a
   LET-bound array. They are charged the cells they read: MATCH/XMATCH/XLOOKUP/LOOKUP linear
   search, SORT/SORTBY, UNIQUE, FILTER, TEXTJOIN/CONCAT, TOCOL/TOROW, INDEX over arrays (no charge:
   O(1)), SUMPRODUCT, MMULT inputs, the *IF(S) over arrays. The audit lists each one in the
   DEVIATION entry. Acceptance: for each reader, a `LET(s,SEQUENCE(1E5),REDUCE(...,<reader>(s)...))`
   test is `#VALUE!` within the budget, no baseline gains a line.

Nothing stays behind an old path: step 1 is complete for the shapes it covers. Steps 2 and 3 close
the build-once, read-many shapes.
