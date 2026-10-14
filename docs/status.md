# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

## Snapshot — 2026-10-14 (M2 is current)

| Suite | Cases | Cells | Bit-exact | Known failures |
|---|---|---|---|---|
| ironcalc-calc | 217 | 72,203 | 97.96% | 1,474 |
| poi | 336 | 27,290 | 98.18% (204 cases pending: no formulas, unreadable, or web-vs-desktop conflicts) | 512 |
| gen-m2 | 30 | 12,000 | 99.95% | 6 |
| gen-m1 | 15 | 3,750 | **100%** | 0 |
| probes | 575 | 25,483 | 79.54% (4 accepted deviations, 11 cells passed by decision 0042) | 5,214 |
| smoke | 6 | 46 | 100% | 0 |
| fixtures | 6 | 67 | 100% | 0 |
| gen-m5 | 10 | 400 | 98.50% | not gated |
| gen-m6 | 25 | 1,000 | 97.90% | not gated |
| gen-m7 | 25 | 994 | 94.57% | not gated |
| gen-m9 | 25 | 998 | 90.88% | not gated |
| probes-numeric | 106 | 15,613 | 52.48% (4 cases time out, see T-1518) | not gated |

`make eval-check` on `main` (6ab245f0) shows 0 regressions and 0 baseline fixes in every gated suite.
probes grew by 40 cases and 4,685 cells, mostly the oracle sweeps T-1042 to T-1048. Its new known
failures come from those new cases, not from regressions.
CI on `main` is green: every CI run on `main` since the last snapshot passed, through 6ab245f0 (T-1498). One run, 148b4ae0, was cancelled because a newer push superseded it.

Where M2's exit stands:
- gen-m2 has 6 cells left (was 11). T-0220 (collation) covers 5 and T-1495 (powers of SQRT(5)) covers 1.
- M0–M2 probes: 355 cases, 13,079 cells, 12,557 bit-exact, 4 deviations, 518 known failures (was 196).
  - The rise comes from probe cases added on 2026-10-13. They carry a post-freeze `M1` tag but probe M9/M10 subjects: `yield-solver-sweep` 66, the `date-system-*-financial-readers*` and `*-odd-years` cases 79, and `text-format-unquoted-letters` 75.
  - `text-collation-order` (156) is a real M2 subject (T-0220).
  - I added the new cases to T-1470 (research, P1).
  - The planner filed **T-1519** (harness), a gate that rejects new probe cases tagged M0/M1 or untagged.
