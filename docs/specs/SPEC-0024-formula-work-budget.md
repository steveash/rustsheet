# SPEC-0024 — A work budget for one formula evaluation

- **Status:** accepted (T-2712; architecture, quality and security reviewers approved); amended by T-2813
  (how per-call capped work is charged and calibrated: sections 1 and 4)
- **Milestone / lane:** M7 / engine
- **Tasks:** T-2712 (this spec); steps 1–3 below are filed as T-2803, T-2804 and T-2805; known gaps
  T-2806, T-2807; the visit caps' 5 s rule (T-2813 amendment) for YIELD and ODDFYIELD is T-2879

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
  recalculation's cost. The budget is per formula evaluation, so a pass costs at most the number of
  formulas times the budget. This is a residual risk: 1,000 cells each near the limit is about
  1,000 times the per-formula time. A per-recalculation bound needs its own design, because the
  result of an over-budget pass, and which cells give up, must be deterministic. It is filed as
  T-2807 (a spec task).
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
in a release build. Units are charged at the chokepoints below. Every multiplying shape the spec
knows of goes through at least one of them. "Known gaps" below lists what is still uncovered.

| charge | where | units |
|---|---|---|
| node evaluation | `Model::evaluate_node_in_context`, once per call (after `ungrouped`) | 1 |
| array cells built | where an array value is allocated: SEQUENCE, RANDARRAY, MAKEARRAY, the result of MAP/SCAN/BYROW/BYCOL, `evaluate_range` (range → array), `broadcast_arrays` and the operator/IF broadcasts, `broadcast_text`, the SPEC-0011 lift driver's result, HSTACK/VSTACK/EXPAND/WRAP*/TEXTSPLIT, MMULT | rows × columns, charged before allocating, at the same call that runs `limits::check_array_size` / `check_array_cells` / `check_built_array_size` |
| array cells read | the aggregate walker's array arm (`aggregate_args.rs` `walk_array_items`: SUM, COUNT, AVERAGE, MIN, MAX, PRODUCT and the other `for_each_value` readers), then the other whole-array readers (step 3) | cells of the array argument |
| text values | the same node chokepoint: a node whose value is a `CalcResult::String` (a function result, a cell read, a LET or LAMBDA variable read, a literal) | ⌈UTF-8 bytes / `TEXT_BYTES_PER_UNIT`⌉ (64 provisional; set by calibration). `broadcast_text` charges the byte total it already sums for its 64 MiB cap (`within_text_budget`) |
| per-call capped work | every per-call cap's spend, charged again to the formula: the SPEC-0011 driver's `kernel_units`, the *IF(S) cell visits counted for `MAX_LIFTED_CELL_VISITS`, GROUPBY/PIVOTBY's visits, the regex engine's compile and match budgets (regex_engine.rs, pcre) | the same number for the visit caps (1 unit per visit or kernel unit, T-2813: see "Per-call capped work" below); a fixed weight per budget step set by calibration for the regex budgets |

Why these charges:
- **Nodes** bound everything that builds no array. Examples are a long scalar LAMBDA body called
  2^20 times by MAKEARRAY, and nested MAPs over small arrays. The LAMBDA call itself needs no
  separate charge, because its body's nodes pay for it.
- **Built cells** bound the three formulas above. Each call builds a SEQUENCE.
- **Read cells** bound the variant that builds once and reads many times:
  `=LET(s,SEQUENCE(100000),REDUCE(0,SEQUENCE(100000),LAMBDA(a,b,a+SUM(s))))`. It builds 2E5 cells
  and reads 1E10.
