# 0043 — Freeze M1 probe scope; everything else is M2 exit scope

- **Status:** accepted (2026-10-08, owner request; measurement in T-1146)
- **Amended by:** decision 0050 (2026-10-12). The owner answered the open question in item 2.
  Probes of later-milestone functions count toward their own milestone, not M2. M2 exit covers
  only the probe cases assigned M0, M1 or M2.
- **Context:**
  - The owner (2026-10-08): "Sure let's freeze m1 scope but we need all of these in m2."
  - M1 exit asked for the whole `probes` suite at 100% bit-exact, apart from accepted deviations
    (decision 0036). `probes` is the catch-all suite of the `excel-probe` skill. It grew from 138
    to 268 cases in one day and has 271 now. Its known failures went 105 → 192 → 243 → 197.
    New probes find gaps faster than they are fixed, so the M1 finish line keeps moving.
  - Many failing probes are outside M1 scope ("core calculation semantics, no functions beyond
    trivial"), even when tagged `M1`. Among the 197 F lines in eval/baseline/probes.txt at the
    freeze commit:
    - GEOMEAN numerics (`geomean-*`, 78 cells);
    - RATE (`rate-empty-arguments`, 9);
    - IFS/SWITCH arrays (`ifs-switch-array-values`, 12);
    - CONCATENATE limits (`concatenate-limit-spill`, 7);
    - WEEKNUM stale state (`weeknum-stale-return-type`, 5; decision 0038);
    - XIRR/XNPV whole columns (`whole-column-empty-walkers`, `xirr-xnpv-whole-column-nogaps`, 9);
    - unimplemented catalog functions (`catalog/*`, 8).
  - Multi-area and union probes (`multiarea-*`, `union-defined-name`; SPEC-0004, T-0133, T-1061)
    and number-text, parsing and `@` probes are M1 scope.
- **Decision:**
  1. **M1's probe criterion is frozen.**
     - The freeze commit is `e5c7ae98315fe7cc76f1e9773d57a7f063dd4d56` (origin/main, 2026-10-08).
     - The M1 probe criterion is the fixed list in `eval/milestones/m1-probes.txt`: 89 cases, one
       case id per line, in the form used by eval/baseline/probes.txt. They are the probe cases on
       main at the freeze commit whose subject is M1 scope:
       - operators, precedence and operator coercion;
       - references: sheet-qualified, the union and intersection operators, multi-area
         references, `@`, and parenthesised calls;
       - number literals, number text and number-to-text, including date and time text read by
         coercion;
       - error literals and stored error constants;
       - 1900 date basics;
       - recalculation (no probe case targets it yet).
       Functions in a listed case only observe the result (ISREF, FORMULATEXT, ERROR.TYPE, ROWS,
       SUM). For `multiarea-*`, the multi-area argument rules are the subject, whichever function
       receives the reference.
     - A case's subject decides, not its tag. Case files and their tags are not edited: that
       could change oracle input hashes. So the list has five cases not tagged `M1`. Their
       subject is operator coercion or error precedence:
       - `paren-number-text-forms`, `paren-negative-variants`, `paren-negative-text`
         (parenthesised number text);
       - `error-operand-broadcast`, `error-operand-broadcast-cmp` (an error operand against an
         array).
     - **Exclusions.** These 66 probe cases are tagged `M1` but are not on the list. They move to
       M2 exit (item 2):
       - GEOMEAN: `geomean-exponent-sum`, `geomean-exponent-sum-2`, `geomean-in-range-many`,
         `geomean-mixed-magnitude`, `geomean-overflow-count`, `geomean-overflow-cut`,
         `geomean-overflow-cut-2`, `geomean-overflow-cut-3`, `geomean-overflow-many-values`,
         `geomean-product-order`, `geomean-root-models`.
       - WEEKNUM and other date functions: `weeknum-1900-later`, `weeknum-invalid-type`,
         `weeknum-serial-0`, `weeknum-stale-return-type`, `dates-early-1900-edges`,
         `edate-serial-0-months`.
       - XIRR/XNPV: `whole-column-empty-walkers`, `xirr-xnpv-whole-column-nogaps`.
       - Variance and statistics: `variance-digits`, `variance-fit`, `variance-rel`,
         `variance-rule`, `variance-scale`, `variance-switch`, `variance-underflow`,
         `large-small-k`, `rank-absent-number`, `dvar-one-vs-two-pass`.
       - MOD: `mod-array-lift`, `mod-huge-quotient`, `mod-quotient-boundary`,
         `mod-quotient-threshold`, `mod-quotient-threshold2`, `mod-quotient-threshold3`,
         `mod-quotient-threshold4`, `mod-quotient-threshold5`.
       - AGGREGATE, SUBTOTAL and aggregate arguments: `aggregate-args`, `aggregate-array-args`,
         `aggregate-grid`, `aggregate-range-args`, `aggregate-overflow`,
         `if-index-ref-args-aggregates`, `sum-long-text-args`.
       - Subnormal results of functions: `subnormal-flush-edges`, `subnormal-function-results`,
         `subnormal-sumsq-sumproduct-edges`.
       - Criteria and lookup: `date-text-dotted-criteria`, `ifs-long-criterion-precedence`,
         `long-text-args-min-criteria`, `tiny-vs-zero-criteria`,
         `lookup-whole-column-blank-tail`, `xlookup-array-return`, `xmatch-xlookup-blanks`,
         `isref-reference-lookups`.
       - Text functions, the TEXT format and ROUND (M2, M5, M10): `error-arg-vs-range-check`,
         `error-precedence-text`, `text-ddd-negative-fraction`,
         `text-general-exponent-boundary`, `round-family`.
       - 3-D references and defined-name bodies (M8): `3d-ref-aggregates`, `3d-ref-functions`,
         `3d-ref-values`, `defined-name-formula-bodies`.
       - Function names and rich-value errors: `ceiling-xlfn-prefix`, `rich-errors-args`.
       Boundary notes: a 3-D operand inside a union (`multiarea-3d-ref`, `multiarea-areas-3d`) is M1,
       because the multi-area argument rules are the subject; the value and function semantics of
       3-D references are M8. VALUE/DATEVALUE date-text cases are in because they exercise the
       text-to-number coercion grammar shared with operators (`"1/2/2020"+0`), not a date function.
       `defined-name-intersection` is in (the intersection operator inside a pure reference body);
       name bodies built from functions (`defined-name-formula-bodies`) are M8 names work.
       Every other probe case not on the list, whatever its tag (catalog availability, `M2`, `M9`,
       `M10`, the M0 limits cases), is M2 exit scope too; for example `ifs-switch-array-values`,
       `round-huge-digits-wrap`, `criteria-gaps`, `criteria-operands`, `filter-if-empty-array` and
       `isref-names-filter-if-empty`, which are tagged M0, M2 or M9.
     - Unchanged: smoke and gen-m1 at 100%, the known parity bugs, and API v0.
     - New probe cases added after the freeze never count toward M1, whatever their tag. The
       list never grows. A line leaves it only by a decision record, for example a case that is
       deleted or split.
  2. **M2 exit requires the whole `probes` suite.** M2 exit needs every `probes` case at 100%
     bit-exact, apart from accepted deviations (decision 0036). That covers all 182 cases not on
     the frozen M1 list, and every probe case added later, until M2 exits.
     - This includes probes of functions whose own milestone is later than M2, such as the
       catalog availability probes for GROUPBY, PIVOTBY and other unimplemented functions. The
       owner asked for "all of these in m2", so they count for M2 by default.
     - *Amended by decision 0050: answered. They count toward their own milestone, and M2 exit
       covers only the probe cases that decision 0050 assigns M0, M1 or M2.*
     - **Open owner question:** should probes of M5+ functions (the catalog availability cases
       and function-specific probes such as RATE, XIRR, GEOMEAN, WEEKNUM) count toward each
       function's own milestone instead of M2? The decision does not settle it. Until the owner
       answers, they count for M2.
  3. **Fix before discovering more.** While a milestone's frozen list has known failures:
     - The steward puts the lanes that fix them (engine, then harness) first on the
       `**Critical lane:**` line, ahead of research.
     - The planner files fix tasks for those failures before new probe (research) tasks for that
       milestone.
     - After the freeze, the planner files no new probe task with `milestone: M1`. New probes go
       to M2, or to the milestone of the function they probe.
     - Open M1 tasks whose cases are excluded keep their ids and milestone labels in this
       change, so that claimed branches do not conflict: T-1099, T-1120, T-1123 (GEOMEAN) and
       T-1124 (XIRR). The cases from the open M1 probe tasks (T-1131, T-1135, T-1137 to T-1141,
       T-1143) come after the freeze, so they count toward M2 only. None of these tasks gates M1
       exit.
  4. **Measurement.** No tool reports a case subset's pass rate today. T-1146 (harness, M1, P1)
     makes `make eval` report the known failures of each list in `eval/milestones/`. Until it
     lands, the steward counts them from the baseline:
     `awk -F'\t' 'NR==FNR{if($0!~/^#/)L[$0]=1;next} $1=="F"{if($2 in L)m++;else o++} END{print m+0, o+0}' eval/milestones/m1-probes.txt eval/baseline/probes.txt`
     prints the M1-list and remaining F lines.
     - At the freeze commit there are 197 F lines. 61 are on the frozen M1 list: `multiarea-*`
       45, `error-stored-constants` 7, `paren-number-text-forms` 3, `at-paren-operands-levels`
       2, `datevalue-separator-spaces` 2, `number-text-exponent-digits` 1 and
       `union-defined-name` 1. The other 136 are M2 exit scope.
- **Consequences:**
  - The M1 and M2 exit criteria in docs/milestones.md cite this record. The steward checks M1
    against the frozen list and reports both counts in docs/status.md at its next run.
  - `.claude/skills/steward/SKILL.md` and `.claude/agents/planner.md` get the rules in item 3.
  - M1 can exit once the 61 list failures, gen-m1, smoke, the parity bugs and API v0 are done.
    The 136 other probe failures, and every later probe, move the M2 finish line instead. M2
    exit grows with the suite until M2 exits, unless the owner answers the open question
    differently.
  - `eval/milestones/m1-probes.txt` is data that the harness reads once T-1146 lands. A PR that
    edits it needs a decision record. T-1146 makes the governance check enforce that.
