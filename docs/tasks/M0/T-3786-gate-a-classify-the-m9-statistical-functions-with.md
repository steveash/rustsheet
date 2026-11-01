---
id: T-3786
title: "Gate A: classify the M9 statistical functions with no table or exemption, rule on the legacy names and the sample-array functions, and file their table tasks"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3529]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Every in-scope numeric M9 statistical function that Gate A counts as missing ends up in one of two states. Either it
has an `exempt.tsv` line backed by a decision record, or an open task names it and will build its table. The legacy
compatibility names and the functions over sample arrays get a ruling on how a table drives them.

## Context
- Decision 0228 (T-3551) classified M5, M6 and M7. Its Consequences say M9's other functions "need their own
  classification task once the M9 table slices settle the solver and distribution patterns". No such task exists on
  main or on any pending branch. The patterns are now set by T-3358 (Student t and F), T-3553 (discrete), T-3535 and
  T-3586 (solvers, array literals), and T-3529 (gamma and beta, in flight, hence the dependency).
- On main (fafb6c6f) the M9 statistical functions with neither a table nor an exemption, leaving out those an open
  slice already names (T-3529; T-3783, T-3785 from this filing), are these 115:
  AVEDEV, AVERAGEA, AVERAGEIF, AVERAGEIFS, BETADIST, BETAINV, BINOM.DIST.RANGE, BINOM.INV, BINOMDIST, CHIDIST,
  CHIINV, CHISQ.DIST, CHISQ.DIST.RT, CHISQ.TEST, CHITEST, CONFIDENCE, CONFIDENCE.NORM, CONFIDENCE.T, CORREL, COVAR,
  COVARIANCE.P, COVARIANCE.S, CRITBINOM, DEVSQ, EXPONDIST, F.DIST.RT, F.INV.RT, F.TEST, FDIST, FINV, FORECAST,
  FORECAST.ETS, FORECAST.ETS.CONFINT, FORECAST.ETS.SEASONALITY, FORECAST.ETS.STAT, FORECAST.LINEAR, FREQUENCY, FTEST,
  GAMMA, GAMMA.DIST, GAMMADIST, GAMMAINV, GAMMALN, GAMMALN.PRECISE, GEOMEAN, GROWTH, HARMEAN, HYPGEOM.DIST,
  HYPGEOMDIST, INTERCEPT, KURT, LARGE, LINEST, LOGEST, LOGINV, LOGNORMDIST, MAXA, MAXIFS, MEDIAN, MINA, MINIFS, MODE,
  MODE.MULT, MODE.SNGL, NEGBINOMDIST, NORMDIST, NORMINV, NORMSDIST, NORMSINV, PEARSON, PERCENTILE, PERCENTILE.EXC,
  PERCENTILE.INC, PERCENTRANK, PERCENTRANK.EXC, PERCENTRANK.INC, PERMUT, PERMUTATIONA, POISSON, PROB, QUARTILE,
  QUARTILE.EXC, QUARTILE.INC, RANK, RANK.AVG, RANK.EQ, RSQ, SKEW, SKEW.P, SLOPE, SMALL, STANDARDIZE, STDEV, STDEV.P,
  STDEV.S, STDEVA, STDEVP, STDEVPA, STEYX, T.DIST.2T, T.DIST.RT, T.TEST, TDIST, TREND, TRIMMEAN, TTEST, VAR, VAR.P,
  VAR.S, VARA, VARP, VARPA, WEIBULL, Z.TEST, ZTEST.
  Recount before filing (`rustsheet-eval accuracy`, `eval/catalog/functions.yaml`): merges in the meantime may change
  the list.
- Questions that need judgment:
  - **Legacy names** (BETADIST, BINOMDIST, CHIDIST, CHIINV, CRITBINOM, EXPONDIST, FDIST, FINV, GAMMADIST, GAMMAINV,
    HYPGEOMDIST, LOGINV, LOGNORMDIST, NEGBINOMDIST, NORMDIST, NORMINV, NORMSDIST, NORMSINV, POISSON, TDIST, WEIBULL,
    COVAR, FTEST, TTEST, ZTEST, CHITEST, PERCENTILE, QUARTILE, RANK, MODE, STDEV, STDEVP, VAR, VARP). T-3358 gave TINV
    its own table, and statistics.md "Legacy distribution names sweep" (T-1048) records where a legacy name differs
    from its modern twin (FINV(0) is `#NUM!` where F.INV(0) is 0; TDIST's tails argument). Decide between a table per
    name and a shared definition with the name's own domain. The `exempt.tsv` classes are only `exact` and
    `composite`, so an "alias of" class would amend SPEC-0029 "Gate A holds" and the reader (`accuracy/lists.rs`). If
    you choose that, the spec amendment is in this PR and the reader change is filed as its own task.
  - **Sample-array functions** (the moments, the order statistics, percentiles and ranks, the regressions, the tests,
    the *IFS and *A aggregates). T-3586 added one-row array literals. Decide which a one-row literal drives (VAR.S,
    SKEW, PERCENTILE.INC with k), which need two arrays (CORREL, SLOPE, T.TEST; check that the generator writes two
    literals), and which are `composite` because they take criteria ranges or references (AVERAGEIF(S), MAXIFS,
    MINIFS) or return arrays (FREQUENCY, LINEST, LOGEST, GROWTH, TREND, MODE.MULT). For an array result, say how a
    one-cell point would read it (INDEX), as decision 0228 did.
  - **`exact` versus a kernel**: LARGE, SMALL, MEDIAN, RANK.EQ, MAXA, MINA, MODE.SNGL and PERMUT pick or count, but
    some round, such as MEDIAN's mean of two or PERMUT's product. Mark a function `exact` only where a semantics entry
    shows there is no rounding question.
  - **FORECAST.ETS and its siblings**: an iterative AAA-ETS fit. Decide whether a table can say anything (no closed
    math) or whether they are `composite`, citing the eval suites that drive them.
- `exempt.tsv` is protected (decision 0199). Every line added needs the decision record (`make new-decision`). T-3787
  also appends to it for the financial, engineering and database functions; whichever lands second rebases.
- Group the functions that need a table into table tasks of at most about six functions each (size M), by family.
  Create each with `make new-task MILESTONE=M0 LANE=harness`, depending on the slice whose definitions it reuses
  (T-3529 for gamma and beta, T-3783 for normal).
- Why opus: an amendment to SPEC-0029's classes may be needed, and each exemption is a judgment against the evidence.

## Acceptance criteria
- [ ] A decision record classifies each function of the recounted list as `exact`, `composite`, the legacy-name
      ruling, or "needs a table (T-XXXX)", with a cited reason for each exemption.
- [ ] `eval/accuracy/exempt.tsv` gains the exempt lines. `rustsheet-eval accuracy` reports `missing tables` lower by
      their number, with no stale exemption.
- [ ] Every function classified "needs a table" is named in the title or Goal of an open task filed by this PR, and
      `python3 scripts/tasks.py validate` passes.
- [ ] If the legacy-name ruling changes SPEC-0029, the spec note is in this PR and the reader change is filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Building any table: the filed tasks do that.
- The financial, engineering and database functions (T-3787).
