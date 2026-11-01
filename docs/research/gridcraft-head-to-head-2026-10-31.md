# RustSheet vs gridcraft: measured head-to-head (2026-10-31)

Owner request on issue #2277, following the read-only comparison
(gridcraft-comparison-2026-10-29.md), which left the measured run to the owner because it means
building and running third-party code (decision 0158). The owner asked for it on 2026-10-31. The
method is the one in mog-head-to-head-2026-10-22.md.

## Method
- RustSheet `main` at d0ddf26. [gridcraft](https://github.com/storytold/gridcraft) at 67e39fae
  (2026-10-31, v0.14.2), built with `cargo build --release -p gridcraft-cli`. That is two days newer
  than the commit the first note read (2ff75179). Both ran on one 4-core, 15 GB Linux container.
- **Values.** rustsheet-eval got a temporary `gridcraft` engine, which is not merged. An xlsx case
  is written out with the cached values of its formula cells removed, so a formula gridcraft skips
  cannot pass by keeping Excel's value. A spec case is built with `spec2xlsx`, the oracle's
  builder. `{"entered": ...}` cells are typed in with gridcraft's `cell.set` command. The adapter
  then runs `gridcraft-cli run --in in.xlsx [--script steps] --out out.xlsx`, which recalculates
  everything on open (`DocState::new` → `recalc_all`). It reads gridcraft's output with the
  snapshot reader that reads Excel's files. Volatile cells are skipped as usual.
- **External links.** gridcraft does not follow links to other workbooks. It keeps the saved value
  of a formula that names one (`[1]Sheet!A1`), and Excel reads its link cache. The adapter therefore
  keeps the cache of those formulas only. Removing it as well cost gridcraft about 18,000 corpus-bench
  cells. Values that reach a link through a defined name still come out `#REF!` in gridcraft.
- **Scoring.** The closeness buckets of crates/rustsheet-eval/src/closeness.rs. Only cases both
  engines evaluated count. Per-function rates use the attributed cells, as docs/progress.md does.
- **Recognition.** `=FN(1,…)` at minimum arity for the 527 names in
  eval/catalog/excel-functions.tsv, counting `#NAME?` as unrecognised.
- **Performance.** Load, full recalculation and save, one process per workbook. Wall time and peak
  RSS come from `wait4`, with a 300 s limit. RSS has a floor of about 15.5 MB, the forked Python
  parent, so small workbooks read as ties.

## Values: all 17 suites
3,773 cases both evaluated, 2,870,621 scored cells:

| | bit-exact | ≤4 ulp | ≤15 digits | visibly off: numeric | visibly off: other | cases all ≤15 digits |
|---|---|---|---|---|---|---|
| rustsheet | 90.61% | 96.42% | 97.46% | 71,305 | 1,582 | 89.0% |
| gridcraft | 77.56% | 81.22% | 83.39% | 376,410 | 100,211 | 33.3% |

probes-numeric holds 674k of those cells. It sweeps numeric kernels into hard regions on purpose,
and RustSheet is only at 89.6% there itself. Without it, over the real-world and generated suites:

| suite (cases, cells) | rustsheet ≤15 digits | gridcraft ≤15 digits | rustsheet cases all ≤15 | gridcraft cases all ≤15 |
|---|---|---|---|---|
| calipers (172, 1,500,225) | 99.99% | 98.14% | 94.2% | 59.3% |
| corpus-bench (320, 205,366) | 100.00% | 90.57% | 99.4% | 92.8% |
| poi (117, 27,206) | 99.65% | 98.67% | 97.4% | 94.0% |
| ironcalc-calc (223, 73,235) | 99.94% | 96.14% | 93.7% | 53.4% |
| gen-m1 … gen-m10 (630, about 238,000) | 99.41% | 87.02% | 71.1% | 7.5% |
| probes (1,990, 151,678) | 99.42% | 53.50% | 96.8% | 26.9% |
| fixtures (25, 348) | 100.00% | 23.12% | 100.0% | 40.0% |

gridcraft lost 107 cases that RustSheet evaluated, and they are left out above:
- 74 probes with `sheet_ops` and 2 with R1C1 entry, which the adapter cannot replay (an adapter limit, not gridcraft's);
- 28 probes where gridcraft's `cell.set` refuses a typed formula ("There's a problem with this formula"), and `run` stops at the first failing step;
- 3 calipers models over the 120 s case limit: pv23_042, pv26_002 and pv26_2_002.

What the gaps are, from the mismatch signatures:
- **Real workbooks.** corpus-bench is mostly external links. Defined names and `INDEX` over
  `'[2]Sheet'!A:A` give `#REF!`, where Excel uses its link cache (15,000 cells). In calipers,
  20,606 of the 27,898 visibly-off cells are in one round-trip workbook, formulas_nested, a 4,000-row
  chain through `MOD(ABS(x),n)`. gridcraft's MOD returns a cleaned result where Excel keeps the
  floating-point one: `MOD(ABS(F13),7)` is 1.7333 in gridcraft and 1.7332999999999998 in Excel. Down
  the chain the difference grows past the 15th digit. The rest are 15th-digit summation
  differences in the financial models, a legacy implicit intersection over
  `BI11:BI53` that gives `#VALUE!`, and `TEXT(name,"")` over a defined name that comes out empty.
- **Probes.** Numeric kernels are the biggest share (ACCRINT, COMBIN overflow, ATANH near 1). Then
  `FORMULATEXT`, which prints 3D spans as `Data1:Data1!A1`, 3D references inside
  `INDEX`/`ROW`/`AREAS`/`ISREF`, spills that `FORMULATEXT(INDIRECT({…}))` does not produce, and
  argument-error order (`#NUM!` where Excel gives `#VALUE!` for boolean dates).

## Functions
Over the 506 in-scope functions, counting a function when at least 99.9% of its attributed cells
are within 15 digits:

| | ≥ 99.9% | 100% |
|---|---|---|
| rustsheet | 421 | 413 |
| gridcraft | 17 | 11 |

(421 is lower than progress.md's 434 because this run counts every suite and leaves out the
desktop-owned exceptions.) gridcraft is ahead on five functions: FORECAST.ETS.SEASONALITY 81% to 9%,
FORECAST.ETS.STAT 73% to 4%, FORECAST.ETS.CONFINT 62% to 15%, FORECAST.ETS 69% to 22%, and CELL 91%
to 79%. Its FORECAST.ETS family (`crates/functions/src/ets.rs`) is the one implementation worth reading,
as an input to future FORECAST.ETS work. Licence and provenance rules apply, as in the first note.
Its weakest functions are JIS and PHONETIC (both 0%: Excel en-US gives `#NAME?` and `#N/A`), ENCODEURL,
the Bessel functions, PIVOTBY, FREQUENCY, F.DIST/T.DIST.RT, YIELD, RATE and XIRR.

Recognition: gridcraft knows 514 of the 527 names and RustSheet 508. gridcraft alone has the 7 CUBE
functions, EUROCONVERT, WEBSERVICE and JIS. RustSheet alone has FIELDVALUE, GETPIVOTDATA, IMAGE and
STOCKHISTORY. gridcraft now has ECMA.CEILING, which the first note found missing at 2ff75179. A known
name is not a right value: Excel en-US gives `#NAME?` for `JIS`, so gridcraft is wrong there by
recognising it.

## Performance
Load, full recalculation and save:

| workbook | rustsheet | gridcraft |
|---|---|---|
| 500k-cell chain | 3.7 s, 235 MB | 6.4 s, 519 MB |
| 300k arithmetic formulas (100k rows × 3) | 3.2 s, 155 MB | 5.4 s, 354 MB |
| 15k exact VLOOKUPs into 20k rows | 0.5 s, 39 MB | 88.2 s, 42 MB |
| 2k SUMIFS + 2k COUNTIFS over 50k rows | 0.6 s, 56 MB | 140.3 s, 53 MB |
| 5k SUM+AVERAGE over growing ranges | 1.0 s, 16 MB | 2.9 s, 24 MB |
| 200k text formulas | 2.8 s, 173 MB | 5.3 s, 459 MB |

(The synthetic workbooks match the shapes of the mog note. They are not the same files, so do not
compare them with that note's numbers.)

- **pv_goldens** (107 financial models). gridcraft did not finish pv26_002 and pv26_2_002 in 300 s
  (1.4 GB when stopped). RustSheet takes 44 s and 41 s at 921 MB. On the other 105, RustSheet is
  faster on 64 and slower on 25, with a geometric mean of 1.35x and 43 s against 236 s in total.
  gridcraft's worst is pv23_042, 108 s against 2.1 s.
- **corpus-bench** (912 workbooks). Both open every file. RustSheet is faster on 186 and slower on
  117, and the rest are ties at timer resolution. The geometric mean is 1.05x, 29 s against 36 s in
  total. RustSheet's slowest is 55392 (3.6 s, gridcraft 2.5 s). 46444 takes RustSheet 0.75 s and
  156 MB, against 0.12 s and 21 MB.

gridcraft's interval index for range dependents (first note, idea 1) helps incremental recalc. A
full recalc of lookup and criteria formulas, though, rescans the range for every formula, which is
where RustSheet was before T-0448 and T-0449.

## What follows from it
- Nothing in gridcraft's results calls for a change to RustSheet's Excel behaviour. Every case
  scored here was scored against Excel's own values.
- One lead: gridcraft's FORECAST.ETS family scores higher than RustSheet's on the same probes.
  Reading it is an input for future FORECAST.ETS work, under the licence and provenance review the
  first note requires.
- The adapter (crates/rustsheet-eval/src/gridcraft.rs, an engine switch in `execute`, and the
  case path passed to the worker thread) and the scripts were left out of the repository on
  purpose, as for mog.
