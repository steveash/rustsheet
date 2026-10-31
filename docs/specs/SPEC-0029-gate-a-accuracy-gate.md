# SPEC-0029 — Gate A, the accuracy gate

- **Status:** draft (T-3355; accepted once the architecture reviewer approves)
- **Milestone / lane:** M0 / harness
- **Tasks:** T-3355 (this spec); rollout T-3356, T-3357, T-3358, T-3379, T-3380; T-3359 reads the result

## Problem
Decision 0188 makes accuracy Phase 1's first gate (R1(a), R2). Today no gate compares the engine with the math:

- The eval gate (`rustsheet-eval check`) forbids bit regressions against Excel on committed cells, and the report
  measures closeness to Excel only. Of the 248,892 numeric cells that differ from Excel, 99.5% agree to 12 digits or
  more, so the rate is dominated by Excel's rounding noise on research sweeps.
- Real bugs that users hit are caught by no gate: IRR and RATE `#NUM!` on ordinary inputs, T.INV.2T losing 5 orders of
  magnitude near p = 1, CHISQ.INV `#NUM!` at df ≥ 2e7, BINOM.DIST off 1e-9 at n ≥ 6e5, F.DIST off 5e-4 at x ≈ 1e-12
  (decision 0188, "The engine is wrong where Excel is right", items 1–7).
- The fit lab scores a fit on the cells it was tuned on. On 120 fresh stratified points per function, T.INV is within
  15 digits of Excel on 30% (69% on the public cases), T.INV.2T on 14% (51%): the fits do not generalize.

## Goals / non-goals
Goals:
1. Every numeric function in scope has a committed, stratified **reference table** (about 40% typical, 30% tails and
   domain edges, 20% extreme-but-valid, 10% log-uniform; R2). Each point carries a 50-digit mpmath value and, once
   filled, Excel's value.
