# 0059 — Older probe cases counted for M0-M2 with a later-milestone subject count toward their subject's milestone

- **Status:** accepted (2026-10-14, T-1547). Amends decision 0051 (the lines of
  `eval/milestones/probe-milestones.tsv`, and its examples `subnormal-*` and `rich-errors-*`) and
  completes decision 0058; the rule of 0051 is unchanged. It also reverses two of 0051's
  "deliberate keeps with no F line today", `subnormal-sumsq-sumproduct-edges` and
  `ifs-shape-mismatch`, which T-1547 named for review (reasons below).
- **Context:**
  - Decision 0058 applied 0051 item 2 to the probes added after 0051 and left the older cases to
    T-1547.
  - The sweep: every `probes` case that counts for M0-M2 (not on m1-probes.txt, not in
    probe-milestones.tsv, tagged `M0`/`M1`/`M2` or untagged) whose called catalog functions,
    apart from observers, are all above M2 in `eval/catalog/scope.json`. It was run twice: once
    with 0051's wrappers only (OR, IFERROR, ERROR.TYPE, INDEX, ISREF, ROWS, FORMULATEXT), and once
    with a wider observer list (IF, IFNA, AND, NOT, TYPE, N, IS*, ROW, COLUMN, COLUMNS, SEQUENCE,
    CELL, INFO). Together they flag 71 cases; T-1547 also named `ecma-ceiling-formulatext` and
    `union-walk-repeated-in-lambda`.
  - Each was decided by its description and formulas under 0051 item 2. Two readings make the
    calls consistent:
    - **A function's argument reader is that function's subject.** A case that feeds an M1
      construct (a union, a legacy-intersected operand, an `IF`/`INDEX` reference) to
      later-milestone functions and asks how each one reads it probes those functions. This is the
      same call as 0051's `stat-argument-modes` (M9). The construct's own control cells keep the
      tag.
    - **The coercion grammar is an M1 subject.** A case that checks TIMEVALUE, DATEVALUE or VALUE
      beside `0+text` or `--text` of the same text probes the grammar the coercion shares. This is
      0051's call on `clock-text-*` and `datevalue-comma-spaces`.
