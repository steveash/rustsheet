# 0245 — New eval cases come with owners: a gate on visibly-off cells of added or refilled cases

- **Status:** accepted (2026-11-01, T-3793).
- **Context:**
  - Gap triage on 2026-11-01 (origin/main fafb6c6f, `scripts/exit_owners.py --all-off`) found every
    non-deferred unowned visibly-off cell came from a PR that added its case with no owner the
    script can read: T-3568's two probes-numeric `bup-terms` cases (7,026 BETA.DIST cells, now
    T-3792), T-3532's follow-ups naming cells as hyphen spans (decision 0136: a hyphen span names
    only its ends), and T-1683's abbreviated case name. Gap triage was the first place those cells
    got an owner.
  - `make unowned-off` and `make exit-owners` exist but gate nothing, and probes-numeric is
    outside the visible-parity lock (decisions 0088, 0093, 0219).
  - Running `scripts/new_case_owners.py --base` T-3568's parent commit lists exactly those cells:
    3,602 in `t3568-r5-bup-terms-tune`, 3,424 in `-terms-hold` and 10 in
    `t3532-numeral-digit-fields` (the hyphen spans), across 53 cases in 75 s.
- **Decision:**
  - *New cases.* A case is new when the branch adds or modifies its case file (`.case.json`,
    `.xlsx`, `.xlsm`) or its `.oracle.json` sidecar against the merge base with `origin/main`
    (a refill counts), the working tree and untracked files included. Vendored suites (`fetch`)
    and deleted cases are not checked. A case the branch does not touch is not checked, whatever
    its baseline says.
  - *Cells.* `exit_owners.all_off_cells`: decision 0188 R1(c)'s visibly-off set (attributed and
    unattributed, less R1(c)'s left-out cells, plus the probe rows' counted cells).
  - *Owners.* `exit_owners`' rule unchanged (`unowned_off`), with the case names computed over
    every case id of the repository's suites, so a one-case run names its case as a full run
    would. A cell owned only by an open `defer:` task passes.
  - *Where it runs.* `scripts/new_case_owners.py`, the last step of `make eval-gates` (so CI's
    eval job and `make check`). It runs each new case alone (`rustsheet-eval run --suite S
    --filter F --out <tmp>`, under a second a case) rather than reusing the gate run, which builds
    no report and leaves out probes-numeric; a branch with no new case runs nothing. CI's eval job
    checks out with full history (`fetch-depth: 0`) for the merge base. It fails closed (exit 2)
    when there is no merge base or a run fails.
  - Failures list `case!Sheet!A1<TAB>formula<TAB>expected<TAB>actual`.
- **Consequences:**
  - A probe or gen PR files or amends the owners of what it adds in the same PR; the excel-probe
    skill's step 4 says so. Gap triage stops being where cells first get an owner.
  - A PR that edits an existing case's file or sidecar answers for every unowned visibly-off cell
    of that case, not only the ones it changed.
  - The ownership rule itself (decision 0136, T-3623) is unchanged.
