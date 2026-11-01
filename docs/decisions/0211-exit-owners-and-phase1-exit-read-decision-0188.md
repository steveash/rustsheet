# 0211 — exit_owners and phase1-exit read decision 0188's R1(c) rows and Gate B

- **Status:** accepted (2026-10-31, T-3428). Implements decision 0188 R1(b), R1(c), R3 and R7 in the
  scripts that list exit cells and print the Phase 1 exit.
- **Context:** T-3359 made Gate B the Phase 1 rate (`v15_checkpoint.gate_b`), and T-3427 made the milestone
  probe and ironcalc-calc rows count only the cells R1(c) names. `scripts/exit_owners.py` still listed
  every visibly-off cell of those rows, and every visibly-off cell of the report for `--all-off`. So
  `make exit-owners`, `make unowned-off` and the `owners` line of `make phase1-exit` asked for owners of
  Phase 1b evidence, R3 passes and M14-owned probe cells, which decision 0188 says never block an exit.
  `scripts/phase1_exit.py` read `phase1_met`, so a run that did not score calipers, poi or corpus-bench read
  `not met` instead of saying the gate was not measured. `scripts/` is a protected path (CLAUDE.md directive
  2), so the change needs this record.
- **Decision:**
  - The report writes `r1c_left_out`: every visibly-off cell R1(c) leaves out of the count, with its class
    (`phase1b`, `r3_pass`, `m14_owned`; `crates/rustsheet-eval/src/r1c.rs`).
  - `exit_owners.py` lists, for a milestone:
    - the probe row's `r1c_off_cells` (`visibly_off_cells` for a report from before T-3427);
    - the function-scope row's visibly-off cells, less the exit exceptions and every left-out class;
    - the attributed cells of the functions below the rate, less only the R3 passes. Gate B is the 15-digit
      share whatever the math says, so a Phase 1b cell of a Gate B suite still counts there.
  - `--all-off` and `--unattributed` drop the left-out cells of the suites outside Gate B
    (`v15_checkpoint.gate_b.suites`), and only the R3 passes of Gate B's suites.
  - `phase1_exit.py`'s `function rate` line reads Gate B: `met`, `not met`, or `n/a` with the suites the run
    did not score. `n/a` fails the exit as `not met` does, because an unmeasured criterion is never met. A
    report from before T-3359 has no `gate_b`, and for it the line reads `phase1_met` as before.
- **Consequences:**
  - The scripts list fewer cells only where decision 0188 already leaves cells out, so no exit reads as met early.
    An unclassified cell counts. `--all-off` follows Gate B and the R1(c) rows. Two exit rows read wider lists: a gen
    suite's row counts an R3 pass, and M1's frozen list counts its Phase 1b cells. Those cells are still listed per
    milestone by `exit_owners.py M<k>`, which `phase1_exit.py` runs for every Phase 1 milestone.
  - M1's frozen `m1-probes` list is unchanged: it still counts every visibly-off cell (T-3427).
  - Tests: `scripts/test_exit_owners.py` (R1(c) rows and the rate; `--all-off` inside and outside Gate B's
    suites) and `scripts/test_phase1_exit.py` (Gate B met, not met and n/a).
