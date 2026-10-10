# 0046 — Fuzz job ASan options: no allocation stack traces

- **Status:** accepted (2026-10-09, T-1220)
- **Context:** The nightly `formula_parse` job (decision 0035) stopped with libFuzzer's
  out-of-memory error (2053 MB of 2048 MB) after 2.5M runs, with only 46.6 MB of live heap. Local
  15-minute runs of the same target and options (T-1220 Outcome) separated the causes:
  - Default ASan: RSS rose steadily, 1000 MB at 0.2M runs to 1662 MB at 2.47M.
  - ASan with `quarantine_size_mb=16`: RSS still rose, 358 to 1358 MB over 3.07M runs.
  - No sanitizer: RSS stayed flat at 45 to 60 MB over 6.28M runs. The engine does not leak.
  - ASan with `malloc_context_size=0`: RSS stayed flat at 688 MB from 0.2M to 2.6M runs, and the
    exec rate doubled.
  The growth is ASan's stack depot. It stores every distinct allocation stack trace for the life
  of the process, and a recursive-descent parser fed fuzzed input produces new ones without end.
- **Options considered:**
  - *Raise `-rss_limit_mb`.* This only postpones the stop, and it would hide a real leak too.
    Rejected.
  - *Shorter runs.* Less coverage per night, and the growth rate is unchanged. Rejected.
  - *Smaller quarantine.* Measured: it does not stop the growth. Rejected.
  - *`ASAN_OPTIONS=malloc_context_size=0` on the fuzz step.*
- **Decision:** The fuzz step in `.github/workflows/fuzz.yml` runs with
  `ASAN_OPTIONS=malloc_context_size=0` for every target. Limits, run time, permissions, triggers
  and secrets are unchanged.
- **Consequences:**
  - An ASan report from the nightly job shows where the bad access happened, but not where the
    memory was allocated or freed.
  - To get those stacks, re-run the uploaded reproducer locally without the option
    (`cargo fuzz run ... <artifact>`).
  - Leak detection, crash, timeout and out-of-memory detection are unaffected.
  - The local fuzz command in decision 0035 is unchanged. Long local runs should set the same
    option.
