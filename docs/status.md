# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

## Snapshot — 2026-10-12 (M1 in progress)

| Suite | Cases | Cells | Bit-exact | Known failures |
|---|---|---|---|---|
| ironcalc-calc | 217 | 72,203 | 97.95% | 1,478 |
| poi | 336 | 27,290 | 97.98% (204 cases pending: no formulas, unreadable, or web-vs-desktop conflicts) | 566 |
| gen-m2 | 30 | 12,000 | 99.81% | 23 |
| gen-m1 | 15 | 3,750 | 99.95% (M1 exit needs 100%; 2 cells left, both negative non-integer exponents, now T-1421) | 2 |
| probes | 471 | 18,037 | 84.02% (4 accepted deviations, 11 cells passed by decision 0042) | 2,881 |
| smoke | 6 | 46 | 100% | 0 |
| fixtures | 6 | 67 | 100% | 0 |
| gen-m5 | 10 | 400 | 97.75% | not gated |
| gen-m6 | 25 | 1,000 | 97.30% | not gated |
| gen-m7 | 25 | 994 | 93.26% | not gated |
| gen-m9 | 25 | 998 | 89.68% (new suite, T-1289) | not gated |
| probes-numeric | 31 | 10,571 | 58.57% | not gated |

`make eval-check` on `main` (c7e1d618) shows 0 regressions and 0 baseline fixes in every suite.
CI on `main`: green on 9ad32ae (T-1269), the last completed run. The runs on 1337fe2 and c7e1d618 were still in progress when I checked; the T-1344 run was cancelled by the next push.