- **Text values** bound string work, which a node charge of 1 does not.
  `MAP(SEQUENCE(1000),LAMBDA(x,MAP(SEQUENCE(1000),LAMBDA(y,LEN(REPT("a",32767))))))` would
  otherwise do 32 KB of work per unit. Charging every text *value* where it enters the evaluator
  covers producers (REPT, SUBSTITUTE, CONCAT, TEXTJOIN, `&`), whose result is a node value. It also
  covers scanners (LEN, FIND, SEARCH, MID over a long text): the text they scan is itself a node
  value, whether a call result, a cell read or a bound variable read
  (`LET(s,REPT("a",32767),MAP(..,LAMBDA(y,LEN(s))))` pays at each read of `s`). Text elements of
  arrays are charged by their length where step 3 charges array reads.

Known gaps (out of scope, recorded for later work):
- **Retained memory.** The budget counts work, not what a formula holds at once. Memory stays bounded
  by the existing per-value caps: arrays of 2^20 cells, 32,767-unit texts, `MAX_CAPTURED_CELLS`, and
  the nested-array caps. 2^20 cells of 32 KB texts is still about 64 GB in principle, as it is today.
  This is pre-existing and is filed as T-2806.
- **Super-linear work inside one scalar call** that has no per-call cap and touches no text: none is
  known. A function found to have one gets a per-call cap of its own, charged to the formula like the
  others.

Union walks keep their own budget (`MAX_AREA_WALK`, 2^24 steps per formula). It is already per
formula and already bounded, so it is not charged twice.

#### Per-call capped work (amendment, T-2813)
**Rule.** A visit cap's spend (the lift driver's cell visits and `kernel_units`, the *IF(S) and
AGGREGATE visits, GROUPBY/PIVOTBY's visits, all counted against `MAX_LIFTED_CELL_VISITS` = 2^27) is
charged to the formula at **1 unit per visit or kernel unit**, everywhere, inside a LAMBDA or not.
The time a call's units take is not the formula budget's to fit: it belongs to the call's own cap.
SPEC-0011's security rule for kernel work makes each kernel weight a measure of time (T-2814 fitted
RATE's to about 18 ns a unit at the cap). Since `MAX_FORMULA_WORK` is twice
`MAX_LIFTED_CELL_VISITS`, a formula spends at most two caps' worth of capped work (`spend_work`
refuses a charge larger than what is left, so two calls exactly at the cap plus any node already
exceed it). So this kind's 10 s target (section 4) holds exactly when **every visit cap's worst
case, one call at its cap, runs within 5 s**. That is a requirement on the kernel weights
(SPEC-0011), and a weight that breaks it is a perf task against that kernel, not a change to this
budget.

Why not the alternatives T-2813 weighed:
- *A weight above 1.* An eval cell may legitimately spend most of one cap on one call
  (gen-m9/0a614cfe974b59c3 Calc!A117, 124,806,376 units). Any weight above 2 turns such a cell into
  `#VALUE!`; 6 cost 13 gen-m9 cells (T-2803). A higher weight would also make the formula budget,
  not the per-call cap, the real limit of one call, so two limits would decide one thing.
- *Charging only inside a LAMBDA call.* It needs a call-depth flag on every charge, and it leaves the
  non-LAMBDA product open: a formula of a few hundred lifted calls side by side
  (`SUM(RATE(..),RATE(..),..)` within 8,192 characters) multiplies the per-call cap by the call
  count with no helper involved. Charging everywhere bounds that too, at the cost of refusing a
  formula with two calls each near its cap (one call at its cap plus a quarter-limit of other work
  always computes, section 4); no eval formula has even two such calls.
- *Lowering the per-call cap or raising the formula limit for this kind.* The first refuses the eval
  cells above; the second raises the time every other kind may take, and the limit stays one
  number (section 4).

