# 0060 — Later-milestone cells of probe cases counted for M0-M2 count toward their subject's milestone

- **Status:** accepted (2026-10-14, T-1562). Amends decision 0051 (the lines of
  `eval/milestones/probe-milestones.tsv`) and completes decisions 0058 and 0059. The rule of 0051
  is unchanged.
- **Context:**
  - Decision 0059 swept the older probe cases *by case*. A case whose non-observer functions
    were all above M2 moved. A kept case got cell lines only where 0059 looked at it cell by
    cell (`multiarea-more`, `and-or-xor-*`, `omitted-arg-compare` and the other cases it lists).
  - Its fidelity review found kept mixed cases that nobody swept by cell, for example
    `legacy-value-argument-intersection-b`. Such a case calls MIN, SUM and CONCAT, so it keeps its
    tag, but about 30 of its cells only call MINA, STDEV.S, GEOMEAN and other M9 functions.
  - Such cells pass today. A later regression in one of them would weigh on M2 exit, although
    their subject is a later milestone (0051 item 2).
  - Probes added since 0059 with a wrong tag were not swept either. Examples are
    `trig-huge-argument` (SIN … CSC, tagged `M2`) and `percentrank-single-value`.
  - The sweep (T-1562) takes every checked cell (a key of the case's `.oracle.json`) of every
    `probes` case that counts for M0-M2: not on m1-probes.txt, not mapped by a case line, tagged
    `M0`/`M1`/`M2` or untagged, and without a cell line already. It flags a cell when every
    catalog function in its formula, apart from observers, is above M2 in
    `eval/catalog/scope.json`. The observers are 0051's wrappers (OR, IFERROR, ERROR.TYPE, INDEX)
    plus 0059's wider list (IF, IFNA, AND, NOT, TYPE, N, IS*, ROW, COLUMN, COLUMNS, ROWS,
    SEQUENCE, CELL, INFO, FORMULATEXT).
  - At origin/main 5ab38239 the sweep flagged **2,020 cells in 117 cases**.
