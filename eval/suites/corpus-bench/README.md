# corpus-bench

Real-world workbooks from **SpreadsheetBench** (RUCKBReasoning/SpreadsheetBench, CC BY-SA 4.0),
the corpus that witanlabs' **xlsx-corpus-bench** (Apache-2.0) benchmarks xlsx engines on
(`results-sb/manifest.jsonl`). The workbooks are never vendored: `rustsheet-eval fetch --suite
corpus-bench` downloads the dataset tarball pinned by commit and sha256 into
`eval/.cache/suites/corpus-bench`.

Attribution: SpreadsheetBench, Zeyao Ma et al., "SpreadsheetBench: Towards Challenging Real World
Spreadsheet Manipulation" (2024), https://github.com/RUCKBReasoning/SpreadsheetBench, licensed
CC BY-SA 4.0. Benchmark design and manifest: https://github.com/witanlabs/xlsx-corpus-bench.

Subset (the sampling rule, `include` in suite.toml): the first test case's input workbook of each
of the 912 instructions, `<id>/1_<id>_input.xlsx` (912 files). Grow it by adding the `2_`/`3_`
inputs, then the `_answer` files (5,458 in all); the FUSE half of xlsx-corpus-bench is a separate
9.4 GB download (Zenodo record 581678) left for later.

Crash-only: the cached values were recalculated by LibreOffice or win32com, not Excel, so they are
no evidence; a case only fails by a panic, timeout or import error. Oracle values come with T-0427.
CI does not fetch external suites, so the suite runs where it is fetched (the steward, locally);
a run of the subset takes about 2-3 minutes on 8 cores.