The regex compile and search budgets are not visit caps: they keep the fixed weights per budget
step that calibration sets (section 4).

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
    /// Takes `units` from the formula's work budget with `checked_sub` (never a wrapping `-`),
    /// past it marks the run as exceeded and gives the error. Inlined fast path (one subtraction
    /// and branch); the error path is `#[cold]`. Callers compute `units` with saturating
    /// arithmetic (`rows.saturating_mul(columns)`, summed kernel units), so an oversized charge
    /// saturates to `u64::MAX` and fails; it never wraps into a small one.
    pub(crate) fn spend_work(&mut self, units: u64, cell: CellReferenceIndex) -> Result<(), CalcResult>;
    /// Whether the run has spent its work budget: the LAMBDA helpers stop their loops on it.
    pub(crate) fn work_exceeded(&self) -> bool;
}
```

The limit the budget starts from is `MAX_FORMULA_WORK`. Tests lower it through a `#[cfg(test)]` field on `Model`
(`formula_work_limit`, read by `AreaBudget`'s fresh-budget constructor in `with_area_budget`). It is not a public setter like
`set_max_evaluation_stack`: the budget is a security bound, not an API option, so no public surface is added.

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
- **Upper:** a formula at the limit finishes well inside the harness's 30 s timeout, whatever
  kind of unit it spends. The only measurement so far is 67 ns per element step of
  `REDUCE(..,SUM(SEQUENCE(1E5)))`, about 2 units, so about 33 ns per unit for built and read
  cells. A node unit (a LAMBDA call's binding, a function dispatch) or a text unit may cost far
  more, so 33 ns is not assumed for them. Step 1 times one worst case per charge kind, run until
  the budget stops it (release build):
  - nodes: `=SUM(MAKEARRAY(1024,1024,LAMBDA(r,c,MAP(SEQUENCE(1),LAMBDA(z,r+c+z)))))` and a nested MAP with a
    scalar body;
  - built cells: the three Problem formulas;
  - read cells: the LET variant;
  - text: `MAP(SEQUENCE(1000),LAMBDA(x,MAP(SEQUENCE(1000),LAMBDA(y,LEN(REPT("a",32767))))))`
    and the `LET(s,REPT(..),..LEN(s)..)` variant;
  - per-call caps: a MAKEARRAY of lifted RATE and a MAP of REGEXTEST calls (amendment, T-2813:
    also a MAKEARRAY of lifted YIELD; the visit caps' target is a per-call one, below).
  The weights (units per node, `TEXT_BYTES_PER_UNIT`, units per regex step) are chosen so that each
  kind's worst case reaches the limit in at most **10 s**. A case can hold several such formulas.
  Weights may differ per kind; the limit stays one number.
- **Lower:** every eval formula computed today stays computed (Goal 2). The gate enforces this
  mechanically. A limit set too low turns cells into `#VALUE!`, which `make eval-check` reports as
  regressions, and the gate refuses them.

Step 1 sets `MAX_FORMULA_WORK = 2^28` (268,435,456 units) and fits the weights to the timings above.
Before committing, it measures the highest units any eval formula spends, with a local, uncommitted
instrumentation run over all suites. It records the maximum, the case and the per-kind timings in
the DEVIATION entry. If the maximum is above 2^26 (a quarter of the limit), or no weights meet both
constraints, step 1 stops and reports it rather than loosening either constraint.

**Per-call capped work (amendment, T-2813).** The quarter-limit margin is measured without the
visit caps' charges, which have a margin of their own:
- *Lower, other kinds:* the heaviest eval formula's units outside the visit caps' charges stay at or
  below 2^26.
- *Lower, visit caps:* the heaviest eval formula's visit-cap charges, summed over all its calls,
  stay at or below one cap, `MAX_LIFTED_CELL_VISITS` (2^27). Together with the line above, every
  eval formula then stays below 2^26 + 2^27, three quarters of the limit, so a formula of one call
  at its cap plus a quarter-limit of other work always computes, and the formula budget refuses no
  single call that its own cap allows. A second call near its cap in the same formula may be
  refused.
- *Upper, visit caps:* each visit cap's worst case, one call at its cap, runs within 5 s (release
  build), so two such calls, the most the limit allows, stop within 10 s ("Per-call capped work" in
  section 1).

Measured for T-2813 (release build, 2026-10-27, the T-2803 branch with origin/main merged in, local
instrumentation of `with_area_budget`, not committed): `rustsheet-eval check` has 0 regressions and
no formula past the limit. The heaviest eval formula spends 118,754,196 units, 118,751,450 of them
visit-cap charges (below 2^27) and 2,746 other units (a Calc!A204 cell; the 124,806,376 of T-2803
predates the kernel charges T-2814 and T-2815 changed). The heaviest outside the visit caps spends
8,388,620 units, none of them visit-cap charges (2^23, below 2^26). Both lower constraints hold.
Time to the limit, each a `SUM(MAKEARRAY(1000,1000,LAMBDA(r,c,SUM(<call>))))`:

| `<call>` | one call alone | to the limit |
|---|---|---|
| `RATE(3000+0*SEQUENCE(16384),-1,100)` (all 200 steps, at the cap) | 2.1 s | 2.1 s (the second call's charge fails) |
| `RATE(3000+0*SEQUENCE(1000),-1,100)` | | 4.2 s (32 calls) |
| `YIELD(43831,47484,0.05,1E-100*SEQUENCE(12329),100,2)` | | 17.6 s |
| `YIELD(43831,80355,0.05,1E-100*SEQUENCE(644),100,4)` | 15.0 s | 28.1 s |
| `REGEXTEST(REPT("a",32767)&SEQUENCE(100),"(a|aa)*b")` (a 100 by 100 MAKEARRAY) | | 0.8 s |

The YIELD and ODDFYIELD shapes miss the 5 s rule today, as one call of theirs at the cap takes
about 17 s; T-2879 refits their weight (about 4.3 s at 2^26, measured by T-2814) and owns the gap.
As built (T-2879): at 2^26 one call at the cap takes at most 4.34 s (financial.md "Yield and odd-coupon work cap").
T-2803 does not wait for it: without the budget the same MAKEARRAY never stops, and with it it stops
within two caps' worth of calls. T-2803 re-measures these figures on its own head and records them
in the DEVIATION entry (section 5).
Until T-2879 merges, the DEVIATION entry names that gap (a MAKEARRAY of YIELD takes about 28 s to
the limit) and T-2879. The other kernels near the cap in eval (CHISQ.DIST, GAMMAINV, POISSON.DIST,
ODDLPRICE) and the *IF(S), AGGREGATE and GROUPBY/PIVOTBY visits were not timed here: SPEC-0011's
rule fits each kernel weight to time, and T-2814 timed IRR, XIRR, ODDLYIELD and PRICE within it;
T-2803 times one call at the cap for each of these families when it resumes and files a perf task
for any over 5 s. A formula refused for two near-cap calls is covered by the same DEVIATION entry.

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
> `evaluate_formula`) may spend 2^28 work units: per node evaluated, per array cell built or read
> whole, per 64 bytes of each text value, plus the per-call caps' own charges (SPEC-0024). Past it the formula is `#VALUE!`
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
    `=LEN(REPT("a",640))` charges the nodes plus 10 text units for the REPT result (64 bytes per unit).
  - With a test-only lower limit (the `#[cfg(test)]` `formula_work_limit` field of section 2), a formula one unit over the limit is `#VALUE!` (`TOO_MUCH_WORK`) and
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
   `work_exceeded`), the uncatchable formula error in `with_area_budget`, node, text and
   built-cell charges, the per-call caps' charges, the helpers' early stop, the weights fitted to
   the per-kind timings (section 4), the DEVIATION entry with the calibration measured. Acceptance:
   the three Problem formulas finish within the harness timeout (as `#VALUE!`), each kind's worst
   case stops within 10 s, no baseline gains a line, `make check` green. The visit caps are charged
   at 1 unit and judged by the amended rules of section 4 (T-2813); their YIELD and ODDFYIELD
   shapes are T-2879's, not a blocker of this step.
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
