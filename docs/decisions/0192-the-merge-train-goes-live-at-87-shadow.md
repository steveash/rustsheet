# 0192 — The merge train goes live at 87% shadow agreement (owner ruling)

- **Status:** accepted (2026-10-31, T-2991; owner ruling in session
  https://claude.ai/code/session_018kHzCtGoS4gXMYKAZWDHEd: the owner approved switching at 87%
  agreement, then asked not to wait for the 24 h mark)
- **Context:**
  - SPEC-0027's rollout bar for `mode = "train"` asks for:
    - at least 10 scored shadow trains over at least 24 h;
    - at least 90% prediction agreement, with every disagreement explained;
    - zero tree-check mismatches;
    - a load bar.
  - `make merge-train-stats` (decision 0191) reported at 2026-10-31T08:07Z: 42 scored trains over
    22 h; 151 of 174 scored
    predictions agree (87%); 0 tree-check failures; 0 red trains in 47; PR CI p90 7.8 min during
    shadow against 6.5 min before (+20%, the bar's limit); job queue p90 3 s.
  - The 23 disagreements, explained:
    - **Stranded, 12** (`merge` predicted): #2406 ×2, #2430 ×7, #2436 and #2474 ×2. These PRs
      were marked `ready-to-merge` and then sat unmerged, because in shadow mode only their own
      session merges them and those sessions had ended. When a cleanup merged them later, each
      first needed a conflict merge or a renumber, which the scoring counts as a disagreement.
      In `train` mode they would have merged in the predicted train.
    - **Changed after marking, 6** (`merge` predicted): #2409, #2418, #2425, #2426 ×2 and #2431.
      Their sessions kept pushing after the PR was marked ready: review fixes, doc edits, and
      renumbers because another merge took their task id. A prediction made on the marked head
      cannot see later pushes. In `train` mode the marked head merges, and later work comes in
      a new PR.
    - **Batch order, 5** (`eject:conflict` predicted): #2350 ×3, #2324 and #2406. Each one
      conflicted with an earlier PR in the predicted batch, not with `main`. The per-PR flow
      merged them in another order, so their next head merged `main` cleanly. A live train
      ejects such a PR and builds it into the next train.
  - None of the 23 is a wrong tree. The disagreements come from how shadow mode differs from the
    real flow, not from the train's logic.
- **Decision:** `scripts/merge_train.toml` moves to `mode = "train"`.
  - The owner waived two points of the bar:
    - the 90% agreement point, given the explanations above;
    - the 24 h span. The 42 scored trains cover 22 h, and the owner judged that evidence enough
      to leave shadow mode now.
  - Every other point holds:
    - at least 10 scored trains, the newest within 2 h (shadow run 41181616335 at 07:41Z, stats
      at 08:07Z);
    - zero tree-check mismatches;
    - the load bar: PR CI p90 at +20%, the limit, on rounded figures (7.8/6.5 min), and queue p90
      at 3 s;
    - the red-train rate: 0/47 against the spec's p = 0.09.
  - The counts measure different things. There are 42 scored *predictions* (trains with a
    prediction), 47 completed shadow *trains* that ran CI, and 51 shadow CI *runs*, of which 50
    are green and 1 was cancelled.
  - The waivers cover this one flip only. A later re-flip from `lock` to `train`, for example
    after an emergency flip, meets SPEC-0027's full bar again or needs its own owner ruling.
  - Owner step 3 (SPEC-0027 "Owner steps") is done:
    - The owner confirmed on 2026-10-31 that the deployed routine prompts match
      docs/process/routines.md.
    - The five worker routines' prompts were checked word for word against it, `train`-mode
      clauses included.
  - The rest of SPEC-0027 is unchanged: T-2990 measures the live train for a week, and a
    tree-check mismatch at any time means the emergency flip back to `lock` (decision 0157) with a
    P0 task.
- **Consequences:**
  - Workers and the steward mark green, approved PRs ready (`make mark-ready`) and no longer
    merge them. The merger routine merges ready PRs in trains, every 30 minutes. The routine
    prompts already carry the `train`-mode clauses (decision 0157, T-2996).
  - Shadow-era `ready-to-merge` labels on PRs whose readiness no longer verifies are removed by
    the first live train (`merge_train.py` unlabels them). So they do not trip the "stranded"
    daily threshold for long.
  - Stranded `ready-to-merge` PRs should stop. The "stranded" daily threshold in
    `make merge-train-stats` becomes the check that they have.
  - T-2990's checkpoint judges the first live week against SPEC-0027's metrics.
