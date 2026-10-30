# 0171 — Nightly fuzz: capped formula seeds and a minimized corpus cache

- **Status:** accepted (2026-10-30, T-3163). It amends decision 0035.
- **Context:** Nightly fuzz run 25 (2026-10-29) was cancelled at the 45-minute job limit. `fuzz (formula_eval)` was
  still in libFuzzer's initial pass over 802,100 seed files at 380-400 runs a second, and `-max_total_time` does not
  cut that pass short. The seeds are one file per distinct formula in eval/suites: 813,249 on 2026-10-30, 589,147
  of them from `probes-numeric`, mostly one generated shape with other numbers. A cancelled job saves no corpus, so
  every later night would fail the same way.
- **Decision:**
  - `fuzz/seed_corpus.py` writes one seed per formula shape (numbers and text literals replaced; 317,305 shapes
    on 2026-10-30), then keeps the 20,000 whose sha256 sorts first (`MAX_FORMULA_SEEDS`). That is about a minute
    of first pass at formula_eval's rate. The dictionary still lists every function name from all the formulas.
  - After fuzzing, the job runs `cargo fuzz cmin` (libFuzzer `-merge=1`) on the corpus. The saved corpus then holds
    only inputs that add coverage, so it is bounded by the target's coverage, not by the number of nights.
  - The cache key gains a `v2` prefix, so the 802,100-file caches saved before this change are never restored.
- **Consequences:** The first night after the merge starts from the capped seeds. Its first pass plus 20 minutes of
  fuzzing plus the minimization fits in the 45-minute limit with room for the build. A seed shape outside the
  sampled 20,000 is still reachable by mutation and the dictionary, not as a seed. Raising the cap is a one-line
  change if a run shows time to spare.
