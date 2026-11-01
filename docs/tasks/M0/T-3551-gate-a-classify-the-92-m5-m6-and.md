---
id: T-3551
title: "Gate A: classify the 92 M5, M6 and M7 functions with no table or exemption, rule on the volatile ones, and file their table tasks"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3550]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
Every in-scope numeric function of M5, M6 and M7 that Gate A counts as missing ends up in one of two states:
- it has an `exempt.tsv` line backed by a decision record;
- or an open task names it and will build its table.

The volatile functions, which neither exemption class fits, get a ruling.

## Context
- `rustsheet-eval accuracy` on main (8ee20308) prints `missing tables 323`: M2 10, M5 74, M6 16, M7 2 and M9 221
  (`eval/catalog/scope.json`, `scope: in` and `returns: number`, minus `eval/accuracy/exempt.tsv`). Open tasks cover
  M2 (T-3555, T-3549, T-3550) and decision 0188's 18 M9 functions (T-3358, T-3529, T-3535, T-3543). No task covers
  the 92 functions of M5, M6 and M7, so Gate A, a Phase 1 exit gate (decision 0188 R1(a)), cannot hold.
- The 92 functions:
  - M5: ACOS, ACOSH, ACOT, ACOTH, AGGREGATE, ARABIC, ASIN, ASINH, ATAN, ATAN2, ATANH, CEILING, CEILING.MATH,
    CEILING.PRECISE, COMBIN, COMBINA, COS, COSH, COT, COTH, CSC, CSCH, DECIMAL, DEGREES, ECMA.CEILING, EVEN, EXP,
    FACT, FACTDOUBLE, FLOOR, FLOOR.MATH, FLOOR.PRECISE, GCD, INT, ISO.CEILING, LCM, LN, LOG, LOG10, MDETERM,
    MINVERSE, MMULT, MOD, MROUND, MULTINOMIAL, MUNIT, N, NUMBERVALUE, ODD, PERCENTOF, PI, POWER, PRODUCT, QUOTIENT,
    RADIANS, RAND, RANDBETWEEN, SEC, SECH, SERIESSUM, SIN, SINH, SQRT, SQRTPI, SUBTOTAL, SUMPRODUCT, SUMSQ,
    SUMX2MY2, SUMX2PY2, SUMXMY2, TAN, TANH, TRUNC, VALUE.
  - M6: DATEDIF, DATEVALUE, DAYS, DAYS360, EDATE, EOMONTH, HOUR, MINUTE, NETWORKDAYS, NETWORKDAYS.INTL, SECOND,
    TIME, TIMEVALUE, WORKDAY, WORKDAY.INTL, YEARFRAC.
  - M7: RANDARRAY, SEQUENCE.
  Recount before filing: tasks merged in the meantime may have changed the list.
