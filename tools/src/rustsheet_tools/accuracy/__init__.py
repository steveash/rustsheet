"""Gate A reference tables (SPEC-0029, decision 0188 R2): the generator behind `rustsheet-accuracy`.

`generate` draws a batch of stratified points for one function (`sampler`), computes each point's
50-digit mpmath value from the function's `Definition` (`defs/`), and writes the batch's case file
and math sidecar under `eval/accuracy/tables/<FUNCTION>/` (`tables`). The oracle fill
(`rustsheet-oracle fill --suite accuracy`) adds Excel's values; `rustsheet-eval accuracy` checks
the engine against both. mpmath runs here only, never in `make check`.
"""
