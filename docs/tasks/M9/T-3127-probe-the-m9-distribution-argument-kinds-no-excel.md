---
id: T-3127
title: "Probe the M9 distribution argument kinds no Excel cell covers yet: empty text and an error in the 43 current-name distribution functions"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of the M9 current-name distribution functions below (the dotted .DIST/.INV names, CONFIDENCE.NORM/T, GAMMA, GAMMALN, GAUSS, FISHER, PERMUT and PERMUTATIONA) has at least one Excel-backed cell for each argument kind the report's argument coverage expects.

## Context
Gap triage 2026-10-29 (`make eval` on 6793e919 with calipers, corpus-bench and poi fetched; report.json
`argument_coverage`, decision 0031 evidence gaps; decision 0094 asks for each argument mode to be covered). Every
visibly-off cell of that run has an open owner (`scripts/exit_owners.py --all-off`), and the evidence floor is met, so
the M9 argument kinds are the largest evidence gap left: 245 M9 functions have a value parameter with a kind no Excel
cell passes (M2 was closed by T-2861; M5 to M7 are T-2862, T-2863, T-2864, T-2865 and T-2871 to T-2878; M9 had none).
The kinds are those of `crates/rustsheet-eval/src/argument_coverage.rs` (`classify_value`): number, zero, negative,
fraction, numeric_text, text, empty_text, boolean, error, blank, range. A kind is seen only when an Excel-evidence cell
passes it as a literal or a single-cell reference: nested calls such as `NA()` are not classified (docs/progress.md
"Argument coverage"), so use literal errors (`#N/A`, `#DIV/0!`) and cells holding them. T-2866 taught the generator to
draw `""` and error literals, but at 3% per parameter the existing gen-m9 cases predate it and do not reach these pairs.
`array`-mode parameters are left out: their coverage is not classified by kind.
Most gaps are empty text in a direct numeric argument, plus an error in the cumulative flag or in a later shape argument. Pick other arguments where the kernel is bit-exact today (e.g. NORM.DIST(1,0,1,TRUE)), so a coercion difference is not confused with the kernel tasks' last-digit cells.
These parameters have no Excel cell of the listed kinds in any suite (highest catalog `prior` first, as
`FUNCTION.parameter: kinds`):
- BINOM.DIST.number_s: empty text; BINOM.DIST.trials: empty text; BINOM.DIST.probability_s: empty text.
- NORM.DIST.x: empty text; NORM.DIST.mean: empty text; NORM.DIST.standard_dev: empty text; NORM.DIST.cumulative: empty text, an error.
- NORM.INV.probability: empty text; NORM.INV.mean: empty text; NORM.INV.standard_dev: empty text.
- NORM.S.DIST.z: empty text.
- NORM.S.INV.probability: empty text.
- BETA.DIST.x: empty text; BETA.DIST.alpha: empty text; BETA.DIST.beta: empty text; BETA.DIST.cumulative: empty text; BETA.DIST.A: empty text; BETA.DIST.B: empty text.
- BETA.INV.probability: empty text, a boolean; BETA.INV.alpha: empty text; BETA.INV.beta: empty text; BETA.INV.A: empty text, a boolean; BETA.INV.B: empty text, a boolean.
- BINOM.DIST.RANGE.trials: empty text; BINOM.DIST.RANGE.probability_s: empty text; BINOM.DIST.RANGE.number_s: empty text; BINOM.DIST.RANGE.number_s2: empty text.
- BINOM.INV.trials: empty text; BINOM.INV.probability_s: empty text; BINOM.INV.alpha: empty text.
- CHISQ.DIST.x: empty text, an error; CHISQ.DIST.deg_freedom: numeric text, text, empty text, a boolean, an error; CHISQ.DIST.cumulative: an error.
- CHISQ.DIST.RT.x: empty text, an error; CHISQ.DIST.RT.deg_freedom: numeric text, empty text, an error.
- CHISQ.INV.probability: empty text, an error; CHISQ.INV.deg_freedom: empty text, an error.
- CHISQ.INV.RT.probability: empty text, an error; CHISQ.INV.RT.deg_freedom: empty text, an error.
- CONFIDENCE.NORM.alpha: empty text; CONFIDENCE.NORM.standard_dev: empty text; CONFIDENCE.NORM.size: empty text.
- CONFIDENCE.T.alpha: empty text; CONFIDENCE.T.standard_dev: empty text; CONFIDENCE.T.size: empty text.
- EXPON.DIST.x: empty text; EXPON.DIST.lambda: empty text.
- F.DIST.x: empty text, an error; F.DIST.deg_freedom1: empty text, an error; F.DIST.deg_freedom2: empty text, a boolean, an error; F.DIST.cumulative: an error.
- F.DIST.RT.x: empty text, an error; F.DIST.RT.deg_freedom1: empty text, an error; F.DIST.RT.deg_freedom2: empty text, an error.
- F.INV.probability: empty text, an error; F.INV.deg_freedom1: empty text, a boolean, an error; F.INV.deg_freedom2: empty text, an error.
- F.INV.RT.probability: empty text, an error; F.INV.RT.deg_freedom1: numeric text, empty text, an error; F.INV.RT.deg_freedom2: numeric text, empty text, an error.
- FISHER.x: empty text.
- FISHERINV.y: empty text.
- GAMMA.x: empty text.
- GAMMA.DIST.x: empty text; GAMMA.DIST.alpha: empty text; GAMMA.DIST.beta: empty text; GAMMA.DIST.cumulative: an error.
- GAMMA.INV.probability: empty text; GAMMA.INV.alpha: empty text; GAMMA.INV.beta: empty text.
- GAMMALN.x: empty text.
- GAMMALN.PRECISE.x: empty text.
- GAUSS.z: empty text.
- HYPGEOM.DIST.sample_s: empty text, an error; HYPGEOM.DIST.number_sample: numeric text, empty text, an error; HYPGEOM.DIST.population_s: numeric text, empty text, an error; HYPGEOM.DIST.number_pop: empty text, an error; HYPGEOM.DIST.cumulative: an error.
- LOGNORM.DIST.x: empty text; LOGNORM.DIST.mean: empty text; LOGNORM.DIST.standard_dev: empty text; LOGNORM.DIST.cumulative: an error.
- LOGNORM.INV.probability: empty text; LOGNORM.INV.mean: empty text; LOGNORM.INV.standard_dev: empty text.
- NEGBINOM.DIST.number_f: empty text; NEGBINOM.DIST.number_s: empty text; NEGBINOM.DIST.probability_s: empty text.
- PERMUT.number: empty text; PERMUT.number_chosen: empty text, an error.
- PERMUTATIONA.number: empty text; PERMUTATIONA.number_chosen: empty text, an error.
- POISSON.DIST.x: empty text; POISSON.DIST.mean: empty text.
- PROB.lower_limit: empty text, an error; PROB.upper_limit: empty text, an error.
- STANDARDIZE.x: empty text, an error; STANDARDIZE.mean: empty text, an error; STANDARDIZE.standard_dev: numeric text, empty text, an error.
- T.DIST.x: empty text; T.DIST.deg_freedom: text, empty text.
- T.DIST.2T.x: empty text; T.DIST.2T.deg_freedom: text, empty text.
- T.DIST.RT.x: empty text; T.DIST.RT.deg_freedom: text, empty text.
- T.INV.probability: empty text; T.INV.deg_freedom: empty text.
- T.INV.2T.probability: empty text; T.INV.2T.deg_freedom: text, empty text.
- WEIBULL.DIST.x: empty text, an error; WEIBULL.DIST.alpha: empty text, an error; WEIBULL.DIST.beta: empty text, an error; WEIBULL.DIST.cumulative: an error.
Write the probe case probes/m9-stat-dist-argument-kind-gaps (excel-probe skill, excel-graph, tag M9); a short generator script beside the existing
probe generators is fine when the list is long. Put each kind in two forms where they differ: a literal (`""`, `#N/A`,
`#DIV/0!`, `"2"`, `TRUE`, `0`) and a single-cell reference to a cell holding it (a formula cell `=""`, `=1/0` or `=NA()`, a typed `TRUE`
or `"2"`, an empty cell for the blank kind). Give every other
argument an ordinary valid value, so the probed argument decides the result, and wrap each error-valued cell in
`ERROR.TYPE` too so the error kind is scored.

## Acceptance criteria
- [ ] probes/m9-stat-dist-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed above; after
      `make eval`, report.json `argument_coverage` shows `covered == expected` for the listed parameters, or the
      Outcome names each pair still short and why.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's own
      module) or listed in a new functions-* task (`campaign: semantics`, with a `## Visible-parity stage` section) filed
      with `make new-task`, naming the case and cells; the new known failures are recorded with `make eval-baseline` and
      each has that owner (`make unowned-off` prints none of them).
- [ ] Any new Excel rule is recorded in docs/excel-semantics/statistics.md (or coercion.md), citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The compatibility names (T-3128) and the aggregates, tests and regression (T-3129).
- Kernel accuracy (T-2507 to T-2511, T-3034, T-3061 and the other distribution tasks).
- Pairwise argument orders (which of two bad arguments wins): T-2611 and the check-order sections of financial.md own
  those for the financial functions; file others as follow-ups.
- Generator pools (T-2866, done).
