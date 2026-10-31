# 0200 — The merge train checks a stale test merge locally

- **Status:** accepted (2026-10-31, T-3455)
- **Context:**
  - Step 7.2 of SPEC-0027 (the pre-squash check) waited up to 2 minutes for GitHub to recompute
    each PR's test merge (`merge_commit_sha`) on the current `main`, then compared its tree with
    the train's.
  - GitHub recomputes a PR's test merge when the PR changes, not when `main` moves. So every
    open PR's test merge sits on whatever `main` was at its last push. On 2026-10-31 these were:
    - #2533 on a3df1f16;
    - #2104 and #2373 on b1bdc1c2;
    - #2524 on 1248554a.
  - Every live train stopped at step 7.2 with nothing merged:
    - 11:51 UTC, run 41198551221;
    - 13:54, 14:42 and 14:55;
    - 16:19, after #2528 tested green in run 519ae8ee.
  - Re-setting the PR's base to `main` (`PATCH pulls/{n}`) updated the PR's `base.sha` at once.
    It recomputed #2104's test merge within about 3 minutes once. It did not recompute #2528's in
    more than 10 minutes. It is not a reliable trigger.
  - `git merge-tree --write-tree <squash^1> <PR head>` gives exactly the tree of GitHub's squash
    for the last four squashes on main. Two of them were on PRs whose test merge was stale, so
    GitHub merges afresh at squash time:

    | PR | squash | GitHub's squash tree | local merge tree |
    |---|---|---|---|
    | #2533 | bc8d263e | 210676dc | 210676dc |
    | #2524 | 06c142a2 | 25f50995 | 25f50995 |
    | #2532 | 625d27be | daa1ff73 | daa1ff73 |
    | #2588 | d4c03854 | 61205076 | 61205076 |
  - The tests' fake GitHub recomputed on every read, and shadow mode never squashes, so step 7.2
    never ran against real GitHub before T-2991 turned the train on.
- **Decision:** step 7.2 no longer waits for GitHub's test merge.
  - It reads the PR once. When GitHub's test merge sits on the expected `main`, its tree must be
    the train's tree, as before.
  - Otherwise the stage checks two things locally. First, `origin/main` must still be the
    expected `main`. Second, `git merge-tree --write-tree <main> <head>` must succeed and give
    the train's tree.
  - A different tree is a P0, as before. So is a moved `main`. A conflict stops the train.
  - The post-squash check is unchanged. It compares the squash GitHub actually made with the
    tested tree, and opens a revert PR with a P0 when they differ.
  - The test fake keeps a stale test merge until the PR changes (`test_bases`). Tests cover:
    - a stale and a computing test merge (merged through the local check);
    - a stale one whose local merge has another tree (P0);
    - a stale one whose local merge conflicts (stop).
- **Consequences:**
  - Live trains squash their green members. The check costs one local `git merge-tree` per
    member instead of a wait of up to 2 minutes.
  - Before, the pre-squash check used GitHub's own merge of the PR. Now, when GitHub's copy is
    stale, it uses the same merge computed by git locally. If GitHub's squash ever produced a
    different tree from git's merge, only the post-squash check would see it. That check is
    mandatory, so at most one squash with an untested tree reaches `main`. The train then stops
    with a P0 and opens a revert PR, which stays open (the train never merges it) until someone
    merges it.
  - No new GitHub write and no change to the merger routine's prompt.
