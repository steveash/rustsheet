# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

## Snapshot — 2026-10-15 (M2 is current)

| Suite | Cases | Cells | Bit-exact | Known failures |
|---|---|---|---|---|
| ironcalc-calc | 217 | 72,203 | 98.00% | 1,447 |
| poi | 336 | 27,290 | 98.18% (204 cases pending: no formulas, unreadable, or web-vs-desktop conflicts) | 512 |
| gen-m2 | 30 | 12,000 | 99.99% | 1 |
| gen-m1 | 15 | 3,750 | **100%** | 0 |
| probes | 610 | 27,220 | 79.44% (4 accepted deviations, 11 cells passed by decision 0042) | 5,596 |
| smoke | 6 | 46 | 100% | 0 |
| fixtures | 14 | 203 | 98.03% | 4 |
| gen-m5 | 10 | 400 | 99.25% | not gated |
| gen-m6 | 25 | 1,000 | 99.10% | not gated |
| gen-m7 | 25 | 994 | 95.57% | not gated |
| gen-m9 | 25 | 998 | 91.88% | not gated |
| probes-numeric | 111 | 16,113 | 52.58% (no timeouts since T-1518) | not gated |

`make eval-check` on `main` (6e51f2bf) shows 0 regressions and 0 baseline fixes in every gated suite.
CI on `main` is green: every CI run on `main` since the last snapshot passed, through 6e51f2bf (T-1577). One run, a7e4b17c, was cancelled because a newer push superseded it.

Where M2's exit stands:
- gen-m2 has 1 cell left (was 6): `fa437879253827be` Calc!A23, `(0.2^-0.5)^99`, covered by T-1495 (engine). T-0220 (collation) merged.
- M0–M2 probes: 292 cases, 8,589 cells, 8,415 bit-exact, 4 deviations, 170 known failures (was 518; T-1470 moved the wrongly tagged cases to their own milestones). Every failing cell has an open task:
  - `text-collation-unprobed` (89) and `text-collation-lookups` (11): T-1594 (engine). It merged during this run as #820 (178420d9), after these numbers were taken.
  - `text-lone-surrogates` (47), `text-surrogate-positions` (6), `concatenate-limit-spill` (4): the SPEC-0010 rollout, T-1579, T-1580, T-1581, T-1585, T-1586 and T-1587.
  - `aggregate-picked-direct-a-variants` (9): T-1571, T-1572, T-1573 (engine).
  - `array-context-legacy-sumproduct` (4): T-0294 (engine).
- The frozen m1-probes list is at 0 failures.
- ironcalc-calc cells using only M1/M2 functions: **3,977 of 3,977 bit-exact, criterion met** (T-1475 fixed the TBILL cells).
- Duplication is at 6,360 lines, 30.5% below the fork's 9,153 (target ≥ 15%), so the number is met. "One implementation of aggregate-argument iteration" waits on T-1476 and T-1567 (quality, sonnet); T-1473, T-1474 and T-1477 merged.
- NOW/TODAY clock: T-1479 (engine) and T-1480 (api) merged, so SPEC-0008 is done.

Hygiene: lint-debt 91 allow entries (= last run) · duplicated lines 6,360 (ratchet 6,369) · oversized files 14 (ratchet 15).
Both ratchets can tighten (`crates/rustsheet-xlsx/src/import/worksheets.rs` is still no longer oversized). The steward left them as they were, because `quality/` is a protected path; the next quality PR should run `--update`.