2. `make check` (and CI's `eval-gates`) fails on a point outside the **envelope**: neither within ε = 1e-9 relative of
   the math (absolute floor 1e-300) nor within 15 significant digits of Excel, or an error against a number either way.
   R3 points are checked against the math only.
3. One envelope definition, shared by the gate, by R1(c)'s probe rows and by Gate B's R3 cells (T-3359), and one way to
   attach a math value to any eval cell, so the probes-numeric sweep cells deferred by R5 stop counting against the exit
   once they are within ε of the math.
4. R4.2's fresh log-uniform sweep is a table batch, not a second format.

Non-goals:
- Fixing the points that fail on landing (the P0 tasks T-3347 to T-3354, T-3367 own them).
- Changing the bit-regression eval gate, `eval/baseline/` or any `quality/*.json` ratchet. Gate A has no baseline
  file: it is an absolute check with a known-failure list whose every line names an open task.
- Mpmath at check time. `make check` reads committed values; mpmath runs only in tools/ when a table is generated.
- Formulas other than one call of the function with literal arguments (no nesting, no references) in the tables.
  Composite formulas stay eval's job.

## Excel behavior to match
The tables are inputs to Excel, so they follow the formula-literal rules Excel imposes (numbers.md):
- **Literals are cut to 15 significant digits** ("15-digit literal parsing", "Formula number literals are read at 15
  significant digits"). The generator writes every argument with at most 15 significant digits, and the math is
  evaluated at the double nearest that decimal, which is the double Excel and the engine read. No point depends on the
  truncation itself.
- **No literal of 1E+308 or more** ("Formula literals of 1E+308 or more are invalid"): Excel for the web rejects the
  workbook. Arguments are bounded by 9.99999999999999E+307; a literal below the smallest normal double is not written
  ("Subnormal results become 0"), the argument is 0 or a normal double.
- **Subnormal results become 0** for many functions, and excel-graph saves a kept subnormal as 0 (decision 0042,
  decision 0089). The absolute floor 1e-300 of R2 puts every such point inside the math envelope whatever either side
  does, so no table depends on the flush.
- **Integer parameters are truncated** by many functions (df, n, number_s, basis). That is argument semantics, which
  eval owns: the generator draws integer parameters as integers, so the math needs no truncation rule.
- **Excel's errors are matched** (R3's last rule). The math says only whether a point is in the function's domain; the
  error code is Excel's.
- **Where Excel is far from the math** (R3): only the regions decision 0188 lists, as `MATH-OVER-EXCEL` entries in
  statistics.md and financial.md, are checked against the math alone; everywhere else Excel's imprecise value is a
  valid target (BESSELJ at 1e-7, XIRR at 1e-8 are within the envelope through the 15-digit side only if the engine
  matches Excel, or through ε if it matches the math).

## Design

### Location and files
```
eval/accuracy/
  suite.toml                    # kind = "spec", gate = false, baseline = "none": discoverable, never in `make eval`
  tables/<FUNCTION>/<batch>.case.json     # rustsheet-case/v1, generated, one cell per point
  tables/<FUNCTION>/<batch>.oracle.json   # rustsheet-oracle/v1, written by `rustsheet-oracle fill`
  tables/<FUNCTION>/<batch>.math.json     # rustsheet-math/v1, written by the generator
  known-failures.tsv            # failing points carried until their owner task lands
  math-over-excel.tsv           # R3 marks: point (or eval cell) → MATH-OVER-EXCEL entry
  exempt.tsv                    # in-scope numeric functions that need no table, with a class
```
`eval/accuracy/` is outside `eval/suites/`, `eval/baseline/` and `quality/`. Because `Suite::load_all` scans
`eval/suites/` only, `make eval`, the eval gate, the evidence floor and every report measure never see the tables.
`rustsheet-eval cases accuracy` and `rustsheet-eval show accuracy/<FUNCTION>/<batch>.case.json` resolve the manifest
through one helper (`Suite::load_named(root, name)`: `eval/suites/*` first, then `eval/accuracy/suite.toml`), so the
oracle fill, `make eval-show` and the fit lab's sweep reader work on tables unchanged.

**One table per function, one file per batch.** A function's table is every batch in its directory. A batch is
generated once and never edited (T-3379 makes committed batch files immutable): new points are a new batch (`s1` is
the first table, `r4-T-XXXX` an R4.2 fresh sweep, `e-T-XXXX` points an investigation adds). This keeps the fill
incremental (`fill` writes a sidecar only where none exists), gives each batch its own `input_sha256`, and makes
"never weaken" mechanical: a diff may add files under `tables/`, never change or delete one.

### Point fields
A point is one cell `Calc!A<n>` of a batch, with its id `accuracy/<FUNCTION>/<batch>!Calc!A<n>` (the runner's
`cell_id`). Its fields live in the three files:

| field | where | form |
|---|---|---|
| formula | case `sheets[0].cells` | `=T.INV(1.87E-297,3937)`: one call, literal arguments of at most 15 significant digits |
| arguments | case `hypotheses` | `gate-a/<stratum>/<arg1>,<arg2>,...` (the hypotheses map's existing convention) |
| math value | math sidecar | `{"t":"n","v":"-40.260...e0"}`: 50 significant digits as text (`mpmath.nstr(v, 50)`) |
| math domain error | math sidecar | `{"t":"e","why":"p outside (0,1)"}`: no error code, Excel's code is the target |
| stratum | math sidecar | `"s": "typical" \| "tail" \| "edge" \| "extreme" \| "log-uniform"` |
| Excel value | oracle sidecar | the existing `{"t":"n","v":...}` / `{"t":"e","v":"#NUM!"}` |
| fill metadata | oracle sidecar `source` | backend, application, `produced_at`, `input_sha256` (unchanged) |
| R3 mark | `math-over-excel.tsv` | `<point id>\t<file>.md#<anchor>` of a `Status: MATH-OVER-EXCEL` entry |

The math sidecar's top level is `{"format":"rustsheet-math/v1","source":{"generator":"rustsheet-accuracy","version":
<tool version>,"dps":60,"seed":<int>,"input_sha256":<sha256 of the case file>},"sheets":{"Calc":{"A1":{...}}}}`, the
oracle sidecar's shape. Each cell also carries `"f"`, its formula text: a reader drops (and reports as stale) a math
value whose `f` differs from the cell's formula, so a math value can never attach to the wrong formula. The 50 digits
are parsed by Rust's `str::parse::<f64>` (correctly rounded); a math value beyond the double range is never written
(see "Sampler", overflow).

### The generator and the sampler (tools/, T-3357)
`tools/src/rustsheet_tools/accuracy/`, CLI `rustsheet-accuracy` (ruff, mypy strict), `mpmath` a pinned uv dependency of
tools/ only (pure Python, BSD; the engine and the bindings never depend on it).

- **Definitions.** `accuracy/defs/<family>.py` registers, per function, a `Definition`: the parameters (name, kind
  real or integer, valid domain with open or closed bounds, typical range, extreme range, optional tail map) and the
  mpmath function at the working precision. Inverses solve the forward mpmath function with `mpmath.findroot` on a
  bracket; solvers (IRR, RATE, XIRR, YIELD) solve their defining equation and return Excel's root by Excel's root
  choice where several exist (the definition cites the semantics entry; with no evidenced choice, points with several
  roots are not drawn). The parameter list comes from the catalog (`eval/catalog/functions.yaml`: names, order,
  `int` domains); the catalog's `domain` and `pool` seed the typical range where the definition gives none, but the
  catalog's ranges are the synthetic generator's and are not the valid domain (T.INV's `max: 10` for df is a typical
  value, df goes to 1e10), so the valid and extreme ranges are the definition's, each with a cited source
  (a semantics entry, Excel's documented limit, or a probe).
- **Precision.** Each value is computed at dps 60 and again at dps 80; a point whose two values differ beyond 1e-45
  relative is not converged and is dropped (logged with its arguments). A point whose evaluation takes more than 10 s is
  dropped the same way. The output is the dps-80 value rounded to 50 significant digits.
- **Determinism.** The seed of a batch is `int(sha256(f"{FUNCTION}/{batch}"), 16) % 2**63`, fed to one
  `random.Random`; the sampler never reads the clock or the environment, and dict iteration is by sorted key. The
  same function, batch and tool version write byte-identical files (a unit test).
- **Strata** (shares of a batch's points, rounded so they sum to the count):
  - **typical, 40%:** every parameter uniform over its typical range, or log-uniform when the range spans more than two
    decades.
  - **tail and domain edge, 30%:** half are tails: one parameter at a time (round-robin) moves toward a finite domain
    bound at a log-uniform relative distance in [1e-15, 1e-1] (p = 1e-k and 1 − 1e-k), and a distribution's x is set
    through the tail map (x = the quantile of a log-uniform tail probability, so a cdf point really is in the tail).
    The other half are edges: each valid bound value itself (df = 1, x = 0, p = 1 where valid), its nearest
    15-digit neighbours inside, and the nearest invalid neighbours outside (math domain error; the point checks the
    engine returns Excel's error).
  - **extreme-but-valid, 20%:** one or more parameters log-uniform over the extreme range (shape, df, n, periods up to
    the largest value Excel accepts; x where the result nears the double's overflow or underflow).
  - **log-uniform, 10%:** every parameter log-uniform (sign uniform for a signed one) over its whole valid domain:
    R4.2's sweep, and the shape of every `r4-*` batch (100% log-uniform).
  - A drawn point whose math value overflows the double range is redrawn (at most 20 tries, then dropped); a math
    underflow (|v| < 1e-300) is kept, since the floor makes it a check of the result's sign and type.
- **Counts.** A first batch has 120 points for a function of up to two numeric parameters (48/36/24/12), 200 for three or
  more, and 300 for a solver (IRR, RATE, XIRR, YIELD, PRICE: the cash flow or the bond is drawn as a whole). With
  about 300 functions that is about 50,000 points: about 6 MB of case, oracle and math JSON, and well under the check's
  budget below.
- **Commands:** `rustsheet-accuracy generate FUNCTION [--batch s1] [--count N] [--stratum S]` writes a batch's case
  and math sidecar; `rustsheet-accuracy verify FUNCTION` recomputes the math of committed points at dps 80 and reports
  any difference (a definition change must not silently change committed truth: a changed value is a new batch plus a
  decision record, T-3379); `rustsheet-accuracy math --suite S` attaches math to eval cells (T-3380).

### The oracle fill and refill
- **Fill:** `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite accuracy`. Nothing in the fill changes:
  it builds each batch's xlsx with the spec→xlsx builder, recalculates it under the canary, and writes the sidecar.
- **Pending:** a point is pending while its batch has no oracle sidecar or the sidecar has no value for its cell
  (undecoded, refused). A pending point is checked against the math only: it passes within ε of the math, or when the
  math says domain error and the engine returns an error. The check's summary counts pending points, and Gate A does
  not hold while any is pending (below).
- **Refill:** never needed for a committed batch, since a batch never changes. `fill --force --case` on a batch is a
  change to a committed file and needs a decision record (T-3379), as an oracle recalibration would.
- **A math domain error against an Excel number** means the definition's domain is wrong. `fill` does not decide it;
  the check fails the point as a table defect (`TABLE`), and the batch is replaced by a corrected one in the same PR.

### The check (`rustsheet-eval accuracy`, T-3356)
A new subcommand of the release harness binary, in `crates/rustsheet-eval/src/accuracy.rs` (with `accuracy/` for its
parts if it outgrows one file), run by `make eval-gates` right after `check --evidence-floor` (so `make check` and CI's
`eval-gates` job run it; scripts/ci_changes.py routes `eval/accuracy/**` and `tools/src/rustsheet_tools/accuracy/**`
to that job).

1. Load the accuracy manifest, every batch, its oracle sidecar (the runner's existing reader) and its math sidecar.
2. Evaluate each batch once with the eval engine adapter, through the runner's panic isolation and per-case timeout.
   The runner gains one entry point that returns the engine's value of every `check` cell, sharing `run_case`'s
   preparation, so pending points get engine values too. A batch that crashes or times out fails every point in it.
3. Classify each point with **`envelope`**, the one definition (also used by T-3359 and T-3380):

   ```rust
   pub enum Envelope { Excel15, Math, Both, Outside, ErrorMismatch, Table }
   pub fn envelope(excel: Option<&CellResult>, engine: &Value, math: Option<&MathValue>, r3: bool) -> Envelope;
   ```

   - The Excel side passes when the point is filled and `!Visible::is_off(cell)` on the `CellResult` the runner's own
     scoring builds from Excel's value and the engine's (decision 0083's 15-digit rule, the 4-ulp precedence of decision
     0088, decision 0042's subnormal rule; never a second 15-digit rule).
   - The math side passes for a number when `|engine − math| ≤ max(1e-9 · |math|, 1e-300)`, with math the double
     nearest its 50 digits, and for a math domain error when the engine returns an error whose code equals Excel's,
     or any error while pending.
   - An R3 point ignores the Excel side.
   - `ErrorMismatch`: one of engine and Excel is an error and the other a number (R2), or both are errors with different
     codes. An R3 mark never turns it into a pass (R3 never applies to an error against a number).
   - `Table`: a math domain error against an Excel number, a stale math value (`f` differs), or a point with no math
     value: a defect of the table, not of the engine.
   - `Outside` and `ErrorMismatch` fail; `Table` fails.
4. **Known failures.** `eval/accuracy/known-failures.tsv`, lines `<point id>\t<owner task>\t<note>`, carries a failing
   point until its owner task lands, never by changing ε or the 15-digit rule. A listed failing point is reported, not
   failed. A listed point that passes is a stale line and fails the check (remove it; as for deviations). T-3379 adds
   the rules a reviewer cannot be trusted to apply: the owner task exists and is not `done`, and a line may be added
   only for a point added in the same diff, so a regression of a committed point can never be listed.
5. **R3 marks.** `eval/accuracy/math-over-excel.tsv`, lines `<point or eval cell id>\t<file>.md#<anchor>`. Each line's
   entry must have a `Status: MATH-OVER-EXCEL` line (the anchor check of `deviations::check_entry`, given the status to
   require instead of a second copy), and a filled marked point must show Excel off the math by more than G = 1e-4
   relative, or the mark has no evidence and fails. The file is a protected path (T-3379): every change needs a
   decision record, and a line citing an entry that decision 0188 or a later owner-ruled record does not list is a new
   R3 case, which needs an owner ruling (decision 0188 R3). Lines naming eval cells (`probes-numeric/...!Calc!A7`) are
   the R3 cells T-3359 counts as passing in Gate B and R1(c); they stay F lines of eval/baseline.
6. **Output.** One summary line, then the failures, worst first by relative error against the math:

   ```
   accuracy: <points> points in <tables> tables (pending <n>): <n> in envelope (Excel 15 digits only <n>,
     math only <n>), <n> known failures (T-XXXX <n>, ...), <n> failing, <n> table defects; missing tables <n>; <s> s
   FAIL accuracy/T.INV.2T/s1!Calc!A<n> tail =T.INV.2T(0.999999999989522,2)
     engine 1.49…E-08  excel <Excel's value>  math 1.48…E-11  rel(math) 1.0E+03
   ```

   `--json PATH` (default `eval/runs/accuracy.json`, git-ignored) writes every point's class, values and table, which
   the eval report and the progress dashboard read for Gate A's status (T-3359). Exit 1 on any failing point, stale
   line or table defect.
7. **Budget.** The check's wall time on the CI runner stays under 30 s for the whole set (about 50,000 points of one call each,
   so reading the JSON is expected to dominate). The summary prints the time; T-3356
   reports it on fixtures and T-3358 on the real tables, and a slice that would pass the budget splits the work first.

### Gate A holds (R1(a))
The check passing is what `make check` requires. Gate A **holds**, the Phase 1 condition, when in addition:
no line in `known-failures.tsv`, no pending point, and no **missing table**: every catalog function with `scope: in`
and `returns: number` has a table directory or an `exempt.tsv` line (`<FUNCTION>\t<class>\t<reason>`, classes
`exact` for results with no floating-point kernel such as COUNT, ROW, LEN, MATCH, DATE; `composite` for functions a
table cannot drive with scalar literals, with the eval suites named instead). `exempt.tsv` is a protected path
(T-3379). The check prints the three counts; T-3359's report shows Gate A as held, or not held with them.

### Math references for eval cells (R1(c), T-3380)
The same `rustsheet-math/v1` sidecar, `<case>.math.json` next to `<case>.oracle.json` (under `oracle_dir` with the
same mirroring for a fetched suite), attaches math values to the cells of any eval suite:
- `rustsheet-accuracy math --suite probes-numeric` reads each case through `rustsheet-eval cases`, selects every checked
  cell whose formula is one call `=FUNC(args)` of a function with a definition, parses its arguments with the fit lab's
  sweep reader (`fitlab/sweep.py`: literals, constant cells, arrays, `DATE`, and arithmetic over literals with Excel's
  operator model), and writes the math value with its formula text. A cell whose arguments read another formula cell,
  or that the reader leaves unparsed, gets no math value. Strata do not apply (`"s"` absent).
- The runner reads the sidecar when present and puts the math value on the cell's `CellResult` (an optional field,
  absent from the JSON when none), dropping a stale one. R1(c) counts a visibly-off numeric cell as inside the envelope
  when `envelope` says so; a cell with no math value counts as outside (decision 0188 R1(c)), so nothing reads as met
  early.
- Size: probes-numeric's numeric cells add about 20 MB of math sidecars. Their generation is a tools run, committed like
  an oracle fill.

### The R4 guards
- R4.2's fresh log-uniform sweep is a batch `r4-T-XXXX` of the function's table, generated after the fit is frozen and
  filled before merge; the fit task records its check result, and the batch stays in the gate from then on.
- The tables never feed a fit: the fit lab's train and hold-out split (R4.1) is over eval cells, and a table batch
  used to tune a constant would no longer measure generalization. The fit lab refuses `accuracy/*` cases as a sweep
  unless `--holdout-only` is given (T-3357 adds the refusal).

## Alternatives considered
- **Tables as a suite under `eval/suites/`.** Gets discovery for free, but every suite-wide measure (the evidence
  floor, visible parity, the Phase 1 rate, the probe rows) would count the points, and the ratchets would move when a
  table is added. Rejected; the manifest outside `eval/suites/` gets the fill and `show` without that.
- **One file per function, refilled on change.** Matches the task's first reading, but every added point rewrites a
  committed file and its Excel values, so append-only and "never weaken" cannot be checked mechanically, and every
  refill re-runs the whole table through the oracle. A directory per function with immutable batches keeps one table
  per function and both properties.
- **A bespoke table format (TOML or TSV of arguments).** Needs its own xlsx builder for the fill, its own engine
  driver and its own `show`, against directive 3. The case format with two sidecars reuses all three.
- **Math values as f64 or 17 digits.** Loses the evidence that a point is exact to well past ε and ties the tables to
  one comparison. 50 digits as text cost about 60 bytes a point.
- **Big-decimal comparison in Rust.** Unneeded: ε = 1e-9 is seven orders of magnitude above the error of rounding the
  math to a double, so the check parses to f64 and adds no dependency.
- **A ratcheted baseline of failing points.** Would let a regression of a committed point be recorded by a ratchet
  update. A known-failure line per point, owned by an open task and allowed only for new points, cannot hide one.
- **Mpmath in `make check`.** Hundreds of findroot calls per run, and a Python dependency in the gate. Committed values
  plus `rustsheet-accuracy verify` give the same guarantee.

## Test plan
- T-3356: unit tests of `envelope` for a point in both, within ε of the math only, within 15 digits of Excel only,
  outside both, the absolute floor, an error against a number either way, two different error codes, a math domain
  error matched by an error, an R3 point that is within 15 digits of Excel but not ε of the math (fails), a stale math
  value; a fixture table under the crate's test data with one known failure, one stale known-failure line and one
  failing point, run end to end through the subcommand (exit codes, output lines); the timing on the fixture.
- T-3357: determinism (two runs, identical bytes); the definitions of one distribution (T.DIST), one inverse (T.INV,
  including `T.INV(1.87E-297,3937)` ≈ −40.26) and one financial solver (RATE, including the decision 0188 item 2
  annuity at 0.0431054691928255) against published values; strata shares and bounds; dropped non-converged points.
- T-3358: the batch's committed tables pass the check with every failing point named in known-failures.tsv with its
  owner task; the R3 points of decision 0188's initial cases are marked and their entries cite them.
- T-3379: governance and hygiene tests: modifying or deleting a committed batch file, adding a known-failure line for a
  committed point, an owner task that is done, and an `exempt.tsv` or `math-over-excel.tsv` change without a decision
  record each fail.
- T-3380: a probes-numeric case with a math sidecar: a visibly-off cell within ε counts inside, a stale or absent math
  value counts outside, the CellResult JSON is unchanged for a cell without math.

## Rollout
1. **T-3356** (M): the format reader, `envelope`, the subcommand, known failures, R3 marks, `make eval-gates` wiring
   (a Makefile change, with its decision record), fixture tables only.
2. **T-3357** (M, parallel with 1): the generator, the definitions of the first three functions, the sampler, `verify`,
   the fit lab's refusal; mpmath added to tools/.
3. **T-3379** (S, after 1): `exempt.tsv` and the missing-table count, protected paths (`exempt.tsv`,
   `math-over-excel.tsv`, committed batch files), the known-failure line rules, ci_changes routing.
4. **T-3358** (M, after 1 and 2): definitions and `s1` tables for decision 0188's 18 functions, filled, failing points
   listed with owners, the R3 points marked and cited; files the slices for the remaining functions by family
   (distributions, legacy statistical names, elementary math and trig, financial annuities, bonds and depreciation,
   engineering, array statistics), each S or M.
5. **T-3380** (M, after 1 and 2): math sidecars for eval cells and the probes-numeric fill.
6. **T-3359** reads `envelope`, the R3 cell list and `eval/runs/accuracy.json` for Gate B, R1(c) and the Gate A line of
   the report; until T-3380 lands it counts every visibly-off numeric cell as outside the envelope.

Nothing changes for the eval gate or its baselines at any step. Until T-3356 lands, the report's Gate A line reads `n/a`.