- **Decision:** add these lines to probe-milestones.tsv (the subject is in each line):
  - **M5 (14 cases):** `ecma-ceiling` and `ecma-ceiling-formulatext` (ECMA.CEILING, like 0051's
    `ceiling-xlfn-prefix`); `legacy-type-intersection` (TYPE's legacy intersection; ROWS,
    SUBTOTAL and AGGREGATE are controls); `multiarea-subtotal-counts-a` to `-e` (SUBTOTAL of a
    union; LET, IF and CHOOSE only route it); `percentof-snap-overflow-union`;
    `rich-errors-stockhistory` (DATE builds the argument); `subnormal-sumsq-sumproduct-edges`
    (SUMSQ's per-square flush; it calls no M2 function, only SUMSQ and SUMPRODUCT, so 0051's
    exception does not reach it);
    `subtotal-variance-one-vs-two-pass` (like 0051's `dvar-one-vs-two-pass`);
    `text-search-casefold-length` (SEARCH, FIND); `textjoin-delimiter-elements`.
  - **M6 (9 cases):** `date-huge-month-day` (DATE); `multiarea-cellfn-types` (CELL, COLUMN over a
    union); the IMAGE/FIELDVALUE cases `rich-errors-arg-kinds`, `-field`, `-image`,
    `-image-array-error-elements`, `-image-array-options`, `-image-sizing`, `-shapes-urls`.
  - **M7 (1 case):** `isref-reference-lookups` (XLOOKUP, TAKE, DROP, TRIMRANGE; ISREF observes).
  - **M9 (6 cases):** `averagea-array-constant-elements`; `if-index-ref-args-aggregates`
    (MAXA, MINA, AVERAGEA, LARGE, RANK; IF and INDEX build the argument);
    `ifs-shape-mismatch` (it calls only AVERAGEIF, AVERAGEIFS, MINIFS and MAXIFS, and the shape
    check it probes is theirs; it has no SUMIFS or COUNTIFS cell, and no criteria-grammar
    cell, so 0051's "shared criteria engine" keep gives it no M2 subject); `legacy-value-argument-intersection-c` (STDEV … PERCENTRANK; the description also names COUNTBLANK, which no cell calls);
    `multiarea-foreach` and `multiarea-order-stats` (statistical functions over a union).
  - **M10 (2 cases):** `text-general-ties-below-one` and `-wide-fixed` (the `General` format
    code, like 0051's `text-general-exponent-boundary`).
  - **Cell lines (84):**
    - XOR → M5: `and-or-xor-error-order` Logic!A5, A6, A9, A25; `and-or-xor-text-arguments`
      Sheet1!H57:H84.
    - SWITCH → M5: `omitted-arg-compare` Sheet1!D1, D2.
    - CHOOSE → M6: `isref-function-results` Sheet1!C5, C6.
    - WORKDAY, WORKDAY.INTL and NETWORKDAYS → M6: `date-system-1904-clock-text` S!A8:A11.
    - `subnormal-text-and-tail-companions`: CSCH → M5 (Sheet1!A5:B5); NORM.S.DIST, PHI,
      NORM.DIST, ERFC, ERFC.PRECISE → M9 (Sheet1!A6:B15).
    - Inside moved cases: `legacy-type-intersection` Sheet1!M2, M10 → M9 (AVERAGEA); its
      legacy control O10 (`=B1:B6*1`) → M2, and so is `legacy-value-argument-intersection-c`
      Sheet1!AG10.
    - Inside moved cases, a cell whose own subject is another milestone: PRODUCT →
      M5 in `if-index-ref-args-aggregates` Calc!A28:A36; `rich-errors-image` Sheet1!A3 (SEQUENCE)
      and A6 (FILTER) → M7, and its `=1/0` control A8 → M2.
    - `multiarea-more` keeps its tag (SUM, COUNT, COUNTA over a union), and its later-milestone
      cells move: Sheet1!H7 (SUMSQ) → M5; H8 (AVERAGEA), H9 (MAXA), H10 (GEOMEAN), H11 (VAR.S),
      H12 (DEVSQ) → M9.
    - Not given lines: cells that only route the argument (LET in `multiarea-subtotal-counts-c`,
      IF and CHOOSE in its twins) and `=1/0` cells that only set up data (C4 of
      `multiarea-subtotal-counts-*`, E1 of `multiarea-order-stats`).
    The rest of the seven kept cases with cell lines keeps its tag (AND, OR, IF, INDEX, the comparison of an
    omitted argument, date-clock coercion, subnormal number text, SUM, COUNT and COUNTA over a union).
  - IMAGE, FIELDVALUE and STOCKHISTORY are `scope: out` (decision 0027); like every function, they
    count by their catalog milestone.
  - **Keep their tag (0051's exception: the subject includes an M2 function or an M1 subject):**
    - Operators and formula printing: `at-group-leftmost-shapes` and `at-signed-operand` (`@`;
      OFFSET and INDIRECT build its operand), `colon-hash-value-operands`,
      `formulatext-written-groups-shapes`, `multiarea-bare-precedence`, and
      `multiarea-3d-union-h8` (the union operator, like its twins `-h1` to `-h7`).
    - Coercion grammar: the nine flagged `clock-text-*` cases, `date-clock-text-length-limit`, `timevalue-*` (4),
      `date-system-1904-timevalue` and `-timevalue-chrono`, `datevalue-comma-spaces`,
      `typed-currency-percent-input`, `typed-number-tiny` and `typed-paren-number-input`.
    - M2 functions: `index-snap-tolerance` (INDEX), `not-array-cast` (NOT),
      `union-walk-repeated-in-lambda` (SUM over a union; BYROW and MAP repeat it).
    - Other M0/M1 subjects: `lifted-range-rows` (the resource limit; DATE only lifts), and
      `rich-errors-calc` (the cached rich-value error kinds and their ERROR.TYPE, next to `1/0`
      and `NA()` controls). This replaces 0051 item 3's example `rich-errors-*` with
      `rich-errors-calc`.
    - Already decided: `isref-implicit-xlookup-take`, `text-collation-order` and
      `written-parens-reference-only` (0058).
  - T-1558, the open task for the two moved failing cells (PERCENTRANK and PERCENTRANK.INC in
    `legacy-value-argument-intersection-c` T2, U2), moves to M9 with them.
- **Consequences:** at origin/main 4fbdd96c, `make eval` assigned **175** probe known-failure
  cells to M0-M2 (319 cases, 10,156 cells). With these lines it assigns **173** (290 cases,
  9,206 cells): `text-collation-order` 156, `array-context-legacy-sumproduct` 6,
  `text-surrogate-positions` 6, `concatenate-limit-spill` 4, `text-order-nbsp-space` 1, all M2
  subjects (0058). Only T2 and U2 moved; the other moved cells pass today, and a later
  regression in them counts toward their own milestone. No case now counted for M0-M2 has only
  later-milestone subjects, unless this record keeps it by name.
