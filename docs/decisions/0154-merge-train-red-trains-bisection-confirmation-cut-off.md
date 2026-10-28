# 0154 — Merge-train red trains: bisection, confirmation, cut-off and resume

- **Status:** accepted (2026-10-28, T-2998). It implements SPEC-0027 section 6 in `train` mode.
  Nothing is in force: `scripts/merge_train.toml` says `mode = "lock"`.
- **Context:** T-2994 (decision 0151) stopped on a red train with nothing merged. SPEC-0027
  section 6 requires a binary search with a cost bound of 1 + ⌈log₂ n⌉ + 1 runs per red train
  that holds across holds, so a cut-off must merge what it proved green and resume, not restart.
- **Decision:**
  - A red train `P_1..P_n` on `M0` is searched for its first red prefix (`lo` green, `hi` red,
    `mid = ⌊(lo + hi) / 2⌋`), each prefix pushed as a record commit of its own. `P_1..P_lo`
    merges through decision 0151's step 7, then `P_lo+1` gets one unconditional confirmation run
    alone on the new `main`. Red: it is ejected (`ready-to-merge` removed, `merge-train-ejected`
    added, a comment with the failing jobs (`@` neutralised), links to the red and confirmation
    runs built from their integer ids, and `<!-- merge-train-ejected v1 head: <sha> reason:
    ci-red -->`). Green: no ejection, it merges, and a P1 `possible flaky test or interaction` is
    queued on `Stage.tasks` with both run links. The rest `P_lo+2..P_n` is rebuilt and tested as
    one new train on the new `main`, in the same hold.
  - The step 6 gate takes n = the PRs the run can merge: the batch, `mid`, or 1. A failed gate,
    or an infrastructure failure (no run, a run over 10 min, a cancelled, timed-out or
    never-started run or job) that repeats on its one retry (a fresh record commit, so a fresh
    run), is a cut-off: the known-green prefix merges, then the state (`bisect-base`, `known-red`
    with heads, `known-green-prefix: 0`) is pushed to `merge-train/current` as a `[skip ci]`
    commit, and the hold stops. A cut-off before the confirmation records the single culprit.
  - The next hold reads that commit through the commits API and resumes (`hi` = the known-red
    count) only when `main` is still `bisect-base` and the batch starts with the same PRs at the
    same heads; otherwise it restarts, logging the reason and writing `restart: <reason>` into
    the next record commit.
  - The state record is parsed strictly (exact header, three lines, 40-hex shas) and is only an
    optimisation: anyone who can push the branch can forge one, but a PR is ejected only after its
    own confirmation run is red, and only prefixes whose CI ran green on that exact tree merge.
- **Consequences:**
  - Filing the queued P1 as a task PR uses T-3055's filer (it already files the P0s); until then
    the P1 is in the run's log and `Stage.tasks`.
  - T-2995's shadow mode reuses `Stage.resolve`'s search on `M0` for its predictions.
