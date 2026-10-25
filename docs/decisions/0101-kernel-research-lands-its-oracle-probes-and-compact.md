# 0101 — Kernel research lands its oracle probes and compact data in git, about 50 MB per task at most

- **Status:** accepted (2026-10-23, T-2464; amended 2026-10-25 for T-2513 and T-2586, which land in the same PR)
- **Context:**
  - T-2464 (incomplete beta, rounds 1-2), T-2513 (densities, round 3) and T-2586 (F density, rounds 4-5) are one
    research line with one data directory, `docs/research/t2464/`. Their evidence is oracle-filled probe cases in
    `eval/suites/probes-numeric/cases/` (excel-graph sidecars, 2026-10-23 to 2026-10-24) plus the scripts, RESULTS.md
    files and compact tables the counts in statistics.md are scored on. Measured on the PR head (raw tracked bytes;
    in parentheses the zlib-compressed size of the same blobs, which is what a clone fetches):

    | task | probe cases | research files | total |
    |---|---|---|---|
    | T-2464 | 20.5 MB, 18 cases, 167,838 cells | 20.0 MB (`cells.tsv`, `beta_exact.tsv`, w*/r2-*) | 40.5 MB (6.9 MB) |
    | T-2513 | 23.3 MB, 27 cases, 192,848 cells | 2.0 MB (r3-*) | 25.3 MB (4.4 MB) |
    | T-2586 | 12.5 MB, 23 cases, 116,460 cells | 1.7 MB (r4-*, r5-*) | 14.2 MB (2.4 MB) |
    | all three | 56.3 MB, 68 cases, 477,146 cells | 23.7 MB | 80.0 MB (13.7 MB) |

    The next largest research directory is under 1 MB.
  - The oracle values cannot be regenerated offline: they cost oracle time, and Excel for the web changes (T-2337).
    Without them the statistics.md entries and the engine tasks (T-2502..T-2513, T-2585, T-2633) could not be
    re-checked. The verifiers' own hold-outs are part of the evidence (`t2513-verify-*`, `t2586-verify-*`).
  - Caches, pickles, copies of other workers' code and tables the scripts rebuild in seconds are git-ignored
    (`docs/research/t2464/.gitignore`), and so are the 87 exploratory round-1 scripts no RESULTS.md cites.
- **Decision:** A kernel-research task may commit its oracle evidence (probe cases with sidecars, or the oracle outputs
  of probes kept in its research directory) and the compact tables its RESULTS.md files are scored on, up to about
  50 MB of raw tracked bytes per task. Anything a script rebuilds from those files is git-ignored. More than that
  needs its own decision record. Each of the three tasks above is under the cap.
- **Consequences:**
  - The PR adds about 80 MB raw (about 14 MB compressed) once: it is squash-merged, so main carries only its final
    tree.
  - The probes-numeric cases are ungated (`gate = false`). Measured with the release binary on a 4-core machine,
    they add about 12 s to `make eval` (5.3 s for the t2464 cases, 4.4 s for t2513, 2.4 s for t2586).
  - Once the engine tasks that use the data have landed, T-2586 prunes outputs that only intermediate scripts read
    (its acceptance criterion), keeping the eval cases, the RESULTS.md files and the scripts behind the statistics.md
    counts.
