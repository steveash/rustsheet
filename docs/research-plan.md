# RustSheet research plan & workstreams

**Goal:** a headless, high-performance Rust spreadsheet engine whose results are identical to
current Microsoft 365 Excel, built by autonomous agents in a closed loop with Excel as the oracle.

This document lists the research **workstreams** (WS), what each must find out or build, how
success is measured, and where results land. Milestones (`milestones.md`) sequence the work;
tasks (`tasks/`) are the executable units. Background research produced at inception is in
`docs/research/`.

## Guiding strategy

1. **Measure before building.** The eval harness and oracle pipeline come first (M0). Every
   later change is judged by "known failures went down, nothing regressed".
2. **Free oracle data first, synthetic second, live Excel third.** Excel-saved xlsx files carry
   exact cached results, so Excel-authored corpora are oracle data at zero cost. Synthetic
   workbooks fill coverage gaps but need a live Excel run. Live Excel runs are batched, cached by
   content hash and committed as golden sidecars, so day-to-day agent work is fully offline.
3. **Strict scoring.** A cell passes only if it is bit-identical to Excel. Near misses are
   classified (ULP / close / mismatch / unsupported) to separate precision work from semantics work.
4. **Fork, then reshape.** Start from IronCalc (496 functions, 97.33% bit-exact on its own
   Excel-authored workbooks at fork time) and replace the architecture underneath: dependency
   graph, lazy evaluation, compact storage, transactional API.
5. **Knowledge compounds.** Every discovered Excel quirk is recorded once
   (`docs/excel-semantics/`) with oracle evidence and becomes a regression case.

## Workstreams

### WS1 — Excel oracle & automation
*Question:* how do we get Excel's answer for any workbook automatically, reliably and cheaply?
- **Done (M0):**
  - Backend interface (`tools/src/rustsheet_tools/oracle/`) with three backends: Graph (Excel
    for the web), Windows COM (desktop) and LibreOffice (approximate, triage only).
  - Results cached by content hash and written as `*.oracle.json` sidecars.
  - Every backend returns a recalculated xlsx parsed by the single Rust snapshot reader, which
    keeps full 17-digit precision.
- **Open research:**
  1. Does Graph return exact doubles? Our design sidesteps this by downloading the recalculated file.
  2. Graph session and persistence latency, and throttling under batch load.
  3. Does app-only auth work for workbook endpoints?
  4. Calibration: do Graph and desktop agree on the whole IronCalc corpus?
  5. Excel version drift: record the build with every oracle result, and re-run calibration
     monthly with the channel update.
  6. A Mac backend (AppleScript), if the owner prefers a Mac.
- **Deliverables:** `docs/architecture/oracle.md` (owner setup checklist); a calibration report
  in `docs/research/oracle-calibration.md` (T-0004); an oracle CI workflow.
- **Metric:** oracle throughput (workbooks/hour), failure rate, agreement between backends.

### WS2 — Test suites & corpora
*Question:* which existing artifacts give the most Excel ground truth per unit of effort?
- **Ranked plan** (from `docs/research/test-suites-and-oracle.md`):
  1. IronCalc `calc_tests`: vendored; 207 workbooks, 71k cells.
  2. Apache POI `test-data/spreadsheet`: Apache-2.0, so vendor the xlsx subset. About 31k formula cells. The `.xls` part needs a BIFF reader or an oracle re-save.
  3. xlcalculator and calipers: MIT, small.
  4. witanlabs xlsx-corpus-bench: SpreadsheetBench and FUSE, fetched not vendored, because CC-BY-SA/CC-BY.
  5. Microsoft support-page function examples: scraped locally, then run through the oracle.
  6. LibreOffice function tests: MPL, fetched. Translate them and use them as *inputs* only.
  7. Enron/VEnron: mainly performance and incremental recalc (old Excel caches).
- **Build:** a fetcher per external suite (`suite.toml` → download + sha256 pin → cache), plus the
  provenance gate for cached values (Excel-authored, not manual-calc, not stale).
- **Metric:** total oracle-backed cells per function and per milestone scope (coverage matrix).

### WS3 — Synthetic workbook generation
*Question:* how do we generate workbooks that find real divergences with few oracle calls?
- **Design:** `docs/architecture/synthetic-generator.md`. It covers the catalog-driven,
  type-directed generator; per-type edge pools; realistic data tables and scenario templates;
  Thompson-sampling bandit weighting toward failing or under-covered functions; metamorphic
  relations that need no oracle; test-case reduction; and canary cells.
- **Phases:**
  - **P0 (done):** catalog plus seeded, milestone-scoped generator (`rustsheet-gen`).
  - **P1:** measure Excel's numeric rules first (close-to-zero snapping, comparison tolerance, 15-digit literal parsing).
  - **P2:** catalog for all functions, bootstrapped from Microsoft's function list.
  - **P3:** metamorphic runner in Rust (no oracle).
  - **P4:** adaptive weighting from eval results.
  - **P5:** reducer (shrink a failing workbook to a minimal repro and file a task automatically).
  - **P6:** scenario templates.
  - **P7:** subexpression probing, which amortizes oracle calls.
- **Metric:** new distinct diff signatures per 1,000 oracle cells (it should stay high), plus the coverage matrix.

### WS4 — Excel semantics knowledge base
*Question:* what are Excel's exact rules where documentation is silent?
- **Topics:**
  - floating point (15-digit display, near-zero snapping in `+`/`-`, comparisons)
  - coercion (text↔number↔bool in operators, aggregates, criteria)
  - blank vs `""`
  - which error wins when several arguments are errors
  - the 1900 leap-year bug and serial ranges
  - implicit intersection vs dynamic arrays
  - wildcard and criteria grammar
  - case sensitivity and collation
  - iteration limits of solvers (IRR/RATE/YIELD)
  - circular references