**Frozen M1 probe list (`eval/milestones/m1-probes.txt`) is at 100%:** 89 cases, 1,858/1,858 cells bit-exact, since T-1222 (#528) landed.

The probes known-failure jump (511 → 2,881 F lines in eval/baseline/probes.txt since 296c2097; the last snapshot's 552 counted failing cells in the report instead) is new coverage, not regressions. It comes from 99 new probe cases, mostly the M9 argument-mode probes and GROUPBY/PIVOTBY:
- groupby-pivotby: 1,431
- stat-argument-modes: 403
- fin-argument-modes: 306
- eng-argument-modes: 119
- db-argument-modes: 38

None of it is M1 scope. Under decision 0051 (#611, merged during this run), each probe case counts toward its own milestone, and M2 exit counts only 74 of the failing probe cells.

Hygiene: lint-debt 99 allow entries · duplicated lines 6,751 (ratchet file still 7,158) · oversized files 15 (ratchet file 15).
`quality/` is protected, so the next PR that touches it records the duplication drop.

Since the 2026-10-10 snapshot (296c2097), 84 PRs merged to `main` through 448c0cee: 82 tasks and decisions 0050 (#609) and 0051 (#611). None was reverted. No steward run happened on 2026-10-11.
- T-0124, T-0219, T-0222, T-0258, T-0283, T-0297, T-0299, T-0962
- T-1080, T-1084, T-1085, T-1086, T-1093, T-1100, T-1105, T-1131, T-1169, T-1181, T-1187, T-1189
- T-1211, T-1222, T-1228, T-1239, T-1244, T-1251, T-1256, T-1260, T-1264, T-1267–T-1269, T-1274–T-1276, T-1283, T-1285, T-1289
- T-1294, T-1300, T-1302–T-1305, T-1309, T-1311–T-1314, T-1320, T-1321, T-1329, T-1330, T-1332, T-1334–T-1346
- T-1362–T-1365, T-1368–T-1370, T-1375, T-1378, T-1380, T-1386, T-1388, T-1398, T-1414, T-1415

Blocked (18): the Phase 2 tasks T-0400/T-0401/T-0402 and T-1066 (decision 0027); T-1099 (GEOMEAN last ulp); the M14
desktop-oracle tasks T-0171, T-0198, T-1016, T-1051, T-1074, T-1075, T-1081, T-1174, and the newly blocked T-1204, T-1277, T-1299 (T-1414), T-1310 and T-1367
(decision 0039; `rustsheet-oracle status` still shows excel-desktop n/a, so none can be unblocked).

**Nightly fuzz is red on two new findings, both filed as P0 and both reproduced on 1337fe2.**
- Run 38109900859 (2026-10-11, 39790b9), `formula_eval`, OOM on `Data!$D:$D >Data!$4:44`.
  - I narrowed it to `=D:D+4:4`: a binary operator broadcasts a whole column against a whole row into a 2^34-cell array without `check_array_size`. Under `ulimit -v 4000000` it aborts.
  - Filed as **T-1420** (engine, M1, P0, opus).
- Run 38202077837 (2026-10-12, eba0eb0), `formula_eval`, crash on `…DATE(Data!B8,1000000000000000.0,1)…`.
  - I narrowed it to `=DATE(2000,1000000000000000,1)`. It panics in chrono (`NaiveDate + Months` out of range, `formatter/dates.rs:167`), so release builds abort too.
  - Filed as **T-1419** (functions-date, P0, opus).
- `formula_parse` and `xlsx_import` passed both nights.

Open PRs: 17. Eight non-draft PRs are over 24 h old:
- #461 (T-0953, about 74 h old) was green and approved, but merging `main` broke its test build: `format_number` now takes a `FormatContext`. Someone also ran GitHub's update-branch on it at 06:15 UTC, and that merge carries the same break. I fixed the test, merged both, and `make check` passed locally. Pushed 4064183a and commented; not merged.
- #520 (T-1235, about 55 h old) conflicted with `main` in the probes baseline and its task file. I merged `main`, kept both sides and regenerated the baseline. `make check` passed locally (probes 2,880 → 2,815, the PR's 65 fixes). Pushed 4c21b7e5 and commented; not merged.
- #330 (T-0411, about 106 h old) and #504 (T-1214 BAHTTEXT, about 63 h old) are blocked by the oversized-file ratchet. `main` keeps `functions/mod.rs` exactly at its ratchet value (2,363 lines), so any added function fails `hygiene`. #504 also conflicts with `main`. I resolved #504 locally, but pushed neither, to avoid red heads. Filed **T-1425** (quality, sonnet, P1): move `Function::to_xlsx_string` (181 lines) out of `functions/mod.rs`. Commented on both PRs.
- #499 (T-1131) and #513 (T-0299) are superseded: both tasks already landed on `main` (63168168, #579). Commented that they can be closed.
- #517 (T-0298) duplicates #586, which is newer and merges cleanly. #524 (T-1225) went stale, and a worker claimed T-1225 again on a new branch at 05:49 UTC today. I commented on both and left them alone.
- #314 (T-1100), #409 (T-1155) and #436 (T-1186) are drafts whose tasks are blocked on their branches. No action.
- The other 6 PRs are less than 24 h old (#551, #553, #586, #607, #608, #610).

Queues at the start of the run:
- functions: 106 ready (T-1419 and T-1425 added during the run). The sonnet buffer is 68 ready, so the gap-triager did not run.
- engine: 0 ready. The lane was busy with T-1370 (merged as #612 during this run), with 8 unclaimed M1 todos, P0 T-1420 among them.
- research, harness and api: 0 ready and no open M1 tasks. quality 7, perf 8.
- The planner ran for M1. It confirmed that T-0221 owned the last 2 gen-m1 cells (A41 `=3^-0.5`, A207 `=0.5^-0.5`). It split them into **T-1421** (engine, M1, P1, opus: probe and fit negative non-integer exponents), so they are no longer queued behind M2 sorting. It found no M1 scope missing for research, harness or api.

Upstream IronCalc: no new commits past 8a82396a.

Oracle: excel-graph OK. The fill on ironcalc-calc wrote 0 sidecars (214 skipped). The fill on poi wrote 0 (292 skipped, the same 4 workbook failures as T-0019).

Spot-audit: none this run (not yet due). The last audit, 2026-10-09 (steward PR #466), re-reviewed #464, #448 and #453 on opus. The next audit is due 2026-10-16.

**Critical lane:** engine, research, harness

(Ordered list read by both tier workers, which then fall back to `functions`.
- M1 exit needs gen-m1 at 100%; the frozen M1 probe list is already at 100%.
  - gen-m1 has 2 last-digit cells left, both in T-1421 (engine).
  - engine also carries the fuzz P0 T-1420.
- research and harness have no M1 exit scope left. They stay on the line only for their M2 probe and harness work; api has none.
- The fuzz P0 T-1419 (functions-date) is reached through `functions`, P0 first. perf stays out: Phase 2.)

## Needs owner

- **Replace step (4) of the steward routine prompt** with the text in decision 0050's "Owner action" section (docs/decisions/0050-…). docs/process/routines.md already carries the new text, but the live routine still says "do not merge PRs opened by another session". This run followed the live prompt, so green and approved orphans such as #461 stay unmerged until the prompt changes. Agents never edit routines (decision 0008).

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
