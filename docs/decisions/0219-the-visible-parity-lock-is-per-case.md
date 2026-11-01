# 0219 — The visible-parity lock is per case

- **Status:** accepted (2026-10-31, T-3573; the repository owner asked for this change in session
  https://claude.ai/code/session_018kHzCtGoS4gXMYKAZWDHEd). Amends decisions 0088, 0093 and 0207; supersedes 0214.
- **Context:**
  - `eval/baseline/visible-parity.json` held one number, a cap on all in-scope visibly-off
    cells (decision 0088). `update-baseline` raised it only up to the visibly-off cells of cases
    new to the tree's baseline, and the steward lowered it daily (decision 0093).
  - A probe PR that added visibly-off cases while `main` had slack below the cap wrote no new
    cap, and its CI passed. Once `main` used up the slack, or the steward lowered the cap, the PR
    merged with `main` failed `check` (for example 2,535 cells against a cap of 2,427). By then its
    cases were in its baseline, so they no longer counted as new and no re-record could clear it.
  - Every PR that did raise the cap changed the same line, so any two of them conflicted. On
    2026-10-31 this ejected about 9 PRs from the merge train.
- **Decision:**
  - The lock is `eval/baseline/visible-parity.txt`: a generated header, then one sorted
    `<suite>/<case><TAB><count>` line for each case of the vendored gated suites that has in-scope
    visibly-off cells. It has no total line, so PRs that touch different cases change different
    lines. This is the same line-oriented shape as the known-failure lists
    `eval/baseline/<suite>.txt`. A separate file keeps the per-suite lists' format, and the code
    that reads them, unchanged.
  - `check` fails each case whose visibly-off count is above its line; a case with no line allows
    none. It prints each such case and the total excess. Lines of cases the run no longer has are
    ignored, because the suite's own baseline already fails a removed case.
  - `update-baseline` (`make eval-baseline`) always writes the line of a case new to its suite's
    committed baseline, so a PR that adds a case carries the case's line. It raises an existing
    case only with `--allow-regressions`, which still needs a decision record. It lowers a line, or
    drops it at 0, only with `--lower-visible-lock` (the steward's daily `make
    eval-lower-visible-lock`, decision 0093). Without that flag a drop is slack, as before. It drops
    the lines of removed cases.
  - By default a case is "new" exactly as before: its id is absent from its suite's committed
    baseline. A case whose checked-cell count changed is not new, so a rise there needs
    `--allow-regressions`.
  - `update-baseline --new-cases <file>` (`make eval-baseline NEW_CASES=<file>`) names the new
    cases explicitly, one case id (`<suite>/<case>`) or repository-relative case file path per
    line. With it, only the listed cases are new. Without `--allow-regressions` it refuses a rise
    in any unlisted case, and a line for any unlisted case that has visibly-off cells and no line,
    even one absent from the tree's baseline. The caller derives the list from the case files the
    PR adds: `git diff --diff-filter=A --name-only origin/main...HEAD -- eval/suites`.
    `make catch-up` passes this list, without the pathspec so that the `ironcalc-calc` workbooks
    under `crates/` count too; the tool ignores every entry that is not a case of a gated suite.
    The owner added this on 2026-10-31, after a review of the interim ruling. Five queued probe
    PRs (#2581, #2600, #2620, #2625, #2564) predate this file, so their cases have no lines. Once
    a catch-up merges `main` into them, their cases already sit in the tree's baseline, so "new to
    the baseline" no longer identifies them. A case absent from the merged baseline could also be
    one of `main`'s. The list records lines only for the PR's cases, and never adds or raises a
    line for a case on `main`. That was how a net count hid regressions.
  - The merge tooling (decision 0207) treats the lock as an eval-gated list:
    `merge_ready.Repo.checked_baselines` includes it.
    - For a case whose case file is on `main` (the merge's second parent), a carry-over merge may
      record a line only at or below `main`'s line, which counts 0 when there is none. A case the
      PR adds may have any line, because `main` has no line for it to hide a regression behind.
      Malformed or repeated lines are refused (`scripts/visible_lock.py`, `visible_rises`).
    - `make catch-up` resolves a conflict on the file three-way, per case, from the merge base. If
      only one side changed a case, that side's count wins; if both did, the lower count wins.
      This keeps a lowering on `main` that the PR did not touch, and a line that `main` dropped
      stays dropped.
    - The first version kept each case at the higher of the two sides. That undid the steward's
      lowerings on every catch-up, and the carry accepted the result (review of #2660). Taking
      `main`'s side instead would drop the PR's new lines.
    - Catch-up then re-records with `NEW_CASES` set to the case files the PR adds. It refuses,
      before pushing, if any of `main`'s cases ends above `main`'s line: a PR whose raise was
      sanctioned with `--allow-regressions` is merged by hand and reviewed again.
    - With `--new-cases`, a listed case that already has a line is not new, so the list cannot
      raise its line. It is not refused outright, because catch-up lists every case file a PR adds,
      and PRs made after this decision have already recorded their lines.
    - PRs from before this decision modified the retired `eval/baseline/visible-parity.json`. When
      `main` no longer has that file, catch-up resolves the modify/delete conflict by deleting it,
      and the carry accepts only that deletion.
    - `crates/rustsheet-eval/tests/data/visible-lock-lines.txt` is a corpus of lock lines that both
      the Rust and the Python parser read in tests, so the two stay in step. Both take ASCII digits
      only, so `+3` and `²` are malformed.
  - **Migration:** the file was generated by the tool, not written by hand (CLAUDE.md: never
    hand-edit `eval/baseline/`). On T-3573's branch, cut from `main` at c05a827f0, the json was
    removed and `rustsheet-eval update-baseline --allow-regressions --lower-visible-lock` was run
    once, giving 2,507 cells in 274 cases. The same command ran again after `main` was merged in
    at b98b3c00d, giving 2,499 cells in 269 cases (only drops). Both times every suite baseline
    reported 0 regressions and stayed unchanged. Both results are below the cap of 2,509, so the
    migration loosens nothing. This is the one sanctioned use of `--allow-regressions` here, as
    decision 0088's bootstrap was.
  - Decision 0214 (a new-coverage rise keeps `main`'s slack) concerned the single cap. With
    per-case lines there is no shared slack, so 0214's rule and code go away with the cap.
- **Consequences:**
  - Slack in one case no longer hides a rise in another. The ratchet is stricter than the single
    cap: a change that makes an already-failing cell visibly off fails CI in that cell's case, even
    if the same PR fixes cells elsewhere.
  - A probe PR's lines are in its own diff. Merging it after `main` moved cannot fail on another
    PR's cells, and two probe PRs conflict only when their cases' lines are adjacent, which
    `make catch-up` resolves.
  - Fix PRs still leave the file alone. The steward's daily run lowers only the lines that
    dropped.
  - `eval/baseline/visible-parity.json` is gone. The report's "Visibly off" section is unchanged.
