# 0125 — The eval CI job judges eval-check and the evidence floor from one suite run

- **Status:** accepted (2026-10-25, owner request to shorten PR CI, T-2761). It changes how two
  gates are run, not what they judge.
- **Context:** PR CI wall time is how long the merge lock (decision 0075) is held, so it caps how
  many PRs merge per hour. The eval job ran `make eval-check`, then `make evidence-floor`. Each
  `make` target starts with `cargo build --release`, and the engine's build script re-runs on
  every cargo call (it watches `crates/rustsheet-engine/.git/HEAD`, which does not exist), so the
  second step rebuilt the whole engine before running the vendored suites a second time. On PR
  #1842 (run 40242117468) the floor step took 2:11: 1:57 of rebuild and 13 s of suites. Over six
  PR runs on 2026-10-25 the step took 1.6–2.2 min of an eval job of 3.5–4.9 min.
- **Decision:** `rustsheet-eval check --evidence-floor` (`make eval-gates`) runs the gate suites
  and the other vendored suites in one run and judges both gates from it: the baseline and
  visible-parity gates from the gate suites' results, exactly as `check`, and the floor from the
  vendored suites' results, exactly as `evidence-floor`. CI's eval job and `make check` run it
  in place of the two targets. It reads no earlier run, so a result from another tree can never
  be judged. `make eval-check` and `make evidence-floor` stay as they were for use on their own.
- **Consequences:** every gate that ran before still runs, on the same results: on main the
  combined output equals the two commands' outputs one after the other, and the exit status is
  the same. The eval job drops the second release build. A failing baseline gate no longer hides
  the floor's verdict (CI skipped the second step after a failed first one). Later gates that
  need a full run of the vendored suites should join this run rather than add another one.