- Questions that need judgment:
  - **Volatile functions** (RAND, RANDBETWEEN, RANDARRAY). They have no reproducible value, so a table has nothing to
    compare against, and neither `exact` nor `composite` describes them. The options:
    - a third class, which amends SPEC-0029 "Gate A holds" and the `exempt.tsv` reader (`accuracy/lists.rs`);
    - `composite`, naming the eval coverage that checks their range and type;
    - leaving them out of scope with a catalog change.
    Pick one in the decision record. A reader change goes in a follow-up task if it does not fit here.
  - **`exact` versus a kernel.** Some functions have an integer result but a floating-point input stage that Excel
    rounds in its own way. For example, HOUR, MINUTE and SECOND round the fractional day (dates.md), and EVEN, ODD
    and INT act on huge or near-integer doubles. Mark them `exact` only when a semantics entry shows there is no
    tolerance question. Otherwise they need a table.
  - **Array-shaped functions** (MDETERM, MINVERSE, MMULT, MUNIT, SUMPRODUCT, SUMX2MY2, SUMX2PY2, SUMXMY2,
    SERIESSUM, SEQUENCE). Decide whether an array-literal table can drive them, as T-3535 plans for IRR's flows, or
    whether they are `composite`. For a function whose result is an array, note how the table's one-cell point reads
    it (e.g. through INDEX).
  - **Reference-only functions** (AGGREGATE functions 1–13, SUBTOTAL: numbers.md "AGGREGATE", "take only
    references") are `composite`.
  - **Text parsers** (VALUE, NUMBERVALUE, DATEVALUE, TIMEVALUE, ARABIC, DECIMAL, N). They take text, not numbers, so
    a math table over numeric literals does not fit them. Decide which have a numeric kernel worth a table (for
    example TIMEVALUE's fraction of a day).
- `exempt.tsv` is protected (decision 0199). Every line added needs the decision record. A line naming a function
  that has a table fails the check, so check the open table tasks before exempting.
- Group the functions that need a table into table tasks of at most about six functions each (an M task, like the
  T-3358 slices), by family: trig, hyperbolic, rounding, exp/log/power, combinatorics, date fractions. Create each
  with `make new-task`, and depend on T-3555 for the rounding family so the decimal-math rule is reused.

## Acceptance criteria
- [x] A decision record classifies each of the 92 functions (or the recounted list) as `exact`, `composite`, the
      volatile ruling, or "needs a table (T-XXXX)", with a cited reason for each exemption.
- [x] `eval/accuracy/exempt.tsv` gains the exempt lines. `rustsheet-eval accuracy` reports `missing tables` lower
      by their number, with no stale exemption.
- [x] Every function classified "needs a table" is named in the title or Goal of an open task filed by this PR, and
      `python3 scripts/tasks.py validate` passes.
- [x] If the volatile ruling changes SPEC-0029 or the reader, the spec note is in this PR, and the reader change is
      either in this PR or filed as a task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Building any table (the filed tasks do that).
- M9's 203 functions outside decision 0188's 18 (a separate classification task, after the T-3358 slices land and
  settle the solver and distribution patterns).

## Outcome
Done (2026-11-01).
- **Recount:** the 92 functions of M5 (74), M6 (16) and M7 (2) match the task's list exactly.
- **Decision 0228** classifies each of them:
  - **`exact`, 16:** the whole-day date functions (DATEDIF, DAYS, DAYS360, EDATE, EOMONTH, NETWORKDAYS(.INTL),
    WORKDAY(.INTL)), plus DATEVALUE, ARABIC, GCD, LCM, MUNIT, N and PI.
  - **`composite`, 12:** SUBTOTAL (references only); VALUE, NUMBERVALUE, TIMEVALUE and DECIMAL (text);
    MDETERM, MINVERSE and MMULT (2-D matrices); SEQUENCE (an array result); and the volatile RAND, RANDARRAY and
    RANDBETWEEN.
  - **Needs a table, 64:** twelve tasks, T-3668 to T-3678 and T-3693 (AGGREGATE's array forms 14-19).
- **Volatile ruling:** `composite`, driven by an invariant-cell probe, the same evidence decision 0225 (T-3631) uses for
  Gate B. So there is no third class, no SPEC-0029 change and no reader change. Decision 0225 is not on main yet, so
  T-3685 adds the three lines once T-3631 lands.
- **`eval/accuracy/exempt.tsv`:** +25 lines. Each `exact` reason cites a semantics entry or gen-m6 cells; each
  `composite` reason names its eval suites. `rustsheet-eval accuracy`: `missing tables` 305 to 280, 0 failing, no stale
  exemption.
- **Follow-ups:** T-3685 (the volatile lines, after T-3631) and T-3686 (2-D array literals, so MDETERM, MINVERSE and
  MMULT get tables in place of their exemptions).
- **Table tasks** (M0, harness, opus, size M):
  - T-3668: trig.
  - T-3669: inverse trig plus DEGREES and RADIANS.
  - T-3670: hyperbolic.
  - T-3671: inverse hyperbolic.
  - T-3672: exp, log, power and square roots.
  - T-3673 and T-3674: rounding, after T-3555.
  - T-3675: combinatorics, QUOTIENT and MOD.
  - T-3676: PRODUCT, SUMSQ, SERIESSUM and PERCENTOF, after T-3549.
  - T-3677: the SUMPRODUCT family.
  - T-3678: TIME, HOUR, MINUTE, SECOND and YEARFRAC.
  - T-3693: AGGREGATE over functions 14-19.

  Each task names the semantics entries its definitions must read.