- ironcalc-calc cells using only M1/M2 functions: 3,977 scored, 9 failing, all TBILLs.xlsx L-column (T-1475).
- Duplication is at 6,522 lines, 28.7% below the fork's 9,153 (target ≥ 15%), so the number is met. "One implementation of aggregate-argument iteration" is still open: T-1473 → T-1474 → T-1476, and T-1477.
- NOW/TODAY clock: the spec landed (T-1478, #682). T-1479 (engine) → T-1480 (api) remain.

Hygiene: lint-debt 91 allow entries (= last run) · duplicated lines 6,522 (ratchet 6,587) · oversized files 14 (ratchet 15).
`crates/rustsheet-xlsx/src/import/worksheets.rs` is no longer oversized. The steward left both ratchets as they were, because `quality/` is a protected path; the next quality PR should run `--update`.

Since the 2026-10-13 snapshot (steward PR #677, d64e83e7), 56 task PRs merged to `main` through 6ab245f0. None was reverted.
- T-0050, T-0053, T-0054, T-0221, T-0290, T-0416, T-0418
- T-1030, T-1042–T-1048, T-1223, T-1248, T-1282, T-1328, T-1377, T-1382, T-1396, T-1397
- T-1426, T-1437, T-1446, T-1448, T-1450, T-1455, T-1457, T-1459, T-1462–T-1464, T-1466–T-1469
- T-1471, T-1472, T-1478, T-1481, T-1482, T-1485, T-1489, T-1491–T-1494, T-1496–T-1498
- T-1501, T-1503, T-1505, T-1506

T-1505 (#714) moved merging to the GitHub merge queue (decision 0053).

Blocked (18, unchanged):
- the Phase 2 tasks T-0400, T-0401, T-0402 and T-1066 (decision 0027);
- T-1099 (GEOMEAN last ulp);
- the M14 desktop-oracle tasks T-0171, T-0198, T-1016, T-1051, T-1074, T-1075, T-1081, T-1174, T-1204, T-1277, T-1299, T-1310 and T-1367 (decision 0039; `rustsheet-oracle status` still shows excel-desktop n/a).

None can be unblocked.

**Nightly fuzz:** run 38508146682 (2026-10-14, cbf9c5c8) failed on `formula_eval`.
- The cause was a libFuzzer timeout on `T.INV(0.9,5E10)`. `fn_t_inv` only rejects df < 1 and passes a huge df to `statrs`, whose quantile search does not finish.
- Excel is `#NUM!` for df >= 1.5E10 (statistics.md, T-1043).
- I reproduced the cause by reading the code: there is no cargo-fuzz/nightly toolchain in the steward container. The same family also makes 4 `probes-numeric` cases time out in `make eval` (`m9-t-edges`, `m9-t-nonint-df`, `t1048-tinv`, `m9-legacy-finv`).
- Filed **T-1518** (functions-stat, P0, sonnet). It covers the df cut, the regression test and the 4 cases.
- `formula_parse` and `xlsx_import` passed.

Open PRs: 10.
- Under 24 h: #698 (T-1486, draft). #718 (T-1490, b8c8d932) and #687 (T-1454, 75903c6a) merged during this run.
- Over 24 h:
  - #661 (T-1450) and #644 (T-1396) are superseded: both tasks merged through #725 and #711. I commented that they can be closed.
  - #551 (T-1331) and #314 (T-1100) are superseded (commented on 2026-10-13).
  - #553 (T-1284), #436 (T-1186) and #409 (T-1155) are blocked drafts. No action.
- The live routine prompt still forbids merging other sessions' PRs, so no orphan was merged (see Needs owner).

Queues at the start of the run:
- functions: 107 ready. The sonnet buffer is 77, so the gap-triager did not run.
- engine, api, harness and research: 0 ready.
  - engine (9 unclaimed todo M2 tasks) and research (2) were busy lanes, not empty ones.
  - harness had no M2 task, and api had only T-1480 (waits on T-1479).
  - The planner ran for M2 and filed **T-1519** (harness, opus). It filed nothing for api: T-1480 is all of SPEC-0008's api scope.
- quality 5, perf 11.
- Planner notes for reviewers:
  - T-1480's acceptance criteria leave out tests from SPEC-0008's test plan: rollback, setting the same clock, `bool` raising `TypeError`, and the `open`/`from_bytes` keywords. Its reviewers should hold it to the spec.
  - Probe tasks are still filed with `milestone: M1` (e.g. T-1489, T-1493), which is how the wrongly tagged cases arise.

Upstream IronCalc: 4 new commits through 1f40af93.
- Filed **T-1517** (functions-date, M6, sonnet) to add the Excel-authored workbooks `EOMONTH_issue_1388.xlsx` (1f40af93) and `WEEKDAY_WEEKNUM_ISOWEEKNUM_issue_1502.xlsx` (37b43745) to calc_tests.
- No code port:
  - the EOMONTH overflow fix (1f40af93): we already use checked month arithmetic;
  - WEEKDAY date-before-return_type (0700c7d7): our lift already does this, and `probes/error-precedence-text` passes;
  - db333e72 (wasm32 target cfg): we build no browser target.

Oracle: excel-graph OK.
- The fill on ironcalc-calc wrote 0 sidecars (214 skipped).
- The fill on poi wrote 0 sidecars (292 skipped, the same 4 workbook failures as T-0019).

Spot-audit: none this run (not yet due). The last audit, 2026-10-09 (steward PR #466), re-reviewed #464, #448 and #453 on opus. The next audit is due 2026-10-16.

**Critical lane:** engine, research, harness, quality, api

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M2.
- engine holds M2's remaining failures: T-0220 (collation: 5 gen-m2 cells and 157 probe cells) and T-1495. It also holds the aggregate walker (T-1473/T-1474) and the clock (T-1479).
- research: T-1470 moves the wrongly tagged probe cases that now hold most of the M0–M2 row's 518 known failures.
- harness: T-1519 stops new wrongly tagged probe cases.
- quality: T-1476 and T-1477 (one implementation).
- api: T-1480 (clock setter), after T-1479.
- T-1475 (TBILLEQ) and the P0 T-1518 are reached through `functions`. perf stays out: Phase 2.)

## Needs owner

- **Replace step (4) of the steward routine prompt** with the text in decision 0050's "Owner action" section (docs/decisions/0050-…). docs/process/routines.md already carries the new text, but the live routine still says "do not merge PRs opened by another session". The 2026-10-14 run's live prompt still carries the old step (4), so this run again merged no orphans. Agents never edit routines (decision 0008).

excel-desktop needs are not owner items (decision 0039): T-1016, T-1051 and the other desktop tasks wait in M14 (Phase 4), tracked by T-1074 and T-1075.

## Log
- 2026-10-02 — Owner set `main` as the GitHub default branch and enabled the routines (repo attached).
- 2026-10-01 — `main` created. Routines must be created in the claude.ai UI with the repo attached (docs/process/routines.md).
- 2026-10-01 — M0 bootstrap: fork, eval harness, oracle client, generator P0, CI, agent process.
- 2026-10-02 — T-0200: shared range-number iterator; duplicated lines 9153→8829; eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0002: pinned external suite fetcher (`rustsheet-eval fetch`); eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0201: criteria/wildcard matching consolidated in functions/criteria.rs; fixed COUNTIF "<>text" on non-text cells; eval 0 regressions, 0 baseline fixes (+2 new spec cases pass).
- 2026-10-02 — T-0003: suite `poi` (Apache POI, fetched, pinned); 334 cases, 3,526 cells, 82.25%; baseline +633 known failures (new suite, 0 regressions); filed T-0111 (timeouts), T-0112 (imports).
- 2026-10-02 — T-0204: coercing range walk shared by VARA/STDEVA family and GCD/LCM; duplicated lines 8829→8610; eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0007: pending cases now run crash-only (panic/timeout/error reported, per-suite crashes column); eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0205: D-function criteria routed through shared criteria grammar (CriteriaFlavor::Database); eval 0 regressions, 0 fixed (+1 new smoke case passes).
- 2026-10-02 — T-0005: transitive volatile skipping + METADATA-pinned clock (NOW/TODAY now scored); NOW() keeps sub-second precision; ironcalc-calc +2 scored cells pass; eval 0 regressions, baseline unchanged.
- 2026-10-02 — T-0008: `rustsheet-gen --policy numeric-rules` → suite `probes-numeric` (2,738 hypothesis-tagged probe cells, pending oracle); eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0202: catalog extended with 18 M2 functions + pair-repeat groups in generator; eval 0 regressions, 0 fixed (tools-only).
- 2026-10-03 — T-0902: eval snapshot reader decodes _xHHHH_ escapes (CHAR 29 cells fixed; ironcalc-calc known failures 1909 -> 1880)
- 2026-10-03 — T-0100: SPEC-0001 public API v0 (facade crate) accepted; filed T-0113; docs only, eval unchanged.
- 2026-10-03 — T-0900: BESSELJ/BESSELY bit-exact for |x|<8 via NR approximations (ironcalc-calc known failures 1880 -> 1761); |x|>=8 filed as T-0908
- 2026-10-03 — T-0113: new `rustsheet` facade crate with typed ErrorKind/XlError/RefError and single ERROR.TYPE table; eval 0 regressions, 0 fixed.
- 2026-10-03 — steward: health snapshot; filed T-0114..T-0117 (api/harness ready queues), T-0910/T-0911 (IronCalc ports); UPSTREAM.md triaged through 9b1ae3fd. Open PRs #11, #13 are blocked docs-only drafts from other sessions (left untouched).
- 2026-10-03 — T-0910 (partial): TIMEVALUE of "date time" text returns serial fraction as Excel does (ironcalc-calc known failures 1761 -> 1760); fractional seconds still need oracle evidence
- 2026-10-03 — T-0114: API v0 `refs.rs` (SheetId/CellRef/RangeRef/Target, pure A1 parse/print); eval 0 regressions, 0 fixed.
- 2026-10-03 — T-0912: MOD uses exact remainder (fmod) as Excel; ironcalc-calc known failures 1760 -> 1756 (fixed 4, 0 regressions)
- 2026-10-03 — T-0115: API v0 typed Value/Array and engine cell conversion; eval 0 regressions, 0 fixed.
- 2026-10-03 — T-0914 (partial): SINH/COSH/TANH use Excel's exp formulation (ironcalc-calc known failures 1756 -> 1731, fixed 25, 0 regressions); T-0916 (IRR/RATE/XIRR) found not fittable without the oracle; filed T-0917 for the rest
- 2026-10-03 — T-0111: VLOOKUP search column bounded by last used row; 2 POI timeout cases re-included (poi known failures 633 -> 655 from 24 newly scored cells, 0 regressions); TANH 0.9<=|x|<1 uses e^2x form (fixes main's poi G1360 regression); baseline also records ~26 crash-only POI cases (filed T-0118, T-0119)
- 2026-10-03 — T-0911 (ROW/COLUMN part): ROW/COLUMN of multi-cell references return vectors (ported ironcalc b6d34abc); ironcalc-calc 21 newly checked cells, 0 failures, 0 regressions; rest filed as T-0918
- 2026-10-03 — T-0919: @ROW/@COLUMN skip materialising whole-column vectors (165 ms -> 0.17 ms); eval delta none
- 2026-10-03 — T-0918: ported ironcalc INDIRECT (R1C1 + INDIRECT.xlsx) and text-on-arrays fixes; ironcalc-calc +311 newly checked cells with 0 failures, 0 regressions; remaining ports filed as T-0920
- 2026-10-04 — T-0920 (partial): ported POWER-accepts-ranges (daf5f6c6) and implicit-intersection export (9212095d); ironcalc-calc +51 newly checked cells (47 pass, 4 NumericUlp baselined), 0 regressions; finite-number parse and --A1:B4 remain
- 2026-10-04 — T-0118: importer no longer panics on POI workbooks (typed errors, optional style sections); poi known failures 655 -> 647, 0 regressions beyond 4 crash-kind relabels (decision 0012); duplicated lines 8610->8420.
- 2026-10-04 — T-0913: eval snapshot reader decodes rich-value `vm` errors (#CALC!/#SPILL! cached as #VALUE!); ironcalc-calc known failures 1735 -> 1727 (8 fixed), 0 regressions; engine unchanged
- 2026-10-04 — T-0101: removed lifetime from Model/UserModel/Parser/Lexer and the Python binding's leak helper; eval delta none (0 fixed, 0 regressions)
- 2026-10-04 — steward: health snapshot; queues healthy, no new upstream commits, no oracle credentials.
- 2026-10-04 — T-0923: TIME divides before dropping whole days (+7 cells, ironcalc-calc 97.60%); triaged T-0922..T-0929 (no oracle needed)
- 2026-10-04 — T-0102: API v0 `Workbook` read path + xlsx load/save with zip-bomb limits; eval adapter on the facade; eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0924: shared accurate erf/erfc + normal CDF/PDF kernel (ERF*, NORM.DIST/S.DIST, LOGNORM.DIST, GAUSS, Z.TEST); ironcalc-calc known failures 1720 -> 1632 (88 fixed), 0 regressions
- 2026-10-04 — T-0103: API v0 `Workbook::edit`/`set` with undo-log rollback and diff-based `ChangeSet`; eval adapter builds specs via `edit`; eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0104: Python bindings on API v0 (`rs.Workbook`, `edit()` transactions, `XlError`, typed exceptions, GIL released); eval unaffected; filed T-0931 (parser nesting stack overflow).
- 2026-10-04 — T-0928: EXPON.DIST cumulative uses -expm1; ironcalc-calc known failures 1632 -> 1626 (6 fixed), 0 regressions
- 2026-10-04 — T-0930: API edit follow-ups (diff split into edit/diff.rs, sheet-position map, error classifier next to RefError, engine take/restore tests, evaluation counter, bench moved); eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0926: GAMMALN/GAMMALN.PRECISE via libm lgamma; ironcalc-calc known failures 1626 -> 1578 (48 fixed), 0 regressions; 12 cells need oracle
- 2026-10-04 — T-0004 (partial): excel-graph oracle live (app-only, `Sites.Selected` site); fixed site drive addressing; T-0004, T-0106..T-0110, T-0203 unblocked.
- 2026-10-04 — T-0932: eval wall time 72.5 s → 71.1 s across T-0930; edit bench median baseline (100k formulas): recalc 13.4 ms, tracked set 16.5 ms, diff overhead 3.0 ms; KeyHasher deduplicated; eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0009: excel-graph sidecars for probes (3) and probes-numeric (4); probes 71.11% (13 new-coverage known failures), probes-numeric 60.19% (not gated); 0 regressions; excel-semantics: 9 entries VERIFIED (snapping corrected), 4 new; inputs with ≥1E+308 literals fixed (Excel rejects them). T-0106/7/8/10 now ready.
- 2026-10-04 — Owner updated the routine prompts to the text in `docs/process/routines.md` (verified identical). Decision 0014: Excel is always right; every quirk found is recorded in docs/excel-semantics/.
- 2026-10-04 — T-0013 (+ T-0010): every oracle workbook carries an embedded canary (poisoned cached values) and is refused unless Excel recalculated it; canary exposed that Excel-authored workbooks were never saved by the Graph session (old flow returned cached values) — fixed with a post-calculate no-trace edit; Graph calls retry dropped connections (T-0010); eval delta none
- 2026-10-04 — T-0004 (partial): smoke oracle-filled (canary-verified); Excel disproved D-function criteria trimming (engine fixed, criteria.md entry); eval now lets oracle values replace hand values keyed without sheet quotes; smoke 38/38 vs Excel, 0 regressions
- 2026-10-04 — Owner review: filed T-0016..T-0023, T-0402 (model tiers, board conflicts, task ids, milestone-order triage, POI oracle fill, evidence coverage, fuzzing, reviewer models, perf gate); T-0203 and T-0004 raised to P0; critical lane engine, harness, research, perf (API v0 done, api lane empty).
- 2026-10-04 — T-0022: security/architecture/quality reviewers on Sonnet, fidelity/performance pinned to Opus (decision 0015); eval delta none.
- 2026-10-04 — T-0017: `make new-task` assigns collision-free ids (all origin/* branches counted; decision 0016); eval delta none.
- 2026-10-04 — T-0016: docs/tasks/BOARD.md no longer committed (git-ignored, `make board` renders locally; decision 0017); eval delta none.
- 2026-10-04 — T-0023: tasks carry `tier: sonnet | opus`; sonnet (hourly) and opus (2-hourly) workers replace the lane workers; stuck sonnet runs hand off (decision 0018). Needs owner: re-apply docs/process/routines.md. Eval delta none.
- 2026-10-04 — T-0004 done: oracle calibration over ironcalc-calc (71,765 cells, 99.96% bit-exact; 7 day-first-locale cached cells, 4 web-vs-desktop INDIRECT, 18 last-digit float); canary pins en-US; probes/date-text-locale added (probes known failures 13 -> 18, new coverage); filed T-0024
- 2026-10-04 — T-0108: Excel 1900 leap-year serials in DATE/DAY/MONTH/YEAR/formatting; probes known failures 18 -> 15 (A1-A3 fixed), ironcalc-calc 1578 -> 1578, 0 regressions; filed T-0125
- 2026-10-04 — T-0108 follow-up: added probes/dates-early-1900 (oracle-filled, 16 cells, 4 known failures -> T-0125); fixed EOMONTH early serials and DATE(1900,1,0)
- 2026-10-04 — T-0019: excel-graph sidecars for 40 POI workbooks (`oracle_dir`, committed in eval/suites/poi/oracle); poi pending 237 -> 198, known failures 647 -> 959 (new coverage, 0 regressions; 257 web-vs-desktop conflicts left pending, decision 0019); harness now parses JSON floats exactly (serde_json `float_roundtrip`; oracle values had read 1 ulp off, probes-numeric 60.19% -> 68.08%); quirk: SIN/COS at multiples of 90°.
- 2026-10-04 — T-0206: probes/criteria-operands oracle-filled (25 cells); fixed <,>,<=,>= with boolean/error operands and D-function explicit '=' exactness; 0 regressions, probes 25/25 new cells pass
- 2026-10-05 — T-0028: agents check PR mergeability after every push, event and merge to main, and resolve conflicts at once (decision 0020); `make check` runs typos; eval delta none.
- 2026-10-05 — T-0015: spec2xlsx output deterministic (pinned core.xml creation time); eval delta none.
- 2026-10-05 — T-0027: shared-formula members shown shifted in eval show/snapshot (engine `shift_formula_text`); eval delta none.
- 2026-10-05 — T-0203: gen-m2 oracle-filled and gated (30 cases, 12,000 cells, 85.62% bit-exact, 1,726 known failures, new coverage); IF arity fixed in the catalog; T-0208 fixed (oracle `#SPILL!` read by key name; 74 cells); 14 gap tasks T-0209..T-0222; unknown rich errorTypes read as `#UNDECODED!` and stay unscored (poi 951 -> 950, T-0030).
- 2026-10-05 — T-0123: names may contain `\` (en-US lexer, identifier validity); poi known failures 950 -> 935 (15 fixed, 0 regressions).
- 2026-10-05 — T-0031: salvaged stale draft PRs #11/#13/#38 (T-0905, T-0907 restored; financial.md; T-0901/T-0925 findings) and closed them; eval delta none.
- 2026-10-05 — Log frozen here (decision 0024, T-0033); later work: `git log --first-parent main` and the steward's daily summary.
- 2026-10-05 — Owner re-applied the T-0023 routines (sonnet hourly, opus every 2 h); T-0032: sonnet queue buffer (≥10 ready sonnet tasks; `make ready-tasks`, decision 0021).