- **Decision:** a flagged cell counts toward its subject's milestone unless one of the keep
  rules below applies. The subject is the cell's *reader*: the outermost non-observer function,
  together with the other non-observer functions it calls. A function that only builds an
  operand does not count (0058's argument-builder rule): REPT, CHAR, NA, DATE, CHOOSE, OFFSET,
  INDIRECT, LET, LAMBDA, TAKE, DROP, XLOOKUP, TRANSPOSE and CHOOSEROWS nested in a reader. This
  is 0059's first reading, "a function's argument reader is that function's subject", applied
  cell by cell. Number-format codes in TEXT are M10, as in 0058. A reference into another
  workbook is M8, as in 0058. A companion cell that only observes another cell's result (the
  `&""` and `=0` columns of decision 0042, or ERROR.TYPE/ISERROR next to the producing cell) follows
  that cell.
  - **Case lines (4).** Every checked cell of these cases is flagged:
    - `external-ref-no-link-ifchoose-b` → M8 (the external-reference feature, like its
      siblings in 0058);
    - `trig-huge-argument` → M5 (SIN … CSC);
    - `percentrank-single-value` → M9;
    - `stat-a-computed-arrays` → M9 (AVERAGEA, MAXA, MINA).
  - **Cell lines (1,086).** These go in the second new section of probe-milestones.tsv. Each line
    names its subject. By milestone:
    - **M5 (508):**
      - text positions and counts in FIND, SEARCH, REPLACE and the *B forms:
        `text-count-argument-bounds` 177 and `text-surrogate-positions` 40 (the LEN, LEFT, RIGHT
        and MID cells keep the tag);
      - PRODUCT, SUMSQ, SUMPRODUCT, SUBTOTAL, AGGREGATE, MMULT, SERIESSUM, SUMX2MY2 and the like:
        `aggregate-overflow` 63 (SUM keeps), `subnormal-function-results` 45,
        `subnormal-flush-edges` 24, `aggregate-error-precedence` (8 PRODUCT cells; SUM, MIN, MAX
        and AVERAGE keep), `near-zero-aggregate-forms` 9;
      - TRUNC in `trunc-decimal-stages` 44 (ROUNDDOWN keeps);
      - TEXTJOIN, SUBSTITUTE, GCD, LCM, SWITCH, VALUETOTEXT, ARRAYTOTEXT, STOCKHISTORY, MOD and
        INT, and REPT's own argument errors (`error-precedence-text` C20–C23).
    - **M6 (292):**
      - LOOKUP, XLOOKUP and XMATCH reading a reference-valued or blank lookup value:
        `lookup-reference-lookup-value` 100, `-blank-error` 75, plus `lookup-2d-vector`,
        `lookup-whole-column-blank-tail`, `lookup-approximate-search`,
        `at-if-spill-anchor-operand` and `tiny-vs-zero-criteria`;
      - the date functions of `date-system-1904-readers` (WEEKDAY, DATEDIF, EDATE …; its
        date-text coercion, criteria and NOW/TODAY cells keep), and DATE and DATEVALUE in
        `date-system-1900/1904-negatives`;
      - WEEKDAY's argument errors in `error-precedence-text`;
      - HOUR, MINUTE and SECOND of clock text;
      - AREAS, OFFSET and CHOOSE as readers or as a picked reference (like 0059's
        `isref-function-results` C5, C6): `lookup-pick-index-shapes` C3, C4 and
        `defined-name-grouped-reference` A8, A9;
      - IMAGE and FIELDVALUE.
    - **M7 (23):**
      - `rich-errors-calc` Sheet1!A1 (SEQUENCE) and A4 (FILTER) with their ERROR.TYPE and
        ISERROR observers B1, C1, B4, C4. These are the `#SPILL!` and `#CALC!` cells, which 0059
        moved in `rich-errors-image` (A3, A6). The cached rich-value error cells A6, A7, B6 and
        B7 keep 0059's call.
      - LET binding a reference read by ROW or ROWS (`min-max-reference-forms` D3, D4, D9, G5;
        `written-parens-reference-context` E14, G14, like 0058's `written-parens-reference-only`
        E40, E45);
      - the value of a LAMBDA call (`operand-wrap-text` C20, C22, like 0059's
        `formulatext-written-groups-shapes` B3);
      - SORT, UNIQUE and FILTER.
    - **M9 (229):**
      - the statistical functions over an M1 construct (legacy intersection, an XLOOKUP pick,
        a union name, an IFERROR empty value): `legacy-value-argument-intersection-b` 38 and
        `-intersection` 6, `subtotal-reference-operands-d` 15, `average-value-args` 24 (MEDIAN,
        GEOMEAN, STDEV …; AVERAGE and COUNT keep), `aggregate-overflow` 51;
      - AVERAGEIF(S), MINIFS, MAXIFS and the D-functions (the criteria-grammar cells of the
        same cases, through SUMIF(S) and COUNTIF(S), keep);
      - COUNTBLANK, FREQUENCY, BITAND and CHISQ.DIST;
      - the coupon functions in `date-system-1904-readers`.
    - **M10 (34):** TEXT with a number-format code (`date-system-1900/1904-negatives` 14 each,
      `date-system-1904-readers` 3, `subnormal-function-results` 3).
  - **Flagged cells that keep their case's milestone (812):**
    - **Coercion grammar (394).** 0059's second reading. These are TIMEVALUE, DATEVALUE and
      VALUE cells in a case that reads the same text through `0+text`, `--text` or `+0`, or that
      0059 kept by name for that reason. That covers the 14 `clock-text-*` and `timevalue-*`
      cases, `date-clock-text-length-limit`, `datevalue-comma-spaces`,
      `date-system-1904-timevalue` and `-timevalue-chrono`, `date-system-1904-pre-epoch-text`
      (its three DATE cells only set up criteria data) and the two coercion cells of
      `date-system-1904-readers`.
    - **An operator counted or built (319).** The subject is a comparison, a concatenation or an
      operator, and a later function only counts it or builds its text. That covers
      SUMPRODUCT(--(…)) in `text-collation-order` (292, an M2 subject, 0058) and CHAR building
      text in `text-order-nbsp-space` (21) and `criteria-padded-text` (2). The remaining four
      (`multiarea-bare-precedence` H8, H9, `match-descending-argcheck` J16,
      `index-array-indices` H44) have NA or CHOOSEROWS building an operand of a union or of INDEX.
    - **A reference construct built by a later function (15).** The subject is `@`, the range
      operator or a union, and OFFSET, INDIRECT, CHOOSE, LET or LAMBDA only make its operand
      (0059: "OFFSET and INDIRECT build its operand"; LET and CHOOSE only route a union). This
      covers `at-signed-operand`, `at-group-leftmost-shapes`, `colon-hash-value-operands`,
      `anchorarray-at-operand-kinds`, `range-operator-unnamed-operands`, `isref-range-operands`,
      `multiarea-letbind` and `counta-empty-value` D17 (LET routes IFERROR's empty value to
      ISBLANK).
    - **Data cells (15).** A cell that only sets up data for other cells, `=NA()` (13) or
      `=REPT("a",255)`/`=REPT("a",256)` (`long-text-args-min-criteria` B2, B3). This is
      0059's `=1/0` rule.
    - **Already decided by name (69).** A case that 0051, 0058 or 0059 kept by name with a reason
      that covers these cells, or swept cell by cell: `array-context-legacy-sumproduct`,
      `let-bind-implicit-index`, `isref-implicit-xlookup-take`,
      `written-parens-reference-only` (E50, E55: 0058's "the rest keeps its tag"),
      `omitted-arg-compare`, `and-or-xor-error-order`, `and-or-xor-text-arguments`,
      `subnormal-text-and-tail-companions`, `date-system-1904-clock-text`,
      `lifted-range-rows`, `union-walk-repeated-in-lambda`, `not-array-cast`, and
      `multiarea-more`.
  - T-1563, the open task for the six failing `trig-huge-argument` cells, moves to M5 with them
    (as 0059 moved T-1558).
  - No case file is edited (decision 0043).
- **Consequences:**
  - `make eval` M0-M2 counts are in the table below. 1,216 cells leave M0-M2: the 1,086 cell lines, plus the cells of the
    four case lines (M5 +86, M8 +6, M9 +38). Of its known
    failures only the six `trig-huge-argument` cells (now M5) move. The 171 that stay are
    `text-collation-order` 156, `text-surrogate-positions` 6 (LEN/LEFT/MID cells),
    `array-context-legacy-sumproduct` 4, `concatenate-limit-spill` 4 and
    `text-order-nbsp-space` 1, all M2 subjects.
  - A later regression in a moved cell counts toward its own milestone. No probe cell counted for
    M0-M2 now has only later-milestone subjects, except the kept cells this record names by rule.
  - New probes still need a correct tag (0057 rejects only `M0`/`M1`/untagged). A reviewer who
    sees an `M2` tag on a later-milestone subject asks for a line here, as with
    `trig-huge-argument`.

| `make eval` probes, M0-M2 row | cases | cells | known failures |
|---|---|---|---|
| before (origin/main 5ab38239) | 292 | 9,328 | 177 |
| after | 288 | 8,112 | 171 |
| after, merged with origin/main 5fb38a9e (adds T-1508's and T-0409's probes; the sweep flags none of their cells) | 288 | 8,112 | 171 |