Since the 2026-10-14 snapshot (steward PR #731, b1dd04d0), 86 PRs merged to `main` through 6e51f2bf: 84 task PRs and 2 task-filing PRs (#735, #737). None was reverted.
- T-0055, T-0220, T-0407, T-0409, T-0419, T-0424, T-0960
- T-1201, T-1202, T-1255, T-1257, T-1261, T-1270, T-1287, T-1288, T-1293, T-1306, T-1318, T-1357, T-1358
- T-1410, T-1411, T-1418, T-1433, T-1460, T-1465, T-1470, T-1473–T-1475, T-1477, T-1479, T-1480, T-1483, T-1487
- T-1500, T-1502, T-1504, T-1507–T-1510, T-1512, T-1513, T-1518–T-1520, T-1522–T-1527
- T-1530, T-1531, T-1533, T-1534, T-1537, T-1539–T-1541, T-1546–T-1551, T-1555, T-1556, T-1558–T-1562
- T-1566, T-1568, T-1575–T-1577, T-1582, T-1583, T-1588, T-1590, T-1592

Blocked: 24 (was 18). None can be unblocked.
- The Phase 2 tasks T-0400, T-0401, T-0402, T-0423 (new) and T-1066 (decision 0027).
- T-1099 (GEOMEAN last ulp).
- New, last-digit fits with no model that fits (decision 0056, time-boxed): T-0968 (normal CDF and ERFC), T-1528 (PMT/IPMT/PPMT/NPV), T-1529 (GAMMALN), T-1592 (YIELD Newton start).
- The M14 desktop-oracle tasks T-0171, T-0198, T-1016, T-1051, T-1074, T-1075, T-1081, T-1174, T-1204, T-1277, T-1299, T-1310, T-1367 and T-1553 (new) (decision 0039; `rustsheet-oracle status` still shows excel-desktop n/a).

**Nightly fuzz:** run 38678152308 (2026-10-15, 6b35f595) failed on `formula_eval`.
- The input is `RANK(5,@IF(A1:B2>0,A1:ANK))`. It hits the debug assertion "Larger-than-1x1 array reached scalar-context cell" (`model.rs:926`, 2x2).
- This is T-1303's crash with an explicit `@` in front of the IF. T-1303's fix covers only the form without `@`, which gives `#VALUE!` with no panic.
- I reproduced it on 6e51f2bf with a debug engine unit test.
- Filed **T-1596** (functions-stat, P0, opus).
- `formula_parse` and `xlsx_import` passed.

Open PRs: 9 at the start of the run, 5 now.
- Under 24 h: #819 (T-1593).
- I closed 4 superseded PRs with a comment linking each survivor. None of their branches had moved for 2 h or more, and no branches were deleted:
  - #661 (T-1450, merged as #725);
  - #644 (T-1396, merged as #711);
  - #551 (T-1331, done through #581);
  - #314 (T-1100, merged as #541).
- Blocked drafts, each with a `status: blocked` task on its branch: #698 (T-1486), #553 (T-1284), #436 (T-1186), #409 (T-1155).
  - #698 needs only one more `reviewer-excel-fidelity` pass. I merged `main` into it locally: the only conflict was the baseline, with fixed 5 and 0 regressions.
  - `make check` then fails the hygiene gate. `functions/text/common.rs` reaches 1,002 lines, over the 1,000 limit.
  - I did not push, and I commented the blocker and the remaining steps on the PR.
- No orphan PR met every decision 0050 condition, so none was merged.

Queues at the start of the run:
- functions: 79 ready. The sonnet buffer is 58, so the gap-triager did not run.
- engine 0, harness 0, research 0, api 1 (T-1578), quality 4, perf 14. The planner ran for M2:
  - engine is busy, not empty: T-1594 was pushed today, and 11 more unclaimed M2 tasks have their dependencies met.
  - research is busy: T-1516 and T-1049 are claimed.
  - harness has no M2 work left: gen-m2 has 12,000 cells, and the M0–M2 report rows and T-1519 are done.
  - api: T-1578 is all of SPEC-0010's api scope.
  - The planner filed **T-1597** (research, sonnet): probe collation equality in SUMIF, SUMIFS and COUNTIFS criteria and in wildcard criteria. T-1590 probed only COUNTIF, and T-1594 changes the shared criteria code.

Upstream IronCalc: 2 new commits through 77237187. Both only refresh IronCalc-authored template workbooks (`xlsx/tests/templates`, 8c0d91e0 and 77237187). These are not Excel-authored evidence, so I filed no task.

Oracle: excel-graph OK.
- The fill on ironcalc-calc wrote 0 sidecars (214 skipped).
- The fill on poi wrote 0 sidecars (292 skipped, the same 4 workbook failures as T-0019).

Spot-audit: none this run (not yet due). The last audit, 2026-10-09 (steward PR #466), re-reviewed #464, #448 and #453 on opus. The next audit is due 2026-10-16.

**Critical lane:** engine, quality, research, api, harness

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M2.
- engine holds most of M2's remaining failures:
  - T-1495 (the last gen-m2 cell);
  - T-1571–T-1573 (aggregate variants);
  - T-1580 (concatenation truncation);
  - T-0294 (legacy SUMPRODUCT).
- quality: T-1476 and T-1567 close "one implementation of aggregate-argument iteration".
- research: T-1597 (criteria collation evidence).
- api: T-1578 (SPEC-0010).
- harness: no M2 task.
- The lone-surrogate text tasks (T-1579, T-1581, T-1585–T-1587) and the P0 T-1596 are reached through `functions`. perf stays out: Phase 2.)

## Needs owner

- **Sync the live steward routine prompt with docs/process/routines.md.** The 2026-10-15 run's live prompt now has decision 0050's step (4), so that item is resolved. But it still opens with "On an up-to-date `main`". routines.md says instead to work on a branch cut from a freshly fetched `origin/main` and never to check out, pull into or reset a local `main` (decision 0055). With the old wording, this run first tried to reset a stale local `main`. Agents never edit routines (decision 0008).

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
