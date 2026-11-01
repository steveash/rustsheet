---
id: T-3798
title: "Gate A: classify the 53 M9 distribution, quantile and special functions outside decision 0188's 18 with no table or exemption, and file their table tasks"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Every in-scope numeric M9 distribution, quantile or special function outside decision 0188's 18 that Gate A counts as
missing ends up in one of two states:
- it has an `eval/accuracy/exempt.tsv` line backed by a decision record;
- or an open task names it in its title or Goal and will build its table.

## Context
- Gate A holds only with no missing table (SPEC-0029 "Gate A holds", decision 0188 R1(a)). Decision 0228 classified
  M5, M6 and M7. Its Consequences, and T-3551's Out of scope, leave M9's functions outside decision 0188's 18 for a
  separate classification task, filed once the distribution pattern settled. It has for the continuous t and F
  families and the discrete ones: T-3358 and T-3553 are done (`accuracy/defs/student_t.py`, `fisher.py`,
  `discrete.py`). T-3529 (CHISQ.INV, CHISQ.INV.RT, GAMMA.INV, BETA.DIST, BETA.INV) is still open.
- `rustsheet-eval accuracy` on main (56917e82c) prints `missing tables 271`. 53 of them are M9 statistical functions of
  this family that no Gate A task names:
  BETADIST, BETAINV, BINOM.DIST.RANGE, BINOM.INV, BINOMDIST, CHIDIST, CHIINV, CHISQ.DIST, CHISQ.DIST.RT, CONFIDENCE,
  CONFIDENCE.NORM, CONFIDENCE.T, CRITBINOM, EXPON.DIST, EXPONDIST, F.DIST.RT, F.INV.RT, FDIST, FINV, FISHER,
  FISHERINV, GAMMA, GAMMA.DIST, GAMMADIST, GAMMAINV, GAMMALN, GAMMALN.PRECISE, GAUSS, HYPGEOM.DIST, HYPGEOMDIST,
  LOGINV, LOGNORM.DIST, LOGNORM.INV, LOGNORMDIST, NEGBINOMDIST, NORM.DIST, NORM.INV, NORM.S.DIST, NORM.S.INV,
  NORMDIST, NORMINV, NORMSDIST, NORMSINV, PERMUT, PERMUTATIONA, PHI, POISSON, STANDARDIZE, T.DIST.2T, T.DIST.RT,
  TDIST, WEIBULL, WEIBULL.DIST.
  Recount before filing, because tasks merged in the meantime may change the list.
- Questions that need judgment, each answered in the decision record:
  - **Legacy aliases** (BETADIST, BINOMDIST, CHIDIST, CHIINV, EXPONDIST, FDIST, FINV, GAMMADIST, GAMMAINV,
    HYPGEOMDIST, LOGINV, LOGNORMDIST, NEGBINOMDIST, NORMDIST, NORMINV, NORMSDIST, NORMSINV, POISSON, TDIST, WEIBULL,
    CRITBINOM). SPEC-0029 knows only a table or an `exact`/`composite` line. A third "alias of X" class is a
    SPEC-0029 and `accuracy/lists.rs` change, which needs a spec note and its own reader task. The default is a table
    per alias that reuses the modern function's Definition (TINV already has its own table beside T.INV.2T). Some
    aliases differ in their arguments (TDIST's tails, legacy CHIDIST the upper tail, FDIST and FINV right-tailed,
    LOGNORMDIST the cdf only), so a shared Definition must take those into account.
  - **Gamma and beta aliases depend on T-3529.** Table tasks for BETADIST, BETAINV, GAMMADIST, GAMMAINV, CHIINV and
    GAMMA.DIST reuse its definitions, so they depend on T-3529.
  - **Special functions** (GAMMA, GAMMALN, GAMMALN.PRECISE). statistics.md "GAMMALN is Excel's own x87 routine" and
    "GAMMA and GAMMALN sweep": Excel's value is a valid target within 15 digits, so a table is still needed.
  - **Integer results with a floating-point stage** (BINOM.INV, CRITBINOM, PERMUT, PERMUTATIONA). They need a table
    unless a semantics entry shows no tolerance question, as decision 0228 required for FACT and COMBIN.
  - **Closed forms** (STANDARDIZE, FISHER, FISHERINV, CONFIDENCE.*, PHI, GAUSS, EXPON.DIST, WEIBULL.DIST). They need
    a table. Group them by shared kernel (normal, exponential/Weibull, Fisher).
- `exempt.tsv` is protected (decision 0199, T-3379): every line added needs the decision record. The three sibling
  classification tasks (T-3797, T-3799, T-3800) also add lines, so rebase before merging and keep the file sorted.
- Group the functions that need a table into table tasks of at most about seven functions each (M tasks, like
  T-3668 to T-3678), by kernel: normal and lognormal, chi-square, gamma and beta aliases (after T-3529), discrete,
  t and F aliases, exponential and Weibull, Fisher and confidence, gamma-function family. Create each with
  `make new-task MILESTONE=M0 LANE=harness`, `tier: opus`, `spec:` SPEC-0029.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) classifies each of the 53 functions (or the recounted list) as
      `exact`, `composite` or "needs a table (T-XXXX)", with a cited reason for each exemption, and rules on the
      legacy-alias question (a table per alias, or a third class with its spec note and reader task filed).
- [ ] `eval/accuracy/exempt.tsv` gains any exempt lines, sorted. `rustsheet-eval accuracy` reports `missing tables`
      lower by their number, with no stale or bad exemption.
- [ ] Every function classified "needs a table" is named in the title or Goal of an open task filed by this PR. The
      gamma and beta alias tasks depend on T-3529.
- [ ] `python3 scripts/tasks.py validate` passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Building any table or definition (the filed tasks do that).
- T-3529's five functions and the rest of decision 0188's 18 (T.DIST, T.INV, T.INV.2T, TINV, F.DIST, F.INV,
  BINOM.DIST, NEGBINOM.DIST, POISSON.DIST already have tables).
- The other M9 families: financial (T-3797), sample statistics (T-3799), engineering and database (T-3800).
- New R3 cases. Any region where Excel looks far from the math goes in the table task as a question for an owner
  ruling (decision 0188 R3).
