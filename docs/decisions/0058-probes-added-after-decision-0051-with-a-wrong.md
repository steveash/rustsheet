# 0058 — Probes added after decision 0051 with a wrong milestone tag count toward their subject's milestone

- **Status:** accepted (2026-10-14, T-1470). Amends decision 0051 (the list of
  `eval/milestones/probe-milestones.tsv` lines); the rule of 0051 is unchanged.
- **Context:** Decision 0051 item 2 says a probe case counts toward the milestone of the function or
  feature it probes, and that a case whose tag does not match its subject gets an override line.
  Probes written after 0051 for M1 or M2 tasks kept the task's tag (`M1`, `M2` or none), so
  they counted toward M2 exit although their subjects are later-milestone functions. Decision
  0057 now stops new `M0`/`M1`/untagged probes; it does not fix the existing ones, nor an `M2`
  tag on a later-milestone subject. At origin/main 72a4f7ce, `make eval` assigned **512** probe
  known-failure cells to M0-M2 (361 cases, 13,249 cells).
- **Decision:** apply 0051 item 2 to every such case and cell. Each subject milestone is the highest
  catalog milestone (`eval/catalog/scope.json`) of the subject functions, ignoring the observing
  wrappers, or the feature milestone (external references M8, number-format codes M10):
  - **M9, financial functions (22 cases):** ODDF*/ODDL*/ACCRINT: `date-system-1900-odd-years`,
    `-1904-odd-years`, `-1900-odd-threshold`, `-1904-odd-threshold`, `-1900-odd-accrint-bound`,
    `-1904-odd-accrint-bound`, `-1900-oddf-issue-after-quasi`, `-1900-accrint-settlement-bound`,
    `-1904-accrint-settlement-bound`. Financial date readers: `date-system-1900-financial-readers`,
    `-1900-financial-readers-shifted`, `-1904-financial-readers`, `-1900-financial-daycount`,
    `-1904-financial-daycount`, `-1904-financial`, `-1904-financial-shift`. Coupon functions:
    `date-system-1900-coupon-bound` and its passing twin `-1904-coupon-bound`. YIELD
    `yield-solver-sweep`; COUPDAYS `coupdays-feb-month-end`; TBILLEQ `tbilleq-zero-price`
    (tagged `M2`). COUNTBLANK `countblank-array-values` (tagged `M2`).
  - **M10, number-format codes in TEXT (12 cases):** `text-format-unquoted-letters`,
    `text-format-length-limit-c` to `-m` (0051 already maps the first two of the series).
  - **M8, external references (3 cases):** `external-ref-no-link-shapes`, `-index`, `-ifchoose`
    (the last tagged `M2`); 0051 maps `external-ref-no-link`.
  - **M7, LAMBDA helpers (2 cases):** `lambda-helper-range-slices` (BYROW/BYCOL/MAP/SCAN/REDUCE;
    COUNTA, COUNTBLANK, ISREF and ROW only observe the slice) and `eta-reduced-names` (both tagged `M2`).
  - **M5, TEXTJOIN (3 cases, untagged):** `textjoin-delimiter-array`, `textjoin-array-arguments`,
    `textjoin-array-limit-empty-strings`.
  - **Cell lines (21):** `volatile-invariant` Calc!A2, A8, A12, A13 and F1:F9 → M5 (RANDBETWEEN;
    in F the SEQUENCE only builds the bounds and MIN/MAX/ROWS observe); B9, B10, B18 → M7
    (RANDARRAY's catalog milestone, not M5 as the task guessed). `index-lifted-edge-cases`
    Sheet1!H131:J131 → M9 (COUNTBLANK of a lifted INDEX; passing today). `written-parens-reference-only`
    Sheet1!E40, E45 → M7 (ISREF of LET; ISREF observes). The rest of each case keeps its tag under
    0051's exception (NOW/TODAY; INDEX and the reference operators are M2).
  - **Keep their tag (0051's exception: the subject includes an M2 function or an M1 subject):**
    `date-system-1904-readers` (WEEKDAY…, date-text coercion, criteria, TODAY/NOW);
    `isref-implicit-xlookup-take` and `let-bind-implicit-index` (`@` on INDEX, implicit intersection);
    `formula-length-nonbmp-8192u` and `name-body-nonbmp-8191u` (formula length limit, LEN);
    `text-collation-order`, `text-order-nbsp-space` (T-0220), `text-surrogate-positions`,
    `concatenate-limit-spill` (T-1030) and `array-context-legacy-sumproduct` (named in 0051).
  Every moved failing cell keeps its open task in its own milestone; nothing leaves the reports.
- **Consequences:** `make eval` now assigns **179** probe known-failure cells to M0-M2 (319 cases,
  10,186 cells), all of them M2 subjects: `text-collation-order` 156,
  `array-context-legacy-sumproduct` 12, `text-surrogate-positions` 6, `concatenate-limit-spill` 4,
  `text-order-nbsp-space` 1. The excel-probe skill tells probe writers to tag the subject's
  milestone. A probe that 0057 lets through with a wrong later tag (`M2` on an M9 subject, as
  `tbilleq-zero-price`) still needs a reviewer to catch it and a line here.
