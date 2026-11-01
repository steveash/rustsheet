---
id: T-3128
title: "Probe the M9 legacy distribution argument kinds no Excel cell covers yet: empty text and an error in the 24 compatibility-name functions (NORMDIST, TDIST, BETADIST, ...)"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Every value parameter of the M9 compatibility-name distribution functions below (NORMDIST, NORMINV, NORMSDIST, NORMSINV, TDIST, TINV, BETADIST, BETAINV, BINOMDIST, CRITBINOM, CHIDIST, CHIINV, FDIST, FINV, GAMMADIST, GAMMAINV, HYPGEOMDIST, LOGINV, LOGNORMDIST, NEGBINOMDIST, POISSON, WEIBULL, EXPONDIST, CONFIDENCE) has at least one Excel-backed cell for each argument kind the report's argument coverage expects.

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
The legacy names share kernels with the current names (T-3127) but have their own argument lists and checks (TDIST's `tails`; NORMDIST's and GAMMADIST's required `cumulative`; statistics.md's lifting rules note that TDIST checks x and `tails` per element), so probe them separately. Pick other arguments where the kernel is bit-exact today.
These parameters have no Excel cell of the listed kinds in any suite (highest catalog `prior` first, as
`FUNCTION.parameter: kinds`):
- BINOMDIST.number_s: empty text, an error; BINOMDIST.trials: empty text, an error; BINOMDIST.probability_s: empty text, an error; BINOMDIST.cumulative: an error.
- NORMDIST.x: empty text, an error; NORMDIST.mean: empty text, an error; NORMDIST.standard_dev: empty text, an error; NORMDIST.cumulative: an error.
- NORMINV.probability: empty text, an error; NORMINV.mean: empty text, an error; NORMINV.standard_dev: empty text, an error.
- NORMSDIST.z: empty text, an error.
- NORMSINV.probability: empty text, an error.
- BETADIST.x: empty text, an error; BETADIST.alpha: empty text, an error; BETADIST.beta: empty text, an error; BETADIST.A: empty text, an error; BETADIST.B: empty text, an error.
- BETAINV.probability: empty text, an error; BETAINV.alpha: empty text, an error; BETAINV.beta: empty text, an error; BETAINV.A: empty text, an error; BETAINV.B: numeric text, empty text, an error.
- CHIDIST.x: empty text, an error; CHIDIST.deg_freedom: empty text, an error.
- CHIINV.probability: empty text, an error; CHIINV.deg_freedom: empty text, an error.
- CONFIDENCE.alpha: empty text, an error; CONFIDENCE.standard_dev: numeric text, empty text, an error; CONFIDENCE.size: empty text, an error.
- CRITBINOM.trials: empty text, an error; CRITBINOM.probability_s: empty text, an error; CRITBINOM.alpha: empty text, an error.
- EXPONDIST.x: empty text, an error; EXPONDIST.lambda: numeric text, empty text, an error; EXPONDIST.cumulative: an error.
- FDIST.x: empty text, an error; FDIST.deg_freedom1: numeric text, empty text, an error; FDIST.deg_freedom2: numeric text, empty text, an error.
- FINV.probability: empty text, an error; FINV.deg_freedom1: numeric text, text, empty text, an error; FINV.deg_freedom2: numeric text, empty text, an error.
- GAMMADIST.x: empty text, an error; GAMMADIST.alpha: empty text, an error; GAMMADIST.beta: empty text, an error; GAMMADIST.cumulative: an error.
- GAMMAINV.probability: empty text, an error; GAMMAINV.alpha: empty text, an error; GAMMAINV.beta: empty text, an error.
- HYPGEOMDIST.sample_s: empty text, an error; HYPGEOMDIST.number_sample: numeric text, empty text, an error; HYPGEOMDIST.population_s: numeric text, empty text, an error; HYPGEOMDIST.number_pop: numeric text, empty text, an error.
- LOGINV.probability: empty text, an error; LOGINV.mean: empty text, an error; LOGINV.standard_dev: empty text, an error.
- LOGNORMDIST.x: empty text, an error; LOGNORMDIST.mean: empty text, an error; LOGNORMDIST.standard_dev: empty text, an error.
- NEGBINOMDIST.number_f: empty text, an error; NEGBINOMDIST.number_s: numeric text, empty text, an error; NEGBINOMDIST.probability_s: empty text, an error.
- POISSON.x: empty text, an error; POISSON.mean: empty text, an error; POISSON.cumulative: an error.
- TDIST.x: empty text, an error; TDIST.deg_freedom: empty text, an error; TDIST.tails: numeric text, empty text, an error.
- TINV.probability: empty text, an error; TINV.deg_freedom: numeric text, empty text, an error.
- WEIBULL.x: empty text, an error; WEIBULL.alpha: empty text, an error; WEIBULL.beta: empty text, an error; WEIBULL.cumulative: an error.
Write the probe case probes/m9-stat-legacy-dist-argument-kind-gaps (excel-probe skill, excel-graph, tag M9); a short generator script beside the existing
probe generators is fine when the list is long. Put each kind in two forms where they differ: a literal (`""`, `#N/A`,
`#DIV/0!`, `"2"`, `TRUE`, `0`) and a single-cell reference to a cell holding it (a formula cell `=""`, `=1/0` or `=NA()`, a typed `TRUE`
or `"2"`, an empty cell for the blank kind). Give every other
argument an ordinary valid value, so the probed argument decides the result, and wrap each error-valued cell in
`ERROR.TYPE` too so the error kind is scored.

## Acceptance criteria
- [ ] probes/m9-stat-legacy-dist-argument-kind-gaps is committed with oracle values (excel-graph) and covers every parameter/kind pair listed above; after
      `make eval`, report.json `argument_coverage` shows `covered == expected` for the listed parameters, or the
      Outcome names each pair still short and why.
- [ ] Every cell where ours differs from Excel is either fixed in this PR (a one-line coercion fix in the function's own
      module) or listed in a new functions-* task (`campaign: semantics`, with a `## Visible-parity stage` section) filed
      with `make new-task`, naming the case and cells; the new known failures are recorded with `make eval-baseline` and
      each has that owner (`make unowned-off` prints none of them).
- [ ] Any new Excel rule is recorded in docs/excel-semantics/statistics.md (or coercion.md), citing the probe cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The current names (T-3127).
- Kernel accuracy (the distribution kernel tasks).
- Pairwise argument orders (which of two bad arguments wins): T-2611 and the check-order sections of financial.md own
  those for the financial functions; file others as follow-ups.
- Generator pools (T-2866, done).
