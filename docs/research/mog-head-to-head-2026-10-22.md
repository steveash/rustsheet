# RustSheet vs mog: parity and performance head-to-head (2026-10-22)

Owner request. [mog](https://github.com/fundamental-research-labs/mog) 1.0.0 is a headless Rust
spreadsheet engine with an Office.js CLI. Its verification corpus is
[Calipers](https://github.com/fundamental-research-labs/calipers) (commit 9e47ea4f), Excel-saved
goldens. This note records how both engines did on both corpora and what it led to: T-0446 (the
`calipers` suite), T-0447 (iterative calculation), T-0448 and T-0449 (range-heavy recalculation), and
T-0441 raised to P2 (data tables).

## Method
- rustsheet `main` at e7b3d29; mog built with `cargo build -p mog --release` (full LTO). Both ran on one
  idle 4-core Linux box, one process per workbook.
- Values: rustsheet-eval got a temporary `mog` engine (not merged). It writes each case's xlsx (spec
  cases through `spec2xlsx`, the oracle's builder), runs `mog -i in.xlsx -r -o out.xlsx`, and reads
  mog's cached values with the snapshot reader used for Excel's files. mog writes shortest
  round-trip doubles, so bit-exact comparison is fair. Volatile cells are skipped as usual.
- Scoring: the closeness buckets of crates/rustsheet-eval/src/closeness.rs. Only cases both engines
  evaluated, and only formula cells, are compared.
- Function recognition: `=FN(1,…)` at minimum arity for the 527 names in
  eval/catalog/excel-functions.tsv, counting `#NAME?` as unrecognised.

## Calipers (mog's corpus)
Calipers' own `verify` passes a case only when values match to 15 significant digits and formula
text, styles, names, merges and panes all match Excel's save. On pv_goldens (107 financial models,
caches stripped, full recalculation) mog passed 18 and rustsheet 2. Rustsheet fails mostly on file
fidelity: it writes `MATCH(a,b,0)` where Excel kept `MATCH(a, b, 0)`, and it normalises style
details. mog passed all 83 round trips, rustsheet 61. Rustsheet cannot run the 263 Office.js cases.

Values only, 132 cases both loaded, 1,563,825 formula cells:

| | bit-exact | ≤4 ulp | ≤15 digits | visibly off: numeric | visibly off: other | cases all ≤15 digits |
|---|---|---|---|---|---|---|
| rustsheet | 99.55% | 99.68% | 99.68% | 658 | 4,422 | 72.0% |
| mog | 96.16% | 99.04% | 99.33% | 9,845 | 603 | 47.0% |

- Rustsheet refuses 21 workbooks with what-if data tables (19 pv_goldens models,
  roundtrip/i18n_hebrew, roundtrip/lbo_model), which T-0441 owns. It also gives `#CIRC!` for about
  3,600 cells in two models saved with `iterate="1"`, which T-0447 owns.
- 78% of mog's non-exact numbers are 1e-16 to 1e-15 relative. They are summation-order differences,
  but some still flip the 15th digit, e.g. `PMT` 7985.22164230558 against Excel's
  7985.221642305569. Semantic misses: `XIRR`, `COUPPCD`/`YEARFRAC` coupon dates, `GETPIVOTDATA`, and
  `INDIRECT` to a sheet-scoped name.

## RustSheet's corpus
All 16 suites, 2,196 cases both engines evaluated, 530,312 formula cells:

| | bit-exact | ≤15 digits | cases all ≤15 digits | real-world corpus-bench cases all ≤15 digits |
|---|---|---|---|---|
| rustsheet | 90.67% | 95.36% | 87.7% | 97.2% |
| mog | 64.44% | 70.96% | 31.2% | 86.4% |

mog lost 121 cases:
- 66 crashes, aborts or kills (a panic at `compute-functions/src/datetime/week.rs:210` on unusual
  date serials, a 123 GB allocation abort);
- 45 spec cases that need sheet edits after load (an adapter limit, not mog's);
- 8 corrupt-zip or strict-OOXML refusals;
- 2 outputs that the snapshot reader's XML-bomb limits rejected.

Mog recognises 501 functions and rustsheet 508. Mog lacks GROUPBY, PIVOTBY, FORECAST.ETS* and
FILTERXML, and only mog has EUROCONVERT and JIS. Our corpus targets quirks rustsheet has been fixed
against, and Calipers' Office.js suites target mog, so each corpus favours its own engine. The real
workbooks (pv_goldens, corpus-bench, poi) are the neutral evidence.

## Performance (load + full recalculation + save, process wall time and peak RSS)

| workbook | rustsheet | mog |
|---|---|---|
| 500k-cell chain | 3.8 s, 230 MB | 24.9 s, 3,558 MB |
| 300k arithmetic grid | 4.2 s, 129 MB | 10.4 s, 2,087 MB |
| 15k exact lookups into 20k rows | 43.4 s, 37 MB | 1.2 s, 213 MB |
| 2k SUMIFS + 2k COUNTIFS over 50k rows | 20.2 s, 54 MB | 2.4 s, 404 MB |
| 5k SUM+AVERAGE over growing ranges | 18.0 s, 52 MB | 2.2 s, 346 MB |
| 200k text formulas | 4.6 s, 187 MB | 9.3 s, 1,747 MB |

On 994 real workbooks (88 pv_goldens, 906 corpus-bench) rustsheet is faster per file by a geometric
mean of 1.75x, and faster on 941 of them. Its range-heavy cliffs, however, reach 12–270 s where mog
takes 0.02–2.5 s:
- corpus-bench 52450, 59932, 59969, 45896, 43227;
- 17049 and 41978, which exceeded 300 s;
- pv26_002 and pv26_2_002 take about 11 minutes in both engines, but mog peaks at 12.2 GB there
  against rustsheet's 0.66 GB.

Every lookup and criteria call rescans its range (T-0448, T-0449, T-0430).

## Rerun after data tables (2026-10-23, T-0441)
Same method, rustsheet at a0141913 with data tables. The calipers suite's own goldens are compared
separately, and are left out of "our corpus" below.

Calipers (153 cases both evaluated, 1,620,357 formula cells):

| | bit-exact | ≤15 digits | visibly off | cases all ≤15 digits |
|---|---|---|---|---|
| rustsheet | 99.33% | 99.45% | 0.55% | 70.6% |
| mog | 95.75% | 99.26% | 0.74% | 40.5% |

- Rustsheet now loads all 107 financial models and all round trips. The data-table results of the
  21 workbooks with tables match Excel bit for bit.
- Its visibly-off cells are almost all 7,572 `#CIRC!` cells in 19 models saved with
  `iterate="1"` (T-0447). Most of those models loaded only once data tables worked.
- Mog's two missing models, pv26_002 and pv26_2_002, hit the adapter's 120 s per-workbook limit.
  Both engines need about 11 minutes each.

Our corpus (2,229 cases, 542,961 formula cells): rustsheet 89.74% bit-exact and 95.23% within 15
digits, mog 63.58% and 70.43%. Case-weighted, 87.8% against 30.7%. Mog lost 141 cases:
- 66 crashes;
- 65 that need post-load edits, more than before because the branch gained probes with sheet ops;
- 8 file refusals;
- 2 outputs the reader rejected.

Performance on the 107 financial models: rustsheet is faster on 90, geometric mean 1.84x. The
data-table models are quick, e.g. pv23_046 (four 9x9 tables) takes 0.14 s against mog's 0.78 s.
Over all 1,013 real workbooks timed, the geometric mean is 1.78x.