- **Method:** the `excel-probe` skill: a probe case, then the oracle, then an entry in
  `docs/excel-semantics/<area>.md`, then a regression case.
- **Metric:** probes answered; semantics entries with evidence.

### WS5 — Engine architecture (calc core)
*Question:* what architecture gives Excel parity *and* high scale and low latency?
- **Facts at fork** (`docs/research/ironcalc-assessment.md`):
  - No dependency graph: every `evaluate()` recomputes every cell, about 0.49 s per 1M formulas even with no change.
  - Pull-based recursive evaluator, which suits laziness.
  - About 400 B per sparse cell; strings stored twice.
  - Formulas deduplicated per sheet as R1C1 text (good).
  - Row/column insert re-types and re-parses formulas (about 3.7 µs per cell).
- **Research and build:**
  1. A persistent dependency graph with range-interval indexing.
  2. Dirty propagation, a volatile set, and dynamic dependencies (INDIRECT/OFFSET).
  3. Lazy compute-on-read (`OnDemand` mode).
  4. Excel-compatible circular references and iterative calculation.
  5. Formula families (shared R1C1 formulas evaluated as a unit).
  6. Chunked copy-on-write columnar storage with interned strings and styles (cheap snapshots).
  7. Structural edits via reference maps instead of re-parsing.
  8. Parallel recalculation by topological level.
- **Prior art to study:** Formualizer (Rust, incremental), HyperFormula (graph and range
  vertices), GRID's vertex ids, LibreOffice's formula groups, and Excel's own documented "smart
  recalculation" and calculation chain.
- **Metric:** incremental recalc latency after a 1-cell edit on 1M-formula models; full recalc
  throughput; bytes per cell; insert-row cost. Benchmarks in `benches/` (T-0030 series).

### WS6 — Public API & bindings
*Question:* what API should a headless engine expose (GRID-inspired, but better)?
- **Proposal:** `docs/architecture/api-design.md`. It covers transactions returning a `ChangeSet`,
  typed errors, stable `SheetId`, `Snapshot`/`Scenario`/`Calculator`, `eval` with bindings,
  precedents and dependents, and a `RefMap` from structural edits.
- **Bindings:** Python first (PyO3, bulk/numpy-friendly, releases the GIL). Java later, via
  UniFFI or Panama FFM over a flat FFI facade.
- **Metric:** API coverage of the use-case catalog (load / edit / recalc / what-if / introspect /
  save); Python binding parity tests.

### WS7 — Function implementation & numerical parity
*Question:* which algorithms does Excel use, so that results match to the last ULP?
- **Approach:**
  1. Gap reports per function, triaged by root cause.
  2. For numeric families (distributions, Bessel, financial solvers), study the reference
     algorithms (Cephes, AS-series, fdlibm) and Excel's documented accuracy improvements
     (Excel 2010 statistical functions).
  3. Fit against dense oracle sweeps.
- **Known gap at fork:** 1,909 non-exact cells concentrated in BESSELJ/Y, IRR/NPV/PV/MIRR,
  statistical distributions, and CHAR. There are also missing functions (AGGREGATE,
  GROUPBY/PIVOTBY, IMAGE, CUBE*, …).
- **Metric:** bit-exact cell rate per function, weighted by real-world usage.

### WS8 — Performance & memory benchmarking
- **Benchmark suite:**
  - synthetic scale models: 10⁴–10⁷ cells, chains, fan-in/out, whole-column references, VLOOKUP-heavy, SUMIFS-heavy
  - real large workbooks: Enron/FUSE top-N by formula count
  - edit scenarios: single-cell edit latency, insert rows, bulk paste
- **Baselines:** Excel desktop timings (COM backend), LibreOffice, Formualizer, upstream IronCalc.
- **CI:** perf-regression gate (> 5% slower on the tracked benchmarks fails) from M3.

### WS9 — Agentic process & code health
*Question:* how do we keep weeks of autonomous development convergent and clean?
- **Built in M0:**
  - lanes, task files, the work-loop skill
  - five reviewer agents, plus a planner and a gap triager
  - ratchets: eval baseline, lint debt, duplication, file size, governance
  - CI
- **Research:** measure agent throughput (merged tasks per day), revert rate, reviewer
  false-positive and false-negative rates (spot audits), and token cost per task. Tune lane
  concurrency and reviewer prompts.
- **Metric:** weekly health report in `docs/status.md`.

### WS10 — Upstream watch
IronCalc is active upstream (commits daily). We **port** relevant fixes rather than merge them
(`scripts/upstream-log.sh`, `UPSTREAM.md`). The weekly task triages new upstream commits.

## Cross-cutting risks

| Risk | Mitigation |
|---|---|
| No Excel access (Graph requires a work/school tenant) | IronCalc and POI cached values cover M0–M2. Use the oracle backends in the order: Graph, Windows VM, Mac. Probes stay pending, not guessed. |
| Excel results change between builds | Record the build per oracle result. Re-run calibration monthly and keep a changelog of Excel behavior changes. |
| Agents game the metrics (editing baselines, deleting tests) | Generated-only baselines, the governance check, reviewer rules, and decision records for protected paths. |
| Architecture rot from many small PRs | Architecture reviewer, specs before big changes, and the duplication, file-size and lint ratchets. |
| Oracle bottleneck | Cached sidecars, batch packing (thousands of probe cells per workbook), and metamorphic relations that need no oracle. |
| Forked code quality debt | Lint-debt and duplication ratchets, plus a `quality` lane that burns them down. |
